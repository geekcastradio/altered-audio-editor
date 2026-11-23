// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! What each process chain action does, with no dialog, in Rust: the same edits
//! the menu commands make, with their settings read from the chain. Mirrors the
//! pure parts of `src/app/chain-actions.ts` (the Loudness Target, Voice Chain
//! Preset and QuickPass steps and the mic settings), tested against
//! `shared-fixtures/chain-actions.json`. Steps that measure audio or call the
//! analysis stay in the interface. The region the browser would guess from the
//! time zone and the custom loudness target are passed in.

use crate::clip_edit::LockedError;
use crate::js_compat::{js_number_to_string, js_round, js_to_fixed, js_trim};
use crate::loudness_check::{predict_export, targets_for, Report};
use crate::quickpass_edit::{duck_op, remove_op};
use crate::quickpass_scan::{data, find_all, ClipInfo, ClipLevels, Settings};
use crate::voice_chain::{apply_preset, builtin_presets, chain_of};
use serde_json::{json, Value};
use std::collections::HashSet;

/// The Master fader's range, matching its dial.
const MASTER_MIN: f64 = -60.0;
const MASTER_MAX: f64 = 12.0;

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// An edit (or none) and the sentence a chain step reports.
pub type Step = (Option<Value>, String);

/// The Master change that lands the mix on a loudness target, or none when it is
/// already close enough, silent, or cannot be raised without passing the
/// target's peak limit. Uses the same rule as the export.
pub fn loudness_target_op(
    report: Report,
    master_db: f64,
    target_id: &str,
    tolerance_db: f64,
    region: &str,
    custom: (f64, f64),
) -> Step {
    let list = targets_for(region, custom.0, custom.1);
    let t = list.iter().find(|t| t.id == target_id).unwrap_or(&list[0]);
    let pred = predict_export(report, Some(t), t.mono);
    if !pred.loudness.is_finite() {
        return (None, "the mix is silent or too short to measure.".into());
    }
    if pred.gain.abs() <= tolerance_db.max(0.0) || pred.gain.abs() < 0.05 {
        return (
            None,
            format!(
                "already at {} LUFS, close enough to {}.",
                js_to_fixed(pred.loudness, 1),
                js_number_to_string(t.lufs)
            ),
        );
    }
    let next = MASTER_MAX.min(MASTER_MIN.max(js_round((master_db + pred.gain) * 10.0) / 10.0));
    if next == master_db {
        return (None, "the Master is already at its limit.".into());
    }
    let how = format!(
        "Master {} {} dB for {} LUFS",
        if pred.gain > 0.0 { "raised" } else { "lowered" },
        js_to_fixed((next - master_db).abs(), 1),
        js_number_to_string(t.lufs)
    );
    let why = if pred.limited_by_peak {
        format!(
            ", stopped by the {} dBTP peak limit (lands at {} LUFS)",
            js_number_to_string(t.peak),
            js_to_fixed(pred.loudness, 1)
        )
    } else {
        String::new()
    };
    (
        Some(json!({ "op": "set_master_volume", "volume_db": next })),
        format!("{how}{why}."),
    )
}

fn num(v: Option<&Value>, fallback: f64) -> f64 {
    match v.and_then(Value::as_f64) {
        Some(x) if x.is_finite() => x,
        _ => fallback,
    }
}

/// A chain's mic mix settings in the names the analysis takes.
pub fn mic_options(p: &Value) -> Value {
    json!({
        "openDb": num(p.get("open_db"), 8.0),
        "marginDb": num(p.get("margin_db"), 6.0),
        "reduceDb": num(p.get("reduce_db"), 12.0),
        "fadeMs": num(p.get("fade_ms"), 150.0),
        "attackMs": num(p.get("attack_ms"), 15.0),
        "holdMs": num(p.get("hold_ms"), 250.0),
    })
}

/// The tracks a step names: every track with audio, or the one with that name.
fn pick_tracks<'a>(doc: &'a Value, which: &str) -> Result<Vec<&'a Value>, String> {
    let tracks = arr(doc, "tracks");
    let with_audio: Vec<&Value> = tracks
        .iter()
        .filter(|t| !arr(t, "clips").is_empty())
        .collect();
    let name = js_trim(which).to_lowercase();
    if name.is_empty() || name == "all" {
        return Ok(with_audio);
    }
    let hit: Vec<&Value> = tracks
        .iter()
        .filter(|t| js_trim(text(t, "name")).to_lowercase() == name)
        .collect();
    if hit.is_empty() {
        return Err(format!("There is no track called \"{which}\"."));
    }
    Ok(hit)
}

/// Gives each named track the Voice Chain preset, as one edit. A voice chain
/// replaces the track's chain; a single-effect preset adds its effect to the end.
pub fn voice_chain_preset_op(
    doc: &Value,
    preset_name: &str,
    which: &str,
    saved: &[Value],
    known: &HashSet<String>,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Step, String> {
    let builtin = builtin_presets();
    let Some(p) = saved
        .iter()
        .chain(builtin.iter())
        .find(|x| text(x, "name") == preset_name)
    else {
        return Err(format!(
            "There is no Voice Chain preset called \"{preset_name}\"."
        ));
    };
    let tracks = pick_tracks(doc, which)?;
    if tracks.is_empty() {
        return Ok((None, "no audio on the tracks.".into()));
    }
    let mut ops = vec![];
    for t in &tracks {
        let Some(next) = apply_preset(
            &chain_of(t.get("effects").filter(|e| !e.is_null())),
            p,
            known,
            new_id,
        ) else {
            return Err(format!("The Voice Chain on {} is full.", text(t, "name")));
        };
        ops.push(json!({ "op": "set_effects", "target": { "kind": "track", "id": t["id"] }, "chain": next }));
    }
    let n = ops.len();
    let op = if n == 1 {
        ops.remove(0)
    } else {
        json!({ "op": "batch", "ops": ops })
    };
    Ok((
        Some(op),
        format!(
            "{} on {} track{}.",
            text(p, "name"),
            n,
            if n == 1 { "" } else { "s" }
        ),
    ))
}

/// The audio a QuickPass step scans: clips that play, on tracks that play, as
/// the panel does. The ids come in track and clip order.
pub fn quick_pass_clips(doc: &Value) -> (Vec<String>, ClipInfo) {
    let mut ids = vec![];
    let mut info = ClipInfo::new();
    for t in arr(doc, "tracks") {
        if t.get("muted").is_some_and(crate::mix_routing::truthy) {
            continue;
        }
        for c in arr(t, "clips") {
            let truthy = |k: &str| c.get(k).is_some_and(crate::mix_routing::truthy);
            if !truthy("locked") && !truthy("muted") {
                ids.push(text(c, "id").to_string());
                info.insert(
                    text(c, "id").to_string(),
                    (text(t, "id").to_string(), text(c, "name").to_string()),
                );
            }
        }
    }
    (ids, info)
}

/// The QuickPass preset a step names, with the step's own gap setting. An error
/// when a chain cannot run it.
pub fn quick_pass_settings(preset_name: &str, ripple: &str) -> Result<Settings, String> {
    let d = data();
    let Some(p) = arr(d, "presets")
        .iter()
        .find(|x| text(x, "name") == preset_name)
    else {
        return Err(format!(
            "There is no QuickPass preset called \"{preset_name}\"."
        ));
    };
    let mut merged = d["defaults"].as_object().cloned().unwrap_or_default();
    for (k, v) in p["settings"].as_object().into_iter().flatten() {
        merged.insert(k.clone(), v.clone());
    }
    let mut s = Settings::from_json(&Value::Object(merged));
    if !["silence", "noise", "audio"].contains(&s.scan.as_str()) {
        let types = arr(d, "scanTypes");
        let label = types
            .iter()
            .find(|t| t["id"] == s.scan.as_str())
            .or(types.first())
            .map_or("", |t| text(t, "label"))
            .to_lowercase();
        return Err(format!("\"{}\" looks for {}, which needs the QuickPass panel. Chains run the silence, noise and audio scans.", text(p, "name"), label));
    }
    if s.action != "remove" && s.action != "duck" {
        return Err(format!(
            "\"{}\" ends by {} what it finds. Chains run presets that remove or turn down.",
            text(p, "name"),
            if s.action == "split" {
                "splitting"
            } else {
                "marking"
            }
        ));
    }
    if !["none", "track", "all"].contains(&ripple) {
        return Err(format!(
            "The gap setting must be none, track or all, not \"{ripple}\"."
        ));
    }
    s.ripple = ripple.to_string();
    Ok(s)
}

/// What a QuickPass step does with levels already measured.
pub fn quick_pass_op(
    doc: &Value,
    levels: &[ClipLevels],
    s: &Settings,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Step, LockedError> {
    let rate = doc
        .get("sample_rate")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let (_, info) = quick_pass_clips(doc);
    let found = find_all(levels, &info, s, rate);
    if found.is_empty() {
        return Ok((None, "found nothing.".into()));
    }
    let op = if s.action == "remove" {
        remove_op(doc, &found, &s.ripple, s.keep_ms, new_id)?
    } else {
        duck_op(doc, &found, s.duck_db)
    };
    let n = found.len();
    if op.is_none() {
        return Ok((
            None,
            format!("found {n}, but nothing was long enough to change."),
        ));
    }
    Ok((
        op,
        format!(
            "{} {} {}.",
            if s.action == "remove" {
                "removed"
            } else {
                "turned down"
            },
            n,
            if n == 1 { "stretch" } else { "stretches" }
        ),
    ))
}
