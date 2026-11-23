// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Small panel rules in Rust: which node of the Parametric EQ graph stands
//! for which controls, the edits that delete selected clips (audio and MIDI),
//! file-safe workspace ids, and the smallest a group of panels may get.
//! Mirrors parts of `src/panels/voice-chain/eq-graph.ts`,
//! `src/panels/editor/midi.ts` and `src/app/docking/registry.ts`, tested
//! against `shared-fixtures/panel-rules.json`.

use crate::voice_chain::{param_value, ParamInfo};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k)
        .and_then(Value::as_array)
        .map_or(&[], |a| a.as_slice())
}

fn node_label(group: &str) -> String {
    match group {
        "High-pass" => "HP".into(),
        "Low shelf" => "L".into(),
        "Band 1" => "1".into(),
        "Band 2" => "2".into(),
        "Band 3" => "3".into(),
        "High shelf" => "H".into(),
        "Low-pass" => "LP".into(),
        other => other.chars().take(2).collect(),
    }
}

/// The nodes of an EQ, in order from low to high, from its controls' groups.
/// Each is `{ label, name, freq, gain, q, on }`, the controls as given.
pub fn eq_nodes(info: &Value) -> Vec<Value> {
    let params = arr(info, "params");
    let mut groups: Vec<&str> = Vec::new();
    for p in params {
        if let Some(g) = p
            .get("group")
            .and_then(Value::as_str)
            .filter(|g| !g.is_empty())
        {
            if !groups.contains(&g) {
                groups.push(g);
            }
        }
    }
    let mut out = Vec::new();
    for g in groups {
        let ps: Vec<&Value> = params
            .iter()
            .filter(|p| p.get("group").and_then(Value::as_str) == Some(g))
            .collect();
        let Some(freq) = ps.iter().find(|p| p["unit"] == "Hz") else {
            continue;
        };
        let find = |f: &dyn Fn(&Value) -> bool| {
            ps.iter()
                .find(|p| f(p))
                .map_or(Value::Null, |p| (*p).clone())
        };
        out.push(json!({
            "label": node_label(g),
            "name": g,
            "freq": freq,
            "gain": find(&|p| p["unit"] == "dB"),
            "q": find(&|p| p["key"].as_str().is_some_and(|k| k.ends_with("_q"))),
            "on": find(&|p| p.get("toggle") == Some(&json!(true))),
        }));
    }
    out
}

/// Whether a node is in use: always, or when its on switch is on.
pub fn node_on(slot: &Value, node: &Value) -> bool {
    let on = &node["on"];
    on.is_null() || param_value(slot, &ParamInfo::from_json(on)) >= 0.5
}

/// Every MIDI clip id in the session.
pub fn all_midi_clip_ids(doc: &Value) -> Vec<String> {
    arr(doc, "tracks")
        .iter()
        .flat_map(|t| arr(t, "midi_clips"))
        .filter_map(|c| c["id"].as_str().map(String::from))
        .collect()
}

/// The edits that delete the selected clips, audio and MIDI alike: one
/// `remove_clips` for the audio ones and one `set_midi_clips` per MIDI track
/// touched.
pub fn delete_clips_ops(doc: &Value, ids: &BTreeSet<String>) -> Vec<Value> {
    let mut ops = Vec::new();
    let audio: Vec<&str> = arr(doc, "tracks")
        .iter()
        .flat_map(|t| arr(t, "clips"))
        .filter_map(|c| c["id"].as_str())
        .filter(|id| ids.contains(*id))
        .collect();
    if !audio.is_empty() {
        ops.push(json!({ "op": "remove_clips", "ids": audio }));
    }
    for t in arr(doc, "tracks") {
        let clips = arr(t, "midi_clips");
        if clips
            .iter()
            .any(|c| c["id"].as_str().is_some_and(|id| ids.contains(id)))
        {
            let kept: Vec<&Value> = clips
                .iter()
                .filter(|c| !c["id"].as_str().is_some_and(|id| ids.contains(id)))
                .collect();
            ops.push(json!({ "op": "set_midi_clips", "track_id": t["id"], "clips": kept }));
        }
    }
    ops
}

/// A file-safe id for a new workspace name that does not clash with `taken`.
pub fn workspace_id(name: &str, taken: &BTreeSet<String>) -> String {
    let lower = name.to_lowercase();
    let mut base = String::new();
    let mut gap = false;
    for c in lower.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            if gap && !base.is_empty() {
                base.push('-');
            }
            gap = false;
            base.push(c);
        } else {
            gap = true;
        }
    }
    let base: String = base.chars().take(48).collect();
    let base = if base.is_empty() {
        "workspace".to_string()
    } else {
        base
    };
    let mut id = base.clone();
    let mut n = 2;
    while taken.contains(&id) {
        id = format!("{base}-{n}");
        n += 1;
    }
    id
}

/// The smallest a panel is allowed to get, in pixels, so its controls never squeeze together.
pub fn min_size(id: &str) -> Option<(f64, f64)> {
    Some(match id {
        "files" => (280.0, 150.0),
        "voice-chain" | "batch-convert" | "quickpass" => (300.0, 36.0),
        "editor" => (520.0, 260.0),
        "mixer" => (360.0, 220.0),
        "levels" => (260.0, 70.0),
        "audio-fx" | "sound-types" | "loudness-match" => (280.0, 160.0),
        "time-info" => (220.0, 110.0),
        "history" => (220.0, 120.0),
        "video" => (320.0, 220.0),
        "takes" | "drive-manager" | "markers" | "review" | "playlist" => (260.0, 140.0),
        "soundboard" => (240.0, 200.0),
        "libraries" => (260.0, 200.0),
        "metadata" => (280.0, 200.0),
        "properties" => (260.0, 160.0),
        "phase-wheel" => (200.0, 160.0),
        "space" => (260.0, 220.0),
        _ => return None,
    })
}

/// The size limit for a group of panels: the largest of each.
pub fn group_min(ids: &[&str]) -> (f64, f64) {
    ids.iter().fold((0.0, 0.0), |m, id| {
        let (w, h) = min_size(id).unwrap_or((0.0, 0.0));
        (f64::max(m.0, w), f64::max(m.1, h))
    })
}
