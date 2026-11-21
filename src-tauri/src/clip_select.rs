// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Selection rules behind Edit > Select (and the empty-track right-click
//! menu): pure functions over clip start positions. This is the Rust copy of
//! `src/app/clip-select.ts`; both are tested against
//! `shared-fixtures/clip-select.json`.

use std::collections::HashSet;

/// A clip as the selection rules see it: an id and where it starts.
#[derive(Debug, Clone, PartialEq)]
pub struct Positioned {
    pub id: String,
    pub start: f64,
}

/// Where a "clips to ..." selection counts from: the earliest chosen clip, or
/// the playhead when no clip is chosen.
pub fn anchor_start(all: &[Positioned], chosen: &HashSet<String>, playhead: f64) -> f64 {
    let mut first = f64::INFINITY;
    for c in all {
        if chosen.contains(&c.id) {
            first = first.min(c.start);
        }
    }
    if first.is_finite() {
        first
    } else {
        playhead
    }
}

/// Ids of clips that start at or after `from`.
pub fn clips_from(clips: &[Positioned], from: f64) -> Vec<String> {
    clips
        .iter()
        .filter(|c| c.start >= from)
        .map(|c| c.id.clone())
        .collect()
}

/// Ids of clips that start at or before `to`.
pub fn clips_up_to(clips: &[Positioned], to: f64) -> Vec<String> {
    clips
        .iter()
        .filter(|c| c.start <= to)
        .map(|c| c.id.clone())
        .collect()
}

/// The next clip in a track: after the last chosen clip, or the first clip at
/// or after the playhead when none in this track is chosen. Clips that start
/// together keep their order in the track.
pub fn next_clip_id(
    track: &[Positioned],
    chosen: &HashSet<String>,
    playhead: f64,
) -> Option<String> {
    let mut sorted: Vec<&Positioned> = track.iter().collect();
    sorted.sort_by(|a, b| a.start.total_cmp(&b.start));
    let last = sorted.iter().rev().find(|c| chosen.contains(&c.id));
    let next = match last {
        Some(last) => sorted.iter().find(|c| c.start > last.start),
        None => sorted.iter().find(|c| c.start >= playhead),
    };
    next.map(|c| c.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clips(v: &[(&str, f64)]) -> Vec<Positioned> {
        v.iter()
            .map(|(id, start)| Positioned {
                id: (*id).into(),
                start: *start,
            })
            .collect()
    }

    fn set(v: &[&str]) -> HashSet<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn next_skips_clips_that_start_together() {
        let t = clips(&[("x", 10.0), ("y", 10.0), ("w", 4.0)]);
        assert_eq!(next_clip_id(&t, &set(&["x"]), 0.0), None);
        assert_eq!(next_clip_id(&t, &set(&["w"]), 0.0).as_deref(), Some("x"));
    }

    #[test]
    fn nothing_chosen_counts_from_the_playhead() {
        let t = clips(&[("a", 0.0), ("b", 10.0)]);
        assert_eq!(anchor_start(&t, &set(&[]), 7.5), 7.5);
        assert_eq!(next_clip_id(&t, &set(&[]), 5.0).as_deref(), Some("b"));
    }
}
