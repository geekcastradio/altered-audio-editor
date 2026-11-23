// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Split Channels to Tracks, in Rust: a clip of a file with two or more
//! channels becomes one track per channel, each playing only that channel the
//! way the original clip played it (its Source Channels, or the default
//! fold-down). With every new track on, the mix sounds the same as the original
//! did. The LFE track starts muted, because the default fold-down leaves it
//! out. The original clips are muted, not removed. Mirrors
//! `src/panels/properties/split-channels.ts`, tested against
//! `shared-fixtures/dead-split.json`.

use crate::channel_rules::{channel_labels, mix_of, Send};
use crate::clip_edit::Located;
use serde_json::{json, Map, Value};

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn channels_of(doc: &Value, media_id: &str) -> usize {
    arr(doc, "media")
        .iter()
        .rev()
        .find(|m| text(m, "id") == media_id)
        .and_then(|m| m.get("channels"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .max(0.0) as usize
}

/// Clips among `found` whose file has two or more channels.
pub fn splittable<'a>(doc: &Value, found: &'a [Located]) -> Vec<&'a Located> {
    found
        .iter()
        .filter(|f| channels_of(doc, text(&f.clip, "media_id")) >= 2)
        .collect()
}

fn sends_of(own: &Value) -> Option<Vec<Send>> {
    let s = own.get("sends")?.as_array()?;
    Some(
        s.iter()
            .map(|x| Send {
                left: x.get("left").and_then(Value::as_f64).unwrap_or(0.0),
                right: x.get("right").and_then(Value::as_f64).unwrap_or(0.0),
            })
            .collect(),
    )
}

fn truthy_str(v: &Value, key: &str) -> bool {
    v.get(key).is_some_and(crate::mix_routing::truthy)
}

/// The edit that splits `found` clips onto new tracks, or None when none of
/// them has two or more channels. One new track per channel of the widest file
/// on each source track, placed right under it.
pub fn split_channels_op(
    doc: &Value,
    found: &[Located],
    uuid: &mut dyn FnMut() -> String,
) -> Option<Value> {
    let usable = splittable(doc, found);
    if usable.is_empty() {
        return None;
    }
    // Group by source track, in track order.
    let mut order: Vec<usize> = usable.iter().map(|f| f.track_index).collect();
    order.sort_unstable();
    order.dedup();
    let mut ops: Vec<Value> = vec![];
    let mut added = 0usize;
    for ti in order {
        let clips: Vec<&&Located> = usable.iter().filter(|f| f.track_index == ti).collect();
        let source = &clips[0].track;
        let widest = clips
            .iter()
            .map(|f| channels_of(doc, text(&f.clip, "media_id")))
            .max()
            .unwrap_or(0);
        let labels = channel_labels(widest);
        let track_ids: Vec<String> = labels.iter().map(|_| uuid()).collect();
        for (k, label) in labels.iter().enumerate() {
            ops.push(json!({ "op": "add_track", "id": track_ids[k], "name": format!("{} {}", text(source, "name"), label), "stereo": true, "index": ti + added + 1 + k }));
        }
        added += widest;
        let mut specs: Vec<Vec<Value>> = labels.iter().map(|_| vec![]).collect();
        let mut changes = vec![];
        for f in &clips {
            let clip = &f.clip;
            let n = channels_of(doc, text(clip, "media_id"));
            let own = clip
                .get("channel_mix")
                .filter(|m| !m.is_null())
                .and_then(sends_of);
            let mix = mix_of(own.as_deref(), n);
            let names = channel_labels(n);
            for k in 0..n {
                let id = uuid();
                specs[k].push(json!({
                    "id": id,
                    "name": format!("{} {}", text(clip, "name"), names[k]),
                    "media_id": clip["media_id"],
                    "start": clip["start"],
                    "source_in": clip["source_in"],
                    "source_out": clip["source_out"],
                }));
                // Only this channel's amounts; every other channel is silent.
                let only: Vec<Value> = mix
                    .iter()
                    .enumerate()
                    .map(|(j, s)| {
                        if j == k {
                            json!({ "left": s.left, "right": s.right })
                        } else {
                            json!({ "left": 0, "right": 0 })
                        }
                    })
                    .collect();
                let mut change = Map::new();
                change.insert("id".into(), json!(id));
                for key in ["gain_db", "fade_in", "fade_out"] {
                    if let Some(v) = clip.get(key) {
                        change.insert(key.into(), v.clone());
                    }
                }
                change.insert("channel_mix".into(), json!({ "sends": only }));
                if truthy_str(clip, "muted") {
                    change.insert("muted".into(), json!(true));
                }
                if truthy_str(clip, "audio_type") {
                    change.insert("audio_type".into(), clip["audio_type"].clone());
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
        for (k, label) in labels.iter().enumerate() {
            let mut t = Map::new();
            if truthy_str(source, "color") {
                t.insert("color".into(), source["color"].clone());
            }
            if label == "LFE" {
                t.insert("muted".into(), json!(true));
            }
            if !t.is_empty() {
                ops.push(json!({ "op": "set_track", "id": track_ids[k], "changes": t }));
            }
        }
        // The originals stay, muted, so nothing is lost and nothing plays twice.
        let muted: Vec<Value> = clips
            .iter()
            .map(|f| json!({ "id": f.clip["id"], "muted": true }))
            .collect();
        ops.push(json!({ "op": "set_clips", "changes": muted }));
    }
    Some(if ops.len() == 1 {
        ops.remove(0)
    } else {
        json!({ "op": "batch", "ops": ops })
    })
}
