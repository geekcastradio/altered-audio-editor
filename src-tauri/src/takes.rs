// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Take lanes and comping, in Rust. A set of takes is a group of clips on one
//! track that cover the same passage; each has a take number, and the `active`
//! ones play. Comping splits the takes at the edges of a range and switches
//! which take plays there. Mirrors `src/panels/editor/takes.ts`, tested against
//! `shared-fixtures/takes-rows.json`. Sessions are JSON values; every function
//! returns ops and leaves its input alone.

use crate::clip_edit::{end_of, isolate, LockedError};
use crate::library_view::natural_cmp;
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::HashSet;

fn num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn take(c: &Value) -> Option<&Value> {
    c.get("take").filter(|t| !t.is_null())
}

fn active(c: &Value) -> bool {
    take(c).is_some_and(|t| crate::mix_routing::truthy(&t["active"]))
}

fn by_start(a: &Value, b: &Value) -> Ordering {
    num(a, "start")
        .partial_cmp(&num(b, "start"))
        .unwrap_or(Ordering::Equal)
}

/// True for a take that is not the chosen one: it stays on the track, silent.
pub fn is_off_take(c: &Value) -> bool {
    take(c).is_some() && !active(c)
}

/// Every set of takes in the session, in track order, then by start.
pub fn take_sets(doc: &Value) -> Vec<Value> {
    let mut out: Vec<(f64, Value)> = vec![];
    for t in arr(doc, "tracks") {
        let mut by_set: Vec<(String, Vec<&Value>)> = vec![];
        for c in arr(t, "clips") {
            let Some(tk) = take(c) else { continue };
            let set = text(tk, "set");
            match by_set.iter_mut().find(|s| s.0 == set) {
                Some(s) => s.1.push(c),
                None => by_set.push((set.to_string(), vec![c])),
            }
        }
        for (set, clips) in by_set {
            let mut lanes: Vec<(f64, Vec<&Value>)> = vec![];
            for c in &clips {
                let n = num(take(c).unwrap(), "number");
                match lanes.iter_mut().find(|l| l.0 == n) {
                    Some(l) => l.1.push(c),
                    None => lanes.push((n, vec![c])),
                }
            }
            lanes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
            let start = clips
                .iter()
                .map(|c| num(c, "start"))
                .fold(f64::INFINITY, f64::min);
            let end = clips
                .iter()
                .map(|c| end_of(doc, c))
                .fold(f64::NEG_INFINITY, f64::max);
            let lanes: Vec<Value> = lanes
                .into_iter()
                .map(|(number, mut cs)| {
                    cs.sort_by(|a, b| by_start(a, b));
                    json!({ "number": number, "clips": cs })
                })
                .collect();
            out.push((start, json!({ "set": set, "trackId": t["id"], "start": start, "end": end, "lanes": lanes })));
        }
    }
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    out.into_iter().map(|o| o.1).collect()
}

/// The next free take number in a set.
pub fn next_take_number(doc: &Value, set: &str) -> f64 {
    let mut n = 0.0_f64;
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            if let Some(tk) = take(c).filter(|tk| text(tk, "set") == set) {
                n = n.max(num(tk, "number"));
            }
        }
    }
    n + 1.0
}

/// Turns the chosen clips (all on one track, none already a take) into one set
/// of takes. They are numbered by start time. Each one plays unless it overlaps
/// one that already plays, so clips that do not overlap all stay audible.
/// Returns None when fewer than two clips qualify. Ties on start order by id
/// (natural order, close to the browser's compare for plain ids).
pub fn make_takes_ops(
    doc: &Value,
    ids: &[String],
    new_set: &mut dyn FnMut() -> String,
) -> Option<Value> {
    let want: HashSet<&str> = ids.iter().map(String::as_str).collect();
    for t in arr(doc, "tracks") {
        let mut picked: Vec<&Value> = arr(t, "clips")
            .iter()
            .filter(|c| {
                want.contains(text(c, "id"))
                    && take(c).is_none()
                    && !crate::mix_routing::truthy(&c["locked"])
            })
            .collect();
        picked
            .sort_by(|a, b| by_start(a, b).then_with(|| natural_cmp(text(a, "id"), text(b, "id"))));
        if picked.len() < 2 {
            continue;
        }
        let set = new_set();
        let mut playing: Vec<(f64, f64)> = vec![];
        let mut changes = vec![];
        for (i, c) in picked.iter().enumerate() {
            let a = num(c, "start");
            let b = end_of(doc, c);
            let clash = playing.iter().any(|(x, y)| a < *y && b > *x);
            if !clash {
                playing.push((a, b));
            }
            changes.push(json!({ "id": c["id"], "take": { "set": set, "number": (i + 1) as f64, "active": !clash } }));
        }
        return Some(json!({ "op": "set_clips", "changes": changes }));
    }
    None
}

/// Makes `number` the take that plays over the range a..b. The takes of the set
/// are split at the edges of the range, the pieces of that take inside switch
/// on, and the pieces of the other takes that it covers switch off. The range
/// is cut back to what the chosen take covers, so nothing it lacks goes silent.
pub fn pick_range_ops(
    doc: &Value,
    set: &str,
    number: f64,
    a: f64,
    b: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Vec<Value>, LockedError> {
    let tracks = arr(doc, "tracks");
    let in_set = |c: &&Value| take(c).is_some_and(|t| text(t, "set") == set);
    let Some(track) = tracks
        .iter()
        .find(|t| arr(t, "clips").iter().any(|c| in_set(&c)))
    else {
        return Ok(vec![]);
    };
    let mine: Vec<&Value> = arr(track, "clips")
        .iter()
        .filter(|c| in_set(c) && num(take(c).unwrap(), "number") == number)
        .collect();
    if mine.is_empty() {
        return Ok(vec![]);
    }
    let a = a.max(
        mine.iter()
            .map(|c| num(c, "start"))
            .fold(f64::INFINITY, f64::min),
    );
    let b = b.min(
        mine.iter()
            .map(|c| end_of(doc, c))
            .fold(f64::NEG_INFINITY, f64::max),
    );
    if b <= a {
        return Ok(vec![]);
    }
    let mut ops: Vec<Value> = vec![];
    let mut inside: Vec<(f64, Vec<String>)> = vec![];
    let mut numbers: Vec<f64> = vec![];
    for c in arr(track, "clips").iter().filter(in_set) {
        let n = num(take(c).unwrap(), "number");
        if !numbers.contains(&n) {
            numbers.push(n);
        }
    }
    for n in numbers {
        let only_clips: Vec<Value> = arr(track, "clips")
            .iter()
            .filter(|c| in_set(c) && num(take(c).unwrap(), "number") == n)
            .cloned()
            .collect();
        let only_tracks: Vec<Value> = tracks
            .iter()
            .map(|t| {
                if t["id"] == track["id"] {
                    set_clips(t, only_clips.clone())
                } else {
                    t.clone()
                }
            })
            .collect();
        let mut only = doc.clone();
        only["tracks"] = Value::Array(only_tracks);
        let r = isolate(&only, &[text(track, "id").to_string()], a, b, new_id)?;
        ops.extend(r.ops);
        inside.push((n, r.inside));
    }
    // Where each piece sits after the split: look it up from the ops, or the original clip.
    let mut placed: std::collections::HashMap<String, (f64, f64)> = Default::default();
    for c in arr(track, "clips") {
        placed.insert(text(c, "id").to_string(), (num(c, "start"), end_of(doc, c)));
    }
    for op in &ops {
        if op["op"] != "restore_clips" {
            continue;
        }
        for pair in arr(op, "clips") {
            let c = &pair[1];
            placed.insert(text(c, "id").to_string(), (num(c, "start"), end_of(doc, c)));
        }
    }
    let covered: Vec<(f64, f64)> = inside
        .iter()
        .find(|i| i.0 == number)
        .map(|i| {
            i.1.iter()
                .filter_map(|id| placed.get(id).copied())
                .collect()
        })
        .unwrap_or_default();
    let mut changes = vec![];
    for (n, ids) in &inside {
        for id in ids {
            let Some(span) = placed.get(id) else { continue };
            let on = *n == number || !covered.iter().any(|(x, y)| span.0 < *y && span.1 > *x);
            changes.push(json!({ "id": id, "take": { "set": set, "number": n, "active": on } }));
        }
    }
    if !changes.is_empty() {
        ops.push(json!({ "op": "set_clips", "changes": changes }));
    }
    Ok(ops)
}

fn set_clips(t: &Value, clips: Vec<Value>) -> Value {
    let mut t = t.clone();
    t["clips"] = Value::Array(clips);
    t
}

/// Flatten Takes: drops the silent pieces and turns the rest into ordinary clips.
pub fn flatten_ops(doc: &Value, set: &str) -> Vec<Value> {
    let (mut off, mut keep) = (vec![], vec![]);
    for t in arr(doc, "tracks") {
        for c in arr(t, "clips") {
            if take(c).is_none_or(|tk| text(tk, "set") != set) {
                continue;
            }
            if active(c) {
                keep.push(c["id"].clone());
            } else {
                off.push(c["id"].clone());
            }
        }
    }
    let mut ops = vec![];
    if !off.is_empty() {
        ops.push(json!({ "op": "remove_clips", "ids": off }));
    }
    if !keep.is_empty() {
        let changes: Vec<Value> = keep
            .into_iter()
            .map(|id| json!({ "id": id, "clear_take": true }))
            .collect();
        ops.push(json!({ "op": "set_clips", "changes": changes }));
    }
    ops
}

/// Delete Take: removes every piece of one take.
pub fn delete_take_ops(doc: &Value, set: &str, number: f64) -> Vec<Value> {
    let ids: Vec<Value> = arr(doc, "tracks")
        .iter()
        .flat_map(|t| arr(t, "clips"))
        .filter(|c| take(c).is_some_and(|tk| text(tk, "set") == set && num(tk, "number") == number))
        .map(|c| c["id"].clone())
        .collect();
    if ids.is_empty() {
        vec![]
    } else {
        vec![json!({ "op": "remove_clips", "ids": ids })]
    }
}

/// Stretches of a set where no take plays, as (from, to) pairs.
pub fn silent_gaps(doc: &Value, set: &str) -> Vec<(f64, f64)> {
    let sets = take_sets(doc);
    let Some(s) = sets.iter().find(|x| text(x, "set") == set) else {
        return vec![];
    };
    let mut spans: Vec<(f64, f64)> = arr(s, "lanes")
        .iter()
        .flat_map(|l| arr(l, "clips"))
        .filter(|c| active(c))
        .map(|c| (num(c, "start"), end_of(doc, c)))
        .collect();
    spans.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(Ordering::Equal));
    let end = num(s, "end");
    let mut gaps = vec![];
    let mut at = num(s, "start");
    for (x, y) in spans {
        if x > at {
            gaps.push((at, x));
        }
        at = at.max(y);
    }
    if at < end {
        gaps.push((at, end));
    }
    gaps
}
