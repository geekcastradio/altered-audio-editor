// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Split a clip into frequency bands on new tracks, in Rust.
//!
//! This mirrors `src/panels/properties/band-split.ts` and is tested against
//! `shared-fixtures/band-phase-fit.json`, which the TypeScript test reads too.
//! Session edits are JSON, as the interface sends them to the core.

use crate::js_compat::{is_js_space, js_string_to_number};
use crate::mix_routing::truthy;
use serde_json::{json, Map, Value};

pub const BAND_MIN_HZ: f64 = 40.0;
pub const BAND_MAX_HZ: f64 = 16_000.0;
/// Two edges closer than this ratio make the bands add up unevenly.
pub const BAND_MIN_RATIO: f64 = 6.0;

/// Reads "200, 2000" (commas, spaces or semicolons, an optional "Hz") into
/// edges in Hz; None when it is not numbers.
pub fn parse_edges(text: &str) -> Option<Vec<f64>> {
    let parts: Vec<&str> = text
        .split(|c: char| is_js_space(c) || c == ',' || c == ';')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return None;
    }
    let mut nums = Vec::with_capacity(parts.len());
    for p in parts {
        let bytes = p.as_bytes();
        let body = if bytes.len() >= 2 && bytes[bytes.len() - 2..].eq_ignore_ascii_case(b"hz") {
            &p[..p.len() - 2]
        } else {
            p
        };
        let n = js_string_to_number(body)?;
        if !n.is_finite() {
            return None;
        }
        nums.push(n);
    }
    Some(nums)
}

/// What is wrong with the edges, or None when they can be used.
pub fn check_edges(edges: &[f64]) -> Option<String> {
    if edges.is_empty() || edges.len() > 2 {
        return Some("Give one frequency for two bands, or two for three bands.".to_string());
    }
    // Two comparisons, not a range: like the TypeScript, a NaN edge passes here.
    #[allow(clippy::manual_range_contains)]
    let out_of_range = edges.iter().any(|&e| e < BAND_MIN_HZ || e > BAND_MAX_HZ);
    if out_of_range {
        return Some(format!(
            "Each frequency must be between {BAND_MIN_HZ} and {BAND_MAX_HZ} Hz."
        ));
    }
    if edges.len() == 2 {
        let (a, b) = (edges[0], edges[1]);
        if b <= a {
            return Some("Give the lower frequency first.".to_string());
        }
        if b / a < BAND_MIN_RATIO {
            return Some(format!("The two frequencies must be at least {BAND_MIN_RATIO} times apart, or the bands will not add back evenly."));
        }
    }
    None
}

/// The Precision Filter slots of band `k` (0 is the lowest): a pair of filters
/// at each edge it touches.
pub fn band_slots(k: usize, edges: &[f64], new_id: &mut dyn FnMut() -> String) -> Vec<Value> {
    let mut slot = |mode: u32, freq: f64| {
        json!({
            "id": new_id(),
            "effect": "altaudio.precision_filter",
            "enabled": true,
            // Butterworth, 2nd order. mode 0 is low-pass, 1 is high-pass.
            "params": { "design": 0, "mode": mode, "order": 0, "freq": freq },
            "preset": "Frequency band",
        })
    };
    let mut out = Vec::new();
    if k > 0 {
        out.push(slot(1, edges[k - 1]));
        out.push(slot(1, edges[k - 1]));
    }
    if k < edges.len() {
        out.push(slot(0, edges[k]));
        out.push(slot(0, edges[k]));
    }
    out
}

/// A clip found on a track: the track, the clip and where the track sits.
#[derive(Clone, Debug)]
pub struct Located {
    pub track: Value,
    pub clip: Value,
    pub track_index: usize,
}

/// Clips that can be split: not reversed (a reversed clip plays from a
/// separate copy) and not locked.
pub fn band_splittable(found: &[Located]) -> Vec<&Located> {
    found
        .iter()
        .filter(|f| {
            !f.clip.get("reversed").is_some_and(truthy) && !f.clip.get("locked").is_some_and(truthy)
        })
        .collect()
}

fn js_text(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => {
            crate::js_compat::js_number_to_string(n.as_f64().unwrap_or(f64::NAN))
        }
        Some(Value::Null) => "null".to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => "undefined".to_string(),
    }
}

/// Copies the listed keys of `from` that are present into `into`.
fn copy_keys(into: &mut Map<String, Value>, from: &Value, keys: &[&str]) {
    for k in keys {
        if let Some(v) = from.get(*k) {
            into.insert((*k).to_string(), v.clone());
        }
    }
}

/// The edit that puts each selected clip's bands on new tracks right under its
/// track, or None when no clip can be split. The originals stay, muted.
pub fn band_split_op(
    found: &[Located],
    edges: &[f64],
    uuid: &mut dyn FnMut() -> String,
) -> Option<Value> {
    let usable = band_splittable(found);
    if usable.is_empty() || check_edges(edges).is_some() {
        return None;
    }
    let labels: &[&str] = if edges.len() == 1 {
        &["Low", "High"]
    } else {
        &["Low", "Mid", "High"]
    };

    let mut by_track: Vec<(usize, Vec<&Located>)> = Vec::new();
    for f in usable {
        match by_track.iter_mut().find(|(i, _)| *i == f.track_index) {
            Some((_, v)) => v.push(f),
            None => by_track.push((f.track_index, vec![f])),
        }
    }
    by_track.sort_by_key(|(i, _)| *i);

    let mut ops: Vec<Value> = Vec::new();
    let mut added = 0usize;
    for (ti, clips) in by_track {
        let source = &clips[0].track;
        let track_ids: Vec<String> = labels.iter().map(|_| uuid()).collect();
        for (k, label) in labels.iter().enumerate() {
            ops.push(json!({
                "op": "add_track",
                "id": track_ids[k],
                "name": format!("{} {}", js_text(source.get("name")), label),
                "stereo": true,
                "index": ti + added + 1 + k,
            }));
        }
        added += labels.len();

        let mut specs: Vec<Vec<Value>> = labels.iter().map(|_| Vec::new()).collect();
        let mut changes: Vec<Value> = Vec::new();
        for f in &clips {
            let clip = &f.clip;
            for (k, label) in labels.iter().enumerate() {
                let id = uuid();
                let mut spec = Map::new();
                spec.insert("id".into(), json!(id));
                spec.insert(
                    "name".into(),
                    json!(format!("{} {}", js_text(clip.get("name")), label)),
                );
                copy_keys(
                    &mut spec,
                    clip,
                    &["media_id", "start", "source_in", "source_out"],
                );
                specs[k].push(Value::Object(spec));
                let mut change = Map::new();
                change.insert("id".into(), json!(id));
                copy_keys(&mut change, clip, &["gain_db", "fade_in", "fade_out"]);
                if clip.get("muted").is_some_and(truthy) {
                    change.insert("muted".into(), json!(true));
                }
                for key in ["audio_type", "channel_mix", "stretch", "looped"] {
                    if let Some(v) = clip.get(key).filter(|v| truthy(v)) {
                        change.insert(key.into(), v.clone());
                    }
                }
                changes.push(Value::Object(change));
            }
        }
        for (k, list) in specs.into_iter().enumerate() {
            if !list.is_empty() {
                ops.push(json!({ "op": "add_clips", "track_id": track_ids[k], "clips": list }));
            }
        }
        if !changes.is_empty() {
            ops.push(json!({ "op": "set_clips", "changes": changes }));
        }

        for (k, track_id) in track_ids.iter().enumerate() {
            if let Some(color) = source.get("color").filter(|c| truthy(c)) {
                ops.push(
                    json!({ "op": "set_track", "id": track_ids[k], "changes": { "color": color } }),
                );
            }
            let slots = band_slots(k, edges, uuid);
            ops.push(json!({
                "op": "set_effects",
                "target": { "kind": "track", "id": track_id },
                "chain": { "enabled": true, "input_gain_db": 0, "output_gain_db": 0, "mix": 1, "slots": slots },
            }));
        }

        let mute: Vec<Value> = clips.iter().map(|f| json!({ "id": f.clip.get("id").cloned().unwrap_or(Value::Null), "muted": true })).collect();
        ops.push(json!({ "op": "set_clips", "changes": mute }));
    }
    Some(if ops.len() == 1 {
        ops.remove(0)
    } else {
        json!({ "op": "batch", "ops": ops })
    })
}
