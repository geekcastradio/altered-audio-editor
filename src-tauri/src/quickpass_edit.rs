// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! What QuickPass does with what it found, in Rust: removing silences (leaving
//! the gap open or closing it), splitting at both edges, turning stretches down
//! with envelope points, marking them, and keeping the other findings in step
//! after one is removed. Mirrors the edit half of
//! `src/panels/quickpass/quickpass.ts`, tested against
//! `shared-fixtures/quickpass-edit.json`. The scan half is in `quickpass_scan`.

use crate::clip_edit::{
    clear_range, end_of, from_points, points_of, preview_tracks, ripple_delete_range, split_clips,
    LockedError,
};
use crate::envelope::{level_at, Point};
use crate::js_compat::js_round;
use crate::quickpass_scan::Finding;
use serde_json::{json, Value};

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

/// What an edit did to the clips, so several can be built one after another.
pub fn apply_local(doc: &Value, op: &Value) -> Value {
    let mut out = doc.clone();
    match text(op, "op") {
        "batch" => {
            for o in arr(op, "ops") {
                out = apply_local(&out, o);
            }
        }
        "remove_clips" => {
            let gone: std::collections::HashSet<&str> =
                arr(op, "ids").iter().filter_map(Value::as_str).collect();
            let tracks: Vec<Value> = arr(doc, "tracks")
                .iter()
                .map(|t| {
                    let mut t = t.clone();
                    t["clips"] = Value::Array(
                        arr(&t, "clips")
                            .iter()
                            .filter(|c| !gone.contains(text(c, "id")))
                            .cloned()
                            .collect(),
                    );
                    t
                })
                .collect();
            out["tracks"] = Value::Array(tracks);
        }
        "set_clips" => {
            out["tracks"] = Value::Array(preview_tracks(doc, arr(op, "changes")));
        }
        "restore_clips" => {
            let entries = arr(op, "clips");
            let tracks: Vec<Value> = arr(doc, "tracks")
                .iter()
                .map(|t| {
                    let mut clips = arr(t, "clips").to_vec();
                    clips.extend(
                        entries
                            .iter()
                            .filter(|e| e[0] == t["id"])
                            .map(|e| e[1].clone()),
                    );
                    let mut t = t.clone();
                    t["clips"] = Value::Array(clips);
                    t
                })
                .collect();
            out["tracks"] = Value::Array(tracks);
        }
        _ => {}
    }
    out
}

/// The part of a silence that is cut, keeping `keep_ms` of it (half at each
/// edge). None when nothing is left to cut.
pub fn cut_range(start: f64, end: f64, keep_ms: f64, rate: f64) -> Option<(f64, f64)> {
    let half = js_round((keep_ms / 1000.0) * rate * 0.5);
    let (a, b) = (start + half, end - half);
    (b > a).then_some((a, b))
}

/// Removes the silences, leaving the gaps open unless `ripple` closes them.
/// Last one first so earlier ones keep their place.
pub fn remove_op(
    doc: &Value,
    found: &[Finding],
    ripple: &str,
    keep_ms: f64,
    new_id: &mut dyn FnMut() -> String,
) -> Result<Option<Value>, LockedError> {
    let mut ops = vec![];
    let mut cur = doc.clone();
    let mut order: Vec<&Finding> = found.iter().collect();
    order.sort_by(|a, b| {
        b.start
            .partial_cmp(&a.start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for f in order {
        let Some(cut) = cut_range(f.start, f.end, keep_ms, num(doc, "sample_rate")) else {
            continue;
        };
        let tracks: Vec<String> = if ripple == "all" {
            arr(&cur, "tracks")
                .iter()
                .map(|t| text(t, "id").to_string())
                .collect()
        } else {
            vec![f.track_id.clone()]
        };
        // "none" strips the audio and leaves the gap open, so every track stays in step.
        let op = if ripple == "none" {
            clear_range(&cur, &tracks, cut.0, cut.1, new_id)?
        } else {
            ripple_delete_range(&cur, &tracks, cut.0, cut.1, new_id, None)?
        };
        let Some(op) = op else { continue };
        cur = apply_local(&cur, &op);
        ops.push(op);
    }
    Ok(if ops.is_empty() {
        None
    } else {
        Some(json!({ "op": "batch", "ops": ops }))
    })
}

/// Splits each track at both edges of every silence. Locked clips are left alone.
pub fn split_op(
    doc: &Value,
    found: &[Finding],
    new_id: &mut dyn FnMut() -> String,
) -> Option<Value> {
    let mut ops = vec![];
    let mut cur = doc.clone();
    for f in found {
        let Some(t) = arr(&cur, "tracks")
            .iter()
            .find(|x| text(x, "id") == f.track_id)
            .cloned()
        else {
            continue;
        };
        let ids: Vec<String> = arr(&t, "clips")
            .iter()
            .map(|c| text(c, "id").to_string())
            .collect();
        for at in [f.end, f.start] {
            let Some(op) = split_clips(&cur, &ids, at, new_id) else {
                continue;
            };
            cur = apply_local(&cur, &op);
            ops.push(op);
        }
    }
    if ops.is_empty() {
        None
    } else {
        Some(json!({ "op": "batch", "ops": ops }))
    }
}

/// The clips that now sit exactly on the findings, to leave selected after a split.
pub fn clips_on(doc: &Value, found: &[Finding]) -> Vec<String> {
    let mut ids = vec![];
    for f in found {
        let Some(t) = arr(doc, "tracks")
            .iter()
            .find(|x| text(x, "id") == f.track_id)
        else {
            continue;
        };
        for c in arr(t, "clips") {
            if num(c, "start") == f.start && end_of(doc, c) == f.end {
                ids.push(text(c, "id").to_string());
            }
        }
    }
    ids
}

fn numbered(name: &str, base: &str) -> bool {
    name.strip_prefix(base)
        .and_then(|r| r.strip_prefix(' '))
        .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
}

/// One range marker per silence, named `base 1`, `base 2` and on from the ones already there.
pub fn markers_for(
    doc: &Value,
    found: &[Finding],
    new_id: &mut dyn FnMut() -> String,
    base: &str,
) -> Vec<Value> {
    let taken = arr(doc, "markers")
        .iter()
        .filter(|m| numbered(text(m, "name"), base))
        .count();
    let mut order: Vec<&Finding> = found.iter().collect();
    order.sort_by(|a, b| {
        a.start
            .partial_cmp(&b.start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    order
        .iter()
        .enumerate()
        .map(|(i, f)| {
            json!({
                "id": new_id(),
                "name": format!("{} {}", base, taken + i + 1),
                "kind": "cue",
                "position": js_round(f.start).max(0.0),
                "duration": js_round(f.end - f.start).max(0.0),
                "description": "",
                "chapter": null,
                "contracted_duration": null,
            })
        })
        .collect()
}

/// After `gone` is removed, where the other findings sit. Those inside it are dropped.
pub fn shift_findings(
    rest: &[Finding],
    gone: &Finding,
    ripple: &str,
    keep_ms: f64,
    rate: f64,
) -> Vec<Finding> {
    let cut = cut_range(gone.start, gone.end, keep_ms, rate).unwrap_or((gone.start, gone.start));
    let len = cut.1 - cut.0;
    let (g_start, g_end) = cut;
    // Left open: nothing moves, only findings inside the stripped part are gone.
    if ripple == "none" {
        return rest
            .iter()
            .filter(|f| {
                f.id != gone.id
                    && !(f.track_id == gone.track_id && f.start >= g_start && f.end <= g_end)
            })
            .cloned()
            .collect();
    }
    rest.iter()
        .filter(|f| {
            f.id != gone.id
                && !((ripple == "all" || f.track_id == gone.track_id)
                    && f.start < g_end
                    && f.end > g_start)
        })
        .map(|f| {
            if f.start >= g_end && (ripple == "all" || f.track_id == gone.track_id) {
                Finding {
                    start: f.start - len,
                    end: f.end - len,
                    ..f.clone()
                }
            } else {
                f.clone()
            }
        })
        .collect()
}

/// Longest fade into and out of a turned-down stretch, in seconds.
const DUCK_RAMP_SEC: f64 = 0.01;

/// Turns each silence down with volume envelope points instead of cutting it,
/// so clips stay whole and every dip can be adjusted afterward. Points already
/// inside a dip are replaced; the level either side is kept.
pub fn duck_op(doc: &Value, found: &[Finding], duck_db: f64) -> Option<Value> {
    let rate = num(doc, "sample_rate");
    let max_ramp = js_round(DUCK_RAMP_SEC * rate);
    let mut changes = vec![];
    for t in arr(doc, "tracks") {
        let mut mine: Vec<&Finding> = found
            .iter()
            .filter(|f| f.track_id == text(t, "id"))
            .collect();
        if mine.is_empty() {
            continue;
        }
        mine.sort_by(|a, b| {
            a.start
                .partial_cmp(&b.start)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for c in arr(t, "clips") {
            if crate::mix_routing::truthy(&c["locked"]) {
                continue;
            }
            let len = end_of(doc, c) - num(c, "start");
            let original = points_of(c, "volume");
            let mut points: Vec<Point> = original.clone();
            let mut touched = false;
            for f in &mine {
                let a = (f.start - num(c, "start")).max(0.0);
                let b = len.min(f.end - num(c, "start"));
                if b - a < 2.0 {
                    continue;
                }
                let ramp = 1.0_f64.max(max_ramp.min(((b - a) / 4.0).floor()));
                let lead = (a - ramp).max(0.0);
                let tail = len.min(b + ramp);
                let clamp = |v: f64| (-60.0_f64).max(12.0_f64.min(v));
                let before = level_at(&points, lead);
                let after = level_at(&points, tail);
                let at_a = level_at(&points, a);
                let at_b = level_at(&points, b);
                let pt = |position: f64, value: f64| Point {
                    position,
                    value,
                    curve: "linear".to_string(),
                };
                let mut add = vec![];
                if a > 0.0 {
                    add.push(pt(lead, before));
                }
                add.push(pt(a, clamp(at_a + duck_db)));
                add.push(pt(b, clamp(at_b + duck_db)));
                if b < len {
                    add.push(pt(tail, after));
                }
                let lo = if a > 0.0 { lead } else { a };
                let hi = if b < len { tail } else { b };
                points.retain(|p| p.position < lo || p.position > hi);
                points.extend(add);
                points.sort_by(|x, y| {
                    x.position
                        .partial_cmp(&y.position)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                // Two points on one position: keep the later one.
                let n = points.len();
                let snapshot = points.clone();
                points = snapshot
                    .iter()
                    .enumerate()
                    .filter(|(i, p)| *i == n - 1 || snapshot[i + 1].position != p.position)
                    .map(|(_, p)| p.clone())
                    .collect();
                touched = true;
            }
            if touched {
                // A clip with no envelope before needs its ends pinned at 0 dB.
                if original.is_empty() {
                    if points[0].position > 0.0 {
                        points.insert(
                            0,
                            Point {
                                position: 0.0,
                                value: 0.0,
                                curve: "linear".to_string(),
                            },
                        );
                    }
                    if points[points.len() - 1].position < len {
                        points.push(Point {
                            position: len,
                            value: 0.0,
                            curve: "linear".to_string(),
                        });
                    }
                }
                changes.push(json!({ "id": c["id"], "volume": from_points(&points) }));
            }
        }
    }
    if changes.is_empty() {
        None
    } else {
        Some(json!({ "op": "set_clips", "changes": changes }))
    }
}
