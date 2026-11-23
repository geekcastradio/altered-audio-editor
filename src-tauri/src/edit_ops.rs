// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Clip edit helpers in Rust: silence a time range in selected clips, bring
//! clips to the front or back, pick the two clips Phase Align works on and
//! the edit it makes, find a clip or the MIDI clips among ids, and the edges
//! worth snapping to. Mirrors `silence-span.ts`, `clip-order.ts`, `align.ts`,
//! `clip-lookup.ts` and parts of `clip-edit.ts` (all under
//! `src/panels/editor/`), tested against `shared-fixtures/edit-ops.json`.

use crate::clip_edit::{end_of, length_of, media_by_id, points_of, TOP_LAYER};
use crate::editor_rules::{silence_span, EnvelopePoint};
use crate::js_compat::js_round;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k)
        .and_then(Value::as_array)
        .map_or(&[], |a| a.as_slice())
}

fn num(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(0.0)
}

fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn locked(c: &Value) -> bool {
    c.get("locked") == Some(&json!(true))
}

/// Length of the ramp on each side of the silence, in seconds.
pub const SILENCE_RAMP_SECONDS: f64 = 0.005;

/// The changes that silence the session range `start` to `end` in the given
/// clips, one per clip it touches: a dip in the volume envelope with short
/// ramps in and out.
pub fn silence_changes(doc: &Value, ids: &BTreeSet<String>, start: f64, end: f64) -> Vec<Value> {
    let rate = num(doc, "sample_rate");
    let ramp = js_round(rate * SILENCE_RAMP_SECONDS);
    let mut out = Vec::new();
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            if !ids.contains(text(c, "id")) || locked(c) {
                continue;
            }
            let len = length_of(c, media_by_id(doc, text(c, "media_id")), rate);
            let points: Vec<EnvelopePoint> = points_of(c, "volume")
                .into_iter()
                .map(|p| EnvelopePoint {
                    position: p.position,
                    value: p.value,
                    curve: p.curve,
                })
                .collect();
            let cs = num(c, "start");
            if let Some(next) = silence_span(&points, len, start - cs, end - cs, ramp) {
                let pts: Vec<Value> = next
                    .iter()
                    .map(|p| json!({ "position": p.position, "value": p.value, "curve": p.curve }))
                    .collect();
                out.push(json!({ "id": c["id"], "volume": pts }));
            }
        }
    }
    out
}

/// The changes that put the selected clips at the front ("front") or the back
/// ("back") of their tracks, keeping their order among themselves. Locked
/// clips stay, and so does a clip already there.
pub fn stack_changes(doc: &Value, ids: &BTreeSet<String>, front: bool) -> Vec<Value> {
    let mut out = Vec::new();
    for t in arr(doc, "tracks") {
        let clips = arr(t, "clips");
        let picked: Vec<&Value> = clips
            .iter()
            .filter(|c| ids.contains(text(c, "id")) && !locked(c))
            .collect();
        if picked.is_empty() {
            continue;
        }
        let rest: Vec<&Value> = clips
            .iter()
            .filter(|c| !picked.iter().any(|p| std::ptr::eq(*p, *c)))
            .collect();
        let want: Vec<&Value> = if front {
            rest.iter().chain(picked.iter()).copied().collect()
        } else {
            picked.iter().chain(rest.iter()).copied().collect()
        };
        if want
            .iter()
            .zip(clips.iter())
            .all(|(a, b)| std::ptr::eq(*a, b))
        {
            continue;
        }
        let order: Vec<&Value> = if front {
            picked.clone()
        } else {
            picked.iter().rev().copied().collect()
        };
        for c in order {
            out.push(json!({ "id": c["id"], "layer": if front { TOP_LAYER } else { 0.0 } }));
        }
    }
    out
}

/// The clip with this id and the track it is on, or None when the id is empty
/// or no longer in the session.
pub fn clip_by_id(doc: Option<&Value>, id: Option<&str>) -> Option<(Value, String)> {
    let (doc, id) = (doc?, id.filter(|i| !i.is_empty())?);
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            if text(c, "id") == id {
                return Some((c.clone(), text(t, "id").to_string()));
            }
        }
    }
    None
}

/// The MIDI clips among `ids`.
pub fn find_midi_clips(doc: &Value, ids: &BTreeSet<String>) -> Vec<Value> {
    arr(doc, "tracks")
        .iter()
        .flat_map(|t| arr(t, "midi_clips"))
        .filter(|c| ids.contains(text(c, "id")))
        .cloned()
        .collect()
}

/// Edges worth snapping to: session start, playhead, marker edges, and every
/// other clip's ends.
pub fn snap_targets(
    doc: &Value,
    exclude: &BTreeSet<String>,
    playhead: f64,
    clips: bool,
    use_playhead: bool,
    markers: bool,
) -> Vec<f64> {
    let mut out = if use_playhead {
        vec![0.0, playhead]
    } else {
        vec![0.0]
    };
    if markers {
        for m in arr(doc, "markers") {
            out.push(num(m, "position"));
            out.push(num(m, "position") + num(m, "duration"));
        }
    }
    if !clips {
        return out;
    }
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            if exclude.contains(text(c, "id")) {
                continue;
            }
            out.push(num(c, "start"));
            out.push(end_of(doc, c));
        }
    }
    out
}

/// `[reference, moving]` clip ids and the moving clip's track, or None unless
/// two clips on two tracks are selected.
pub fn align_pair(doc: &Value, selected: &[String]) -> Option<(String, String, String)> {
    if selected.len() != 2 {
        return None;
    }
    let tracks = arr(doc, "tracks");
    let at: Vec<(&String, Option<usize>, Option<&Value>)> = selected
        .iter()
        .map(|id| {
            let t = tracks
                .iter()
                .position(|x| arr(x, "clips").iter().any(|c| text(c, "id") == id));
            let clip = t.and_then(|t| {
                arr(&tracks[t], "clips")
                    .iter()
                    .find(|c| text(c, "id") == id)
            });
            (id, t, clip)
        })
        .collect();
    if at.iter().any(|x| x.1.is_none()) || at[0].1 == at[1].1 {
        return None;
    }
    if at.iter().any(|x| x.2.is_some_and(locked)) {
        return None;
    }
    let (r, m) = if at[0].1 < at[1].1 {
        (&at[0], &at[1])
    } else {
        (&at[1], &at[0])
    };
    Some((
        r.0.clone(),
        m.0.clone(),
        text(&tracks[m.1?], "id").to_string(),
    ))
}

/// The edit that applies an alignment (`lag` samples, `inverted`) to the
/// moving clip, or None when it is already in line.
pub fn align_ops(
    doc: &Value,
    other: &str,
    track_id: &str,
    lag: f64,
    inverted: bool,
) -> Option<Value> {
    let track = arr(doc, "tracks")
        .iter()
        .find(|t| text(t, "id") == track_id)?;
    let clip = arr(track, "clips")
        .iter()
        .find(|c| text(c, "id") == other)?;
    let mut ops = Vec::new();
    let start = 0.0_f64.max(num(clip, "start") - lag);
    if start != num(clip, "start") {
        ops.push(json!({ "op": "set_clips", "changes": [{ "id": other, "start": start }] }));
    }
    if inverted {
        let now = track.get("polarity_inverted") == Some(&json!(true));
        ops.push(
            json!({ "op": "set_track", "id": track_id, "changes": { "polarity_inverted": !now } }),
        );
    }
    match ops.len() {
        0 => None,
        1 => ops.pop(),
        _ => Some(json!({ "op": "batch", "ops": ops })),
    }
}
