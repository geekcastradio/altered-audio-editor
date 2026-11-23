// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Clip editing on a session, in Rust (first part): where a clip sits, trimming,
//! fades, splitting, looping, cutting out pieces and trimming to a range.
//! Mirrors the matching functions of `src/panels/editor/clip-edit.ts`, tested
//! against `shared-fixtures/clip-edit.json`. A session and its clips are JSON
//! values, as the interface sends them; every function returns new values and
//! leaves its input alone. An edit comes back as an op, or as clip changes
//! (objects holding only the fields that change).

use crate::clip_math::{clip_length, nearest, ClipSpan, MIN_CLIP, STRETCH_MAX, STRETCH_MIN};
use crate::envelope::{slice_volume, Point};
use crate::js_compat::js_round;
use serde_json::{json, Map, Value};
use std::collections::HashSet;

/// A clip that needs a split is locked.
#[derive(Debug, PartialEq)]
pub struct LockedError;

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn truthy(v: &Value, key: &str) -> bool {
    v.get(key).is_some_and(crate::mix_routing::truthy)
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn set(v: &Value, key: &str, val: Value) -> Value {
    let mut m = v.as_object().cloned().unwrap_or_default();
    m.insert(key.to_string(), val);
    Value::Object(m)
}

fn merged(base: &Value, extra: Map<String, Value>) -> Value {
    let mut m = base.as_object().cloned().unwrap_or_default();
    m.extend(extra);
    Value::Object(m)
}

/// The id a clip's envelope uses, as in the core: the clip id for volume, and
/// the clip id with its lowest bit flipped for pan.
pub fn envelope_id(clip_id: &str, kind: &str) -> String {
    if kind == "volume" {
        return clip_id.to_string();
    }
    let mut chars: Vec<char> = clip_id.chars().collect();
    let last = chars.pop().and_then(|c| c.to_digit(16)).unwrap_or(0) ^ 1;
    let mut out: String = chars.into_iter().collect();
    out.push(char::from_digit(last, 16).unwrap_or('0'));
    out
}

/// A copy of a clip with one envelope's points replaced. Empty removes it.
pub fn with_envelope(clip: &Value, kind: &str, points: Vec<Value>) -> Value {
    let mut envs: Vec<Value> = arr(clip, "envelopes")
        .iter()
        .filter(|e| e["target"]["type"] != kind)
        .cloned()
        .collect();
    if !points.is_empty() {
        envs.push(json!({
            "id": envelope_id(text(clip, "id"), kind),
            "target": { "type": kind },
            "enabled": true,
            "points": points,
        }));
    }
    set(clip, "envelopes", Value::Array(envs))
}

fn points_of(clip: &Value, kind: &str) -> Vec<Point> {
    arr(clip, "envelopes")
        .iter()
        .find(|e| e["target"]["type"] == kind && truthy(e, "enabled"))
        .map(|e| to_points(arr(e, "points")))
        .unwrap_or_default()
}

fn to_points(v: &[Value]) -> Vec<Point> {
    v.iter()
        .map(|p| Point {
            position: num(p, "position"),
            value: num(p, "value"),
            curve: text(p, "curve").to_string(),
        })
        .collect()
}

fn from_points(p: &[Point]) -> Vec<Value> {
    p.iter()
        .map(|p| json!({ "position": p.position, "value": p.value, "curve": p.curve }))
        .collect()
}

/// Envelope slices of the piece from `from` to `to`, as clip-change fields.
fn slice_of(clip: &Value, from: f64, to: f64) -> Map<String, Value> {
    let mut out = Map::new();
    for kind in ["volume", "pan"] {
        let pts = points_of(clip, kind);
        if !pts.is_empty() {
            out.insert(
                kind.into(),
                Value::Array(from_points(&slice_volume(&pts, from, to))),
            );
        }
    }
    // Effect settings over time move with the audio too.
    let fx: Vec<Value> = arr(clip, "envelopes")
        .iter()
        .filter(|e| {
            e["target"]["type"] == "effect_param"
                && truthy(e, "enabled")
                && !arr(e, "points").is_empty()
        })
        .map(|e| {
            json!({
                "slot_id": e["target"]["slot_id"],
                "param": e["target"]["param"],
                "points": from_points(&slice_volume(&to_points(arr(e, "points")), from, to)),
            })
        })
        .collect();
    if !fx.is_empty() {
        out.insert("effect_points".into(), Value::Array(fx));
    }
    out
}

/// A new clip with the given envelope slices applied.
fn with_slices(clip: &Value, slices: &Map<String, Value>) -> Value {
    let mut c = clip.clone();
    for kind in ["volume", "pan"] {
        if let Some(Value::Array(p)) = slices.get(kind) {
            c = with_envelope(&c, kind, p.clone());
        }
    }
    for f in slices
        .get("effect_points")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let envs: Vec<Value> = arr(&c, "envelopes")
            .iter()
            .map(|e| {
                let t = &e["target"];
                if t["type"] == "effect_param"
                    && t["slot_id"] == f["slot_id"]
                    && t["param"] == f["param"]
                {
                    set(e, "points", f["points"].clone())
                } else {
                    e.clone()
                }
            })
            .collect();
        c = set(&c, "envelopes", Value::Array(envs));
    }
    c
}

fn modulo(x: f64, r: f64) -> f64 {
    ((x % r) + r) % r
}

fn looped_length(clip: &Value) -> Option<f64> {
    clip.get("looped")
        .filter(|l| !l.is_null())
        .map(|l| num(l, "length"))
}

fn stretch_ratio(clip: &Value) -> Option<f64> {
    clip.get("stretch")
        .filter(|s| !s.is_null())
        .map(|s| num(s, "ratio"))
}

/// How a clip's speed changes: 1 when looped or not stretched.
pub fn stretch_of(clip: &Value) -> f64 {
    match (looped_length(clip), stretch_ratio(clip)) {
        (None, Some(r)) => STRETCH_MIN.max(STRETCH_MAX.min(r)),
        _ => 1.0,
    }
}

fn span_of(clip: &Value) -> ClipSpan {
    ClipSpan {
        start: num(clip, "start"),
        source_in: num(clip, "source_in"),
        source_out: num(clip, "source_out"),
        looped: looped_length(clip),
        stretch: stretch_ratio(clip),
    }
}

/// The media entry with this id.
fn media_by_id<'a>(doc: &'a Value, id: &str) -> Option<&'a Value> {
    arr(doc, "media").iter().find(|m| text(m, "id") == id)
}

fn media_rate(m: Option<&Value>) -> Option<f64> {
    m.and_then(|m| m.get("sample_rate")).and_then(Value::as_f64)
}

/// A clip's length in session samples.
pub fn length_of(clip: &Value, m: Option<&Value>, rate: f64) -> f64 {
    clip_length(&span_of(clip), media_rate(m), rate)
}

/// Where a clip ends on the timeline.
pub fn end_of(doc: &Value, clip: &Value) -> f64 {
    let m = media_by_id(doc, text(clip, "media_id"));
    num(clip, "start") + length_of(clip, m, num(doc, "sample_rate"))
}

/// Session samples to source samples for a clip's file. A stretched clip
/// covers less source per sample.
fn to_source(samples: f64, m: Option<&Value>, rate: f64, ratio: f64) -> f64 {
    let sr = match media_rate(m) {
        Some(r) if r != 0.0 => r,
        _ => rate,
    };
    js_round(((samples * sr) / rate) / ratio)
}

/// What changes when a clip loses (or, with a negative `d`, gains) `d` session
/// samples on its left.
fn cut_left(clip: &Value, m: Option<&Value>, rate: f64, d: f64) -> Map<String, Value> {
    let len = length_of(clip, m, rate);
    let mut out = Map::new();
    if let Some(l) = clip.get("looped").filter(|l| !l.is_null()) {
        let region = (num(clip, "source_out") - num(clip, "source_in")).max(1.0);
        out.insert(
            "looped".into(),
            json!({ "length": len - d, "phase": modulo(num(l, "phase") + to_source(d, m, rate, 1.0), region) }),
        );
        return out;
    }
    let v = (num(clip, "source_in") + to_source(d, m, rate, stretch_of(clip))).max(0.0);
    out.insert("source_in".into(), json!(v));
    out
}

/// What changes when a clip becomes `new_len` session samples long, keeping its left edge.
fn cut_right(clip: &Value, m: Option<&Value>, rate: f64, new_len: f64) -> Map<String, Value> {
    let mut out = Map::new();
    if let Some(l) = clip.get("looped").filter(|l| !l.is_null()) {
        out.insert("looped".into(), set(l, "length", json!(new_len)));
        return out;
    }
    let mut end = num(clip, "source_in") + to_source(new_len, m, rate, stretch_of(clip));
    if let Some(len) = m.map(|m| num(m, "length")).filter(|l| *l > 0.0) {
        end = end.min(len);
    }
    out.insert("source_out".into(), json!(end));
    out
}

/// A clip found in the session with the track it is on.
pub struct Located {
    pub clip: Value,
    pub track: Value,
    pub track_index: usize,
}

pub fn find_clips(doc: &Value, ids: &[String]) -> Vec<Located> {
    let want: HashSet<&str> = ids.iter().map(String::as_str).collect();
    let mut out = vec![];
    for (i, track) in arr(doc, "tracks").iter().enumerate() {
        for clip in arr(track, "clips") {
            if want.contains(text(clip, "id")) {
                out.push(Located {
                    clip: clip.clone(),
                    track: track.clone(),
                    track_index: i,
                });
            }
        }
    }
    out
}

fn fade_len(clip: &Value, key: &str) -> f64 {
    num(&clip[key], "length")
}

fn with_fade(clip: &Value, key: &str, length: f64) -> Value {
    set(&clip[key], "length", json!(length))
}

/// Moves one edge of a clip by `delta`. The clip start and its source start
/// move together, so the audio stays put in time. Neither edge can pass the
/// ends of the file or make the clip shorter than `MIN_CLIP`. The dragged edge
/// snaps to `targets`.
pub fn trim_clip(
    doc: &Value,
    clip: &Value,
    left: bool,
    delta: f64,
    targets: &[f64],
    tolerance: f64,
) -> Option<Value> {
    if truthy(clip, "locked") {
        return None;
    }
    let rate = num(doc, "sample_rate");
    let m = media_by_id(doc, text(clip, "media_id"));
    let sr = match media_rate(m) {
        Some(r) if r != 0.0 => r,
        _ => rate,
    };
    let len = length_of(clip, m, rate);
    let start = num(clip, "start");
    let end = start + len;
    let mut out = Map::new();
    out.insert("id".into(), clip["id"].clone());
    if left {
        let mut at = start + js_round(delta);
        let snapped = if tolerance > 0.0 {
            nearest(at, targets, tolerance)
        } else {
            None
        };
        if let Some(s) = snapped {
            at = s;
        }
        // How far left the source allows (a looped clip can grow to the session start).
        let earliest = if looped_length(clip).is_some() {
            0.0
        } else {
            (0.0_f64).max(start - ((num(clip, "source_in") * rate) / sr * stretch_of(clip)).floor())
        };
        at = at.max(earliest).min(end - MIN_CLIP);
        let d = at - start;
        if d == 0.0 {
            return None;
        }
        let fade_in = (0.0_f64).max(fade_len(clip, "fade_in").min(end - at));
        out.insert("start".into(), json!(at));
        out.extend(cut_left(clip, m, rate, d));
        if fade_in != fade_len(clip, "fade_in") {
            out.insert("fade_in".into(), with_fade(clip, "fade_in", fade_in));
        }
        out.extend(slice_of(clip, d, len));
        return Some(Value::Object(out));
    }
    let mut at = end + js_round(delta);
    let snapped = if tolerance > 0.0 {
        nearest(at, targets, tolerance)
    } else {
        None
    };
    if let Some(s) = snapped {
        at = s;
    }
    let file_len = m.map(|m| num(m, "length")).filter(|l| *l > 0.0);
    let file_end = match (looped_length(clip), file_len) {
        (None, Some(fl)) => {
            start + (((fl - num(clip, "source_in")) * rate) / sr * stretch_of(clip)).floor()
        }
        _ => f64::INFINITY,
    };
    at = (start + MIN_CLIP).max(at.min(file_end));
    if at == end {
        return None;
    }
    let fade_out = (0.0_f64).max(fade_len(clip, "fade_out").min(at - start));
    out.extend(cut_right(clip, m, rate, at - start));
    if fade_out != fade_len(clip, "fade_out") {
        out.insert("fade_out".into(), with_fade(clip, "fade_out", fade_out));
    }
    Some(Value::Object(out))
}

/// Sets a fade length in samples. A fade can cover the whole clip, but the
/// fade in and fade out together never overlap. A new fade takes
/// `default_shape` (the shape picked in Settings > General).
pub fn fade_clip(
    doc: &Value,
    clip: &Value,
    fade_in: bool,
    length: f64,
    default_shape: &str,
) -> Option<Value> {
    let len = length_of(
        clip,
        media_by_id(doc, text(clip, "media_id")),
        num(doc, "sample_rate"),
    );
    let (key, other_key) = if fade_in {
        ("fade_in", "fade_out")
    } else {
        ("fade_out", "fade_in")
    };
    let other = fade_len(clip, other_key);
    let l = (0.0_f64).max(js_round(length).min(len - other));
    let cur = &clip[key];
    if l == num(cur, "length") {
        return None;
    }
    let shape = if num(cur, "length") == 0.0 && text(cur, "shape") == "linear" {
        default_shape.to_string()
    } else {
        text(cur, "shape").to_string()
    };
    let mut fade = cur.as_object().cloned().unwrap_or_default();
    fade.insert("length".into(), json!(l));
    fade.insert("shape".into(), json!(shape));
    Some(json!({ "id": clip["id"], key: fade }))
}

/// Splits clips at `at`. Each clip that spans `at` keeps its left part and
/// gains a new clip for the right part. The fade in stays on the left, the
/// fade out moves to the right. Locked clips are not split.
pub fn split_clips(
    doc: &Value,
    ids: &[String],
    at: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Option<Value> {
    let rate = num(doc, "sample_rate");
    let mut changes = vec![];
    let mut added = vec![];
    for Located { clip, track, .. } in find_clips(doc, ids) {
        if truthy(&clip, "locked") {
            continue;
        }
        let end = end_of(doc, &clip);
        let start = num(&clip, "start");
        if at <= start || at >= end {
            continue;
        }
        let m = media_by_id(doc, text(&clip, "media_id"));
        let left_len = at - start;
        let right_len = end - at;
        if looped_length(&clip).is_none() {
            let cut = num(&clip, "source_in") + to_source(left_len, m, rate, stretch_of(&clip));
            if cut <= num(&clip, "source_in") || cut >= num(&clip, "source_out") {
                continue;
            }
        }
        let mut change = Map::new();
        change.insert("id".into(), clip["id"].clone());
        change.extend(cut_right(&clip, m, rate, left_len));
        change.insert(
            "fade_in".into(),
            with_fade(&clip, "fade_in", fade_len(&clip, "fade_in").min(left_len)),
        );
        change.insert("fade_out".into(), with_fade(&clip, "fade_out", 0.0));
        change.extend(slice_of(&clip, 0.0, left_len));
        changes.push(Value::Object(change));
        let mut extra = cut_left(&clip, m, rate, left_len);
        extra.insert("id".into(), json!(new_id()));
        extra.insert("start".into(), json!(at));
        extra.insert("fade_in".into(), with_fade(&clip, "fade_in", 0.0));
        extra.insert(
            "fade_out".into(),
            with_fade(
                &clip,
                "fade_out",
                fade_len(&clip, "fade_out").min(right_len),
            ),
        );
        let right = merged(&clip, extra);
        added.push(json!([
            track["id"],
            with_slices(&right, &slice_of(&clip, left_len, left_len + right_len))
        ]));
    }
    if changes.is_empty() {
        return None;
    }
    Some(
        json!({ "op": "batch", "ops": [{ "op": "set_clips", "changes": changes }, { "op": "restore_clips", "clips": added }] }),
    )
}

/// Clip > Loop Clip for the given clips (locked ones are skipped). Turns
/// looping on when any of them is not looped yet. Turning it off plays the
/// region once from where the loop started, so nothing jumps.
pub fn loop_changes(doc: &Value, ids: &[String]) -> (Vec<Value>, bool) {
    let rate = num(doc, "sample_rate");
    // A stretched clip cannot loop: remove the stretch first.
    let found: Vec<Located> = find_clips(doc, ids)
        .into_iter()
        .filter(|f| !truthy(&f.clip, "locked") && !truthy(&f.clip, "stretch"))
        .collect();
    let on = !found.iter().all(|f| looped_length(&f.clip).is_some());
    let mut changes = vec![];
    for Located { clip, .. } in &found {
        let m = media_by_id(doc, text(clip, "media_id"));
        if on {
            if looped_length(clip).is_none() {
                changes.push(json!({ "id": clip["id"], "looped": { "length": length_of(clip, m, rate), "phase": 0 } }));
            }
            continue;
        }
        let l = &clip["looped"];
        let si = num(clip, "source_in") + num(l, "phase");
        let out = si
            + 1.0_f64.max((num(clip, "source_out") - si).min(to_source(
                num(l, "length"),
                m,
                rate,
                1.0,
            )));
        let sr = match media_rate(m) {
            Some(r) if r != 0.0 => r,
            _ => rate,
        };
        let len = js_round(((out - si) * rate) / sr);
        let mut c = Map::new();
        c.insert("id".into(), clip["id"].clone());
        c.insert("unloop".into(), json!(true));
        c.insert("source_in".into(), json!(si));
        c.insert("source_out".into(), json!(out));
        if fade_len(clip, "fade_in") > len {
            c.insert("fade_in".into(), with_fade(clip, "fade_in", len));
        }
        if fade_len(clip, "fade_out") > len {
            c.insert("fade_out".into(), with_fade(clip, "fade_out", len));
        }
        changes.push(Value::Object(c));
    }
    (changes, on)
}

/// The piece of `clip` from `from` to `to` (session samples from its start) as
/// a clip of its own starting at `start`, keeping its place in the source (or
/// loop), its envelopes, and the fades that fall inside.
pub fn piece_of(
    clip: &Value,
    m: Option<&Value>,
    rate: f64,
    from: f64,
    to: f64,
    start: f64,
    id: &str,
) -> Value {
    let len = length_of(clip, m, rate);
    let mut p = merged(
        clip,
        [
            ("id".to_string(), json!(id)),
            ("start".to_string(), json!(start)),
        ]
        .into_iter()
        .collect(),
    );
    if from > 0.0 {
        p = merged(&p, cut_left(clip, m, rate, from));
    }
    if to < len {
        let r = cut_right(&p, m, rate, to - from);
        p = merged(&p, r);
    }
    let fi = if from == 0.0 {
        fade_len(clip, "fade_in").min(to - from)
    } else {
        0.0
    };
    let fo = if to == len {
        fade_len(clip, "fade_out").min(to - from)
    } else {
        0.0
    };
    p = set(&p, "fade_in", with_fade(clip, "fade_in", fi));
    p = set(&p, "fade_out", with_fade(clip, "fade_out", fo));
    // Envelope ids follow the clip id, as in the core.
    let env: Vec<Value> = arr(&p, "envelopes")
        .iter()
        .filter(|e| e["target"]["type"] != "volume" && e["target"]["type"] != "pan")
        .cloned()
        .collect();
    with_slices(
        &set(&p, "envelopes", Value::Array(env)),
        &slice_of(clip, from, to),
    )
}

/// What `isolate` finds: the ops that make the cuts and the ids of every piece inside the range.
pub struct Isolated {
    pub ops: Vec<Value>,
    pub inside: Vec<String>,
}

/// Splits clips on the given tracks so the range a..b is covered by whole
/// pieces. Locked clips that need a split give a LockedError.
pub fn isolate(
    doc: &Value,
    track_ids: &[String],
    a: f64,
    b: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Isolated, LockedError> {
    let rate = num(doc, "sample_rate");
    let which: HashSet<&str> = track_ids.iter().map(String::as_str).collect();
    let mut removed = vec![];
    let mut added = vec![];
    let mut inside = vec![];
    for t in arr(doc, "tracks") {
        if !which.contains(text(t, "id")) {
            continue;
        }
        for c in arr(t, "clips") {
            let end = end_of(doc, c);
            let start = num(c, "start");
            if end <= a || start >= b {
                continue;
            }
            if start >= a && end <= b {
                inside.push(text(c, "id").to_string());
                continue;
            }
            if truthy(c, "locked") {
                return Err(LockedError);
            }
            let m = media_by_id(doc, text(c, "media_id"));
            let len = end - start;
            // Cut points inside the clip, then one piece per stretch between them.
            let mut cuts = vec![0.0];
            cuts.extend(
                [a - start, b - start]
                    .into_iter()
                    .filter(|x| *x > 0.0 && *x < len),
            );
            cuts.push(len);
            removed.push(c["id"].clone());
            for i in 0..cuts.len() - 1 {
                let (from, to) = (cuts[i], cuts[i + 1]);
                let id = if i == 0 {
                    text(c, "id").to_string()
                } else {
                    new_id()
                };
                added.push(json!([
                    t["id"],
                    piece_of(c, m, rate, from, to, start + from, &id)
                ]));
                if start + from >= a && start + to <= b {
                    inside.push(id);
                }
            }
        }
    }
    let ops = if removed.is_empty() {
        vec![]
    } else {
        vec![
            json!({ "op": "remove_clips", "ids": removed }),
            json!({ "op": "restore_clips", "clips": added }),
        ]
    };
    Ok(Isolated { ops, inside })
}

/// Clips under `at`: the given ids if any span it, otherwise every clip that does.
pub fn clips_at(doc: &Value, at: f64, preferred: &HashSet<String>) -> Vec<String> {
    let mut all = vec![];
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            if at > num(c, "start") && at < end_of(doc, c) {
                all.push(text(c, "id").to_string());
            }
        }
    }
    let chosen: Vec<String> = all
        .iter()
        .filter(|id| preferred.contains(*id))
        .cloned()
        .collect();
    if chosen.is_empty() {
        all
    } else {
        chosen
    }
}

/// Trims each clip's start or end to `at`. Clips that do not span `at` are left alone.
pub fn trim_to_playhead(doc: &Value, ids: &[String], at: f64, start_edge: bool) -> Vec<Value> {
    let mut out = vec![];
    for Located { clip, .. } in find_clips(doc, ids) {
        let end = end_of(doc, &clip);
        let start = num(&clip, "start");
        if at <= start || at >= end {
            continue;
        }
        let ch = if start_edge {
            trim_clip(doc, &clip, true, at - start, &[], 0.0)
        } else {
            trim_clip(doc, &clip, false, at - end, &[], 0.0)
        };
        out.extend(ch);
    }
    out
}

/// Trims clips so they sit inside the range a..b: a clip that starts before
/// `a` loses its start, one that ends after `b` loses its end.
pub fn trim_to_range(doc: &Value, ids: &[String], a: f64, b: f64) -> Vec<Value> {
    let mut out = vec![];
    for Located { clip, .. } in find_clips(doc, ids) {
        let end = end_of(doc, &clip);
        let start = num(&clip, "start");
        if end <= a || start >= b {
            continue;
        }
        let mut m = Map::new();
        m.insert("id".into(), clip["id"].clone());
        if start < a {
            if let Some(Value::Object(ch)) = trim_clip(doc, &clip, true, a - start, &[], 0.0) {
                m.extend(ch);
            }
        }
        if end > b {
            if let Some(Value::Object(ch)) = trim_clip(doc, &clip, false, b - end, &[], 0.0) {
                m.extend(ch);
            }
        }
        if m.len() > 1 {
            out.push(Value::Object(m));
        }
    }
    out
}
