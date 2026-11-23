// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Editor's rows, in Rust: audio tracks with bus tracks placed among them.
//! A bus stores how many tracks sit above it (`position`), so moving a track
//! never drags a bus along with it, and a bus can sit anywhere. Mirrors
//! `src/panels/editor/rows.ts`, tested against `shared-fixtures/takes-rows.json`.

use crate::timeline::default_track_color;
use serde_json::{json, Value};
use std::collections::HashSet;

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}

/// One row: a track (with its index among the listed tracks, or -1) or a bus.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub is_track: bool,
    pub id: String,
    pub i: i64,
    pub listed: bool,
    pub value: Value,
}

impl Row {
    pub fn to_json(&self) -> Value {
        if self.is_track {
            json!({ "kind": "track", "id": self.id, "i": self.i })
        } else {
            json!({ "kind": "bus", "id": self.id })
        }
    }
}

/// Where a bus sits: below this many tracks of the full list.
pub fn bus_slot(b: &Value, tracks: usize) -> f64 {
    let p = b
        .get("position")
        .and_then(Value::as_f64)
        .unwrap_or(tracks as f64);
    p.min(tracks as f64).max(0.0)
}

fn all_rows(doc: &Value, listed: &HashSet<String>) -> Vec<Row> {
    let tracks = arr(doc, "tracks");
    let buses = arr(doc, "buses");
    let mut out = vec![];
    let mut i = 0;
    for k in 0..=tracks.len() {
        for b in buses {
            if bus_slot(b, tracks.len()) == k as f64 {
                out.push(Row {
                    is_track: false,
                    id: text(b, "id").to_string(),
                    i: 0,
                    listed: true,
                    value: b.clone(),
                });
            }
        }
        let Some(t) = tracks.get(k) else { continue };
        let shown = listed.contains(text(t, "id"));
        out.push(Row {
            is_track: true,
            id: text(t, "id").to_string(),
            i: if shown { i } else { -1 },
            listed: shown,
            value: t.clone(),
        });
        if shown {
            i += 1;
        }
    }
    out
}

/// The rows the Editor shows, in order. `listed` holds the ids of the tracks it lists.
pub fn lane_rows(doc: &Value, listed: &HashSet<String>) -> Vec<Row> {
    all_rows(doc, listed)
        .into_iter()
        .filter(|r| r.listed)
        .collect()
}

/// The edits that move row `id` to just above the shown row `before_id` (or to
/// the bottom when None), as one undoable step: the track or bus order, each
/// bus's place among the tracks, and, when tracks move, the colors of tracks
/// still on the default color (which follows the row), so every track keeps the
/// color it had. Empty when nothing moves.
pub fn move_row_ops(
    doc: &Value,
    listed: &HashSet<String>,
    id: &str,
    before_id: Option<&str>,
) -> Vec<Value> {
    let rows = all_rows(doc, listed);
    let Some(from) = rows.iter().position(|r| r.id == id) else {
        return vec![];
    };
    let moving = rows[from].clone();
    let rest: Vec<Row> = rows.iter().filter(|r| r.id != id).cloned().collect();
    let at = match before_id {
        None => rest.len(),
        Some(b) => rest.iter().position(|r| r.id == b).unwrap_or(rest.len()),
    };
    let mut next: Vec<Row> = rest[..at].to_vec();
    next.push(moving.clone());
    next.extend_from_slice(&rest[at..]);
    if next.iter().zip(&rows).all(|(n, r)| n.id == r.id) {
        return vec![];
    }
    let mut ops = vec![];
    if moving.is_track {
        for r in &rows {
            let color_unset = r
                .value
                .get("color")
                .is_none_or(|c| !crate::mix_routing::truthy(c));
            if r.is_track && r.listed && color_unset {
                ops.push(json!({ "op": "set_track", "id": r.id, "changes": { "color": default_track_color(r.i) } }));
            }
        }
        let to = next
            .iter()
            .filter(|r| r.is_track)
            .position(|r| r.id == id)
            .unwrap_or(0);
        ops.push(json!({ "op": "move_track", "id": id, "to": to }));
    } else {
        let to = next
            .iter()
            .filter(|r| !r.is_track)
            .position(|r| r.id == id)
            .map_or(-1, |p| p as i64);
        let was = arr(doc, "buses")
            .iter()
            .position(|b| text(b, "id") == id)
            .map_or(-1, |p| p as i64);
        if to != was {
            ops.push(json!({ "op": "move_bus", "id": id, "to": to }));
        }
    }
    let total = arr(doc, "tracks").len();
    let mut above = 0usize;
    for r in &next {
        if r.is_track {
            above += 1;
            continue;
        }
        let stored = r
            .value
            .get("position")
            .and_then(Value::as_f64)
            .filter(|p| *p < total as f64);
        let want = if above >= total {
            None
        } else {
            Some(above as f64)
        };
        if stored != want {
            let changes = match want {
                None => json!({ "clear_position": true }),
                Some(w) => json!({ "position": w }),
            };
            ops.push(json!({ "op": "set_bus", "id": r.id, "changes": changes }));
        }
    }
    ops
}
