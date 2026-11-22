// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The marks a host drops while recording (Cut this, Retake, Great bit) and
//! moving through them in time order. The Rust copy of
//! `src/panels/review/review.ts`; both are tested against
//! `shared-fixtures/playlist-review.json`.

use crate::js_compat::js_trim;
use crate::playlist::Marker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewKind {
    Cut,
    Retake,
    Great,
}

/// Which review mark a marker is, by its name ("Cut this 3" is a cut), or None for any other marker.
pub fn review_kind(name: &str) -> Option<ReviewKind> {
    let n = js_trim(name).to_lowercase();
    if n.starts_with("cut this") {
        Some(ReviewKind::Cut)
    } else if n.starts_with("retake") {
        Some(ReviewKind::Retake)
    } else if n.starts_with("great bit") {
        Some(ReviewKind::Great)
    } else {
        None
    }
}

/// The review marks that pass the filter (None means all), in time order.
pub fn review_marks(markers: &[Marker], filter: Option<ReviewKind>) -> Vec<&Marker> {
    let mut out: Vec<&Marker> = markers
        .iter()
        .filter(|m| match review_kind(&m.name) {
            Some(k) => filter.is_none() || filter == Some(k),
            None => false,
        })
        .collect();
    out.sort_by(|a, b| a.position.total_cmp(&b.position));
    out
}

/// How many review marks of each kind there are: (cut, retake, great).
pub fn review_counts(markers: &[Marker]) -> (usize, usize, usize) {
    let mut out = (0, 0, 0);
    for m in markers {
        match review_kind(&m.name) {
            Some(ReviewKind::Cut) => out.0 += 1,
            Some(ReviewKind::Retake) => out.1 += 1,
            Some(ReviewKind::Great) => out.2 += 1,
            None => {}
        }
    }
    out
}

/// The index of the mark after (dir 1) or before (dir -1) a spot, skipping any
/// at the spot itself. `marks` must be in time order.
pub fn step_mark(marks: &[&Marker], at: f64, dir: i32) -> Option<usize> {
    if dir == 1 {
        return marks.iter().position(|m| m.position > at);
    }
    (0..marks.len()).rev().find(|&i| marks[i].position < at)
}
