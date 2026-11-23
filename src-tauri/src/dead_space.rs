// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Close Dead Space, in Rust: after QuickPass has taken the silence out of each
//! track, the tracks can still share stretches where nobody is talking. This
//! finds the stretches where no track has a clip and closes them down to a short
//! pause, moving everything after them left together so the tracks stay in step.
//! Mirrors `src/panels/quickpass/dead-space.ts`, tested against
//! `shared-fixtures/dead-split.json`. Values are session samples. Only the empty
//! time between clips is touched, so no clip is cut.

use crate::clip_edit::{end_of, LockedError};
use serde_json::{json, Value};
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

/// A stretch where no track has a clip.
#[derive(Debug, Clone, PartialEq)]
pub struct DeadSpace {
    /// Where the empty time starts (the end of the clip before it).
    pub start: f64,
    /// Where it ends (the start of the next clip).
    pub end: f64,
    /// How much of it is taken out. The pause that stays is `end - start - remove`.
    pub remove: f64,
}

impl DeadSpace {
    pub fn to_json(&self) -> Value {
        json!({ "start": self.start, "end": self.end, "remove": self.remove })
    }
}

/// The stretches between the first and last clip where none of `track_ids` has
/// a clip, in timeline order. `pause` is the pause left where a stretch is
/// closed, `min_remove` the least a stretch must give back. Muted clips and
/// tracks still count as occupying time, so nothing is ever moved over them.
pub fn dead_spaces(
    doc: &Value,
    track_ids: &[String],
    pause: f64,
    min_remove: f64,
) -> Vec<DeadSpace> {
    let which: HashSet<&str> = track_ids.iter().map(String::as_str).collect();
    let mut spans: Vec<(f64, f64)> = vec![];
    for t in arr(doc, "tracks") {
        if !which.contains(text(t, "id")) {
            continue;
        }
        for c in arr(t, "clips") {
            spans.push((num(c, "start"), end_of(doc, c)));
        }
    }
    spans.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap()
            .then(a.1.partial_cmp(&b.1).unwrap())
    });
    let mut out = vec![];
    let mut reach = -1.0_f64;
    for (s, e) in spans {
        if reach >= 0.0 && s > reach {
            let remove = s - reach - pause.max(0.0);
            if remove > 0.0 && remove >= min_remove.max(1.0) {
                out.push(DeadSpace {
                    start: reach,
                    end: s,
                    remove,
                });
            }
        }
        reach = reach.max(e);
    }
    out
}

/// What closing the dead space does.
#[derive(Debug, PartialEq)]
pub enum Closed {
    /// A track has MIDI clips, which a ripple does not move yet.
    Blocked,
    Done {
        op: Value,
        spaces: Vec<DeadSpace>,
        removed: f64,
        moved: usize,
    },
}

impl Closed {
    pub fn to_json(&self) -> Value {
        match self {
            Closed::Blocked => json!("blocked"),
            Closed::Done {
                op,
                spaces,
                removed,
                moved,
            } => {
                json!({ "op": op, "spaces": spaces.iter().map(DeadSpace::to_json).collect::<Vec<_>>(), "removed": removed, "moved": moved })
            }
        }
    }
}

/// Closes the dead space on `track_ids` and moves their clips, and the markers
/// after it, left. None when there is nothing to close; Blocked when a track
/// has MIDI clips, which a ripple does not move yet (so the track would drift
/// out of step). A LockedError when a clip that has to move is locked.
pub fn close_dead_space(
    doc: &Value,
    track_ids: &[String],
    pause: f64,
    min_remove: f64,
) -> Result<Option<Closed>, LockedError> {
    let which: HashSet<&str> = track_ids.iter().map(String::as_str).collect();
    if arr(doc, "tracks")
        .iter()
        .any(|t| which.contains(text(t, "id")) && !arr(t, "midi_clips").is_empty())
    {
        return Ok(Some(Closed::Blocked));
    }
    let spaces = dead_spaces(doc, track_ids, pause, min_remove);
    if spaces.is_empty() {
        return Ok(None);
    }
    // Each stretch gives back `remove` samples from the point where the kept pause ends.
    let cuts: Vec<(f64, f64)> = spaces
        .iter()
        .map(|s| (s.end - s.remove, s.remove))
        .collect();
    let shift = |p: f64| {
        cuts.iter()
            .fold(0.0, |n, c| n + (p - c.0).max(0.0).min(c.1))
    };
    let mut changes = vec![];
    for t in arr(doc, "tracks") {
        if !which.contains(text(t, "id")) {
            continue;
        }
        for c in arr(t, "clips") {
            let by = shift(num(c, "start"));
            if by == 0.0 {
                continue;
            }
            if crate::mix_routing::truthy(&c["locked"]) {
                return Err(LockedError);
            }
            changes.push(json!({ "id": c["id"], "start": num(c, "start") - by }));
        }
    }
    let mut marks = vec![];
    for m in arr(doc, "markers") {
        let by = shift(num(m, "position"));
        if by > 0.0 {
            marks.push(json!({ "id": m["id"], "position": num(m, "position") - by }));
        }
    }
    let moved = changes.len();
    let mut ops = vec![];
    if !changes.is_empty() {
        ops.push(json!({ "op": "set_clips", "changes": changes }));
    }
    if !marks.is_empty() {
        ops.push(json!({ "op": "set_markers", "changes": marks }));
    }
    if ops.is_empty() {
        return Ok(None);
    }
    let op = if ops.len() == 1 {
        ops.remove(0)
    } else {
        json!({ "op": "batch", "ops": ops })
    };
    Ok(Some(Closed::Done {
        op,
        spaces,
        removed: cuts.iter().fold(0.0, |n, c| n + c.1),
        moved,
    }))
}
