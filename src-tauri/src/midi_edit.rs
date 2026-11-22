// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! MIDI clips and notes: beats and samples, snapping, and the edits the lanes and the piano
//! roll make. The Rust copy of `src/panels/editor/midi.ts` (without the two helpers that read
//! a whole session), tested against `shared-fixtures/keys-tempo-midi.json`. Clips and notes
//! are placed in beats, so they follow the session tempo. Numbers are floats, as in the
//! interface.

use crate::js_compat::js_round;
use crate::tempo_map::{
    bar_at, bar_beats, bar_start, beats_to_seconds, seconds_to_beats, segment_at_beat, Tempo,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub pitch: f64,
    pub velocity: f64,
    pub start: f64,
    pub length: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub name: String,
    pub start: f64,
    pub length: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gain_db: Option<f64>,
    pub notes: Vec<Note>,
}

/// Beats to session samples, through the tempo map.
pub fn beats_to_samples(beats: f64, t: &Tempo, rate: f64) -> f64 {
    js_round(beats_to_seconds(t, beats) * rate)
}

/// Session samples to beats, through the tempo map.
pub fn samples_to_beats(samples: f64, t: &Tempo, rate: f64) -> f64 {
    seconds_to_beats(t, samples / rate.max(1.0))
}

/// Rounds to the nearest multiple of `grid` beats (0 or less leaves it alone).
pub fn snap(beats: f64, grid: f64) -> f64 {
    if grid <= 0.0 {
        return beats;
    }
    js_round(beats / grid) * grid
}

/// Rounds down to a multiple of `grid` beats.
pub fn snap_down(beats: f64, grid: f64) -> f64 {
    if grid <= 0.0 {
        return beats;
    }
    (beats / grid + 1e-9).floor() * grid
}

/// Grid choices for the piano roll, in beats: a label and a size (0 is one bar, -1 is off).
pub const GRIDS: [(&str, f64); 8] = [
    ("1 bar", 0.0),
    ("1/4 (beat)", 1.0),
    ("1/8", 0.5),
    ("1/16", 0.25),
    ("1/32", 0.125),
    ("1/8 triplet", 1.0 / 3.0),
    ("1/16 triplet", 1.0 / 6.0),
    ("Off", -1.0),
];

/// The grid size in beats: "1 bar" is the bar length (where `at` beats falls), "Off" is none.
pub fn grid_beats(grid: f64, t: &Tempo, at: f64) -> f64 {
    if grid == 0.0 {
        let s = segment_at_beat(t, at);
        return bar_beats(s.beats_per_bar, Some(s.beat_unit));
    }
    if grid < 0.0 {
        0.0
    } else {
        grid
    }
}

const NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Note name with octave: 60 is C4 (middle C).
pub fn note_name(pitch: i64) -> String {
    format!(
        "{}{}",
        NAMES[pitch.rem_euclid(12) as usize],
        pitch.div_euclid(12) - 1
    )
}

pub fn is_black_key(pitch: i64) -> bool {
    [1, 3, 6, 8, 10].contains(&pitch.rem_euclid(12))
}

/// A new empty clip one bar long, starting at the bar that holds `at_beats`.
pub fn new_clip(id: &str, at_beats: f64, t: &Tempo, name: &str) -> Clip {
    let bar = bar_at(t, at_beats.max(0.0)).bar;
    let start = bar_start(t, bar);
    Clip {
        id: id.to_string(),
        name: name.to_string(),
        start,
        length: bar_start(t, bar + 1.0) - start,
        muted: None,
        gain_db: None,
        notes: Vec::new(),
    }
}

/// The clip under a timeline position in beats, latest-starting first.
pub fn clip_at_beat(clips: &[Clip], beat: f64) -> Option<&Clip> {
    let mut best: Option<&Clip> = None;
    for c in clips {
        if beat >= c.start && beat < c.start + c.length && best.is_none_or(|b| c.start >= b.start) {
            best = Some(c);
        }
    }
    best
}

fn by_start(a: &Clip, b: &Clip) -> Ordering {
    a.start.partial_cmp(&b.start).unwrap_or(Ordering::Equal)
}

/// Clips in timeline order.
pub fn sorted(clips: &[Clip]) -> Vec<Clip> {
    let mut v = clips.to_vec();
    v.sort_by(by_start);
    v
}

fn least(it: impl Iterator<Item = f64>) -> f64 {
    it.fold(f64::INFINITY, f64::min)
}

/// Moves the clips `ids` by `delta` beats, none before the start.
pub fn move_clips(clips: &[Clip], ids: &BTreeSet<String>, delta: f64) -> Vec<Clip> {
    let first = least(
        clips
            .iter()
            .filter(|c| ids.contains(&c.id))
            .map(|c| c.start),
    );
    let d = (-first).max(delta);
    let moved: Vec<Clip> = clips
        .iter()
        .map(|c| {
            if ids.contains(&c.id) {
                Clip {
                    start: c.start + d,
                    ..c.clone()
                }
            } else {
                c.clone()
            }
        })
        .collect();
    sorted(&moved)
}

/// Moves (or with `copy`, copies under new ids) the clips `ids` from one track's clips to
/// another's, `delta` beats later, none before the start.
pub fn transfer_clips(
    from: &[Clip],
    to: &[Clip],
    ids: &BTreeSet<String>,
    delta: f64,
    copy: bool,
    mut new_id: impl FnMut() -> String,
) -> (Vec<Clip>, Vec<Clip>) {
    let moving: Vec<&Clip> = from.iter().filter(|c| ids.contains(&c.id)).collect();
    if moving.is_empty() {
        return (from.to_vec(), to.to_vec());
    }
    let d = (-least(moving.iter().map(|c| c.start))).max(delta);
    let placed: Vec<Clip> = moving
        .iter()
        .map(|c| Clip {
            id: if copy { new_id() } else { c.id.clone() },
            start: c.start + d,
            ..(*c).clone()
        })
        .collect();
    let rest = if copy {
        from.to_vec()
    } else {
        from.iter()
            .filter(|c| !ids.contains(&c.id))
            .cloned()
            .collect()
    };
    let mut all = to.to_vec();
    all.extend(placed);
    (rest, sorted(&all))
}

/// Sets a clip's length (at least `min` beats). Notes past the end stay, unheard.
pub fn resize_clip(clips: &[Clip], id: &str, length: f64, min: f64) -> Vec<Clip> {
    clips
        .iter()
        .map(|c| {
            if c.id == id {
                Clip {
                    length: length.max(min),
                    ..c.clone()
                }
            } else {
                c.clone()
            }
        })
        .collect()
}

pub fn remove_clips(clips: &[Clip], ids: &BTreeSet<String>) -> Vec<Clip> {
    clips
        .iter()
        .filter(|c| !ids.contains(&c.id))
        .cloned()
        .collect()
}

/// Copies of the clips `ids`, right after the last of them, with new ids. Gives the clips
/// and the ids that were added.
pub fn duplicate_clips(
    clips: &[Clip],
    ids: &BTreeSet<String>,
    mut new_id: impl FnMut() -> String,
) -> (Vec<Clip>, Vec<String>) {
    let picked: Vec<&Clip> = clips.iter().filter(|c| ids.contains(&c.id)).collect();
    if picked.is_empty() {
        return (clips.to_vec(), Vec::new());
    }
    let from = least(picked.iter().map(|c| c.start));
    let to = picked
        .iter()
        .map(|c| c.start + c.length)
        .fold(f64::NEG_INFINITY, f64::max);
    let copies: Vec<Clip> = picked
        .iter()
        .map(|c| Clip {
            id: new_id(),
            start: c.start + (to - from),
            ..(*c).clone()
        })
        .collect();
    let added = copies.iter().map(|c| c.id.clone()).collect();
    let mut all = clips.to_vec();
    all.extend(copies);
    (sorted(&all), added)
}

pub fn replace_clip(clips: &[Clip], clip: &Clip) -> Vec<Clip> {
    clips
        .iter()
        .map(|c| {
            if c.id == clip.id {
                clip.clone()
            } else {
                c.clone()
            }
        })
        .collect()
}

/// Shortest note the piano roll makes, in beats.
pub const MIN_NOTE: f64 = 1.0 / 64.0;

fn clamp_pitch(p: f64) -> f64 {
    127.0_f64.min(0.0_f64.max(js_round(p)))
}

fn note_order(a: &Note, b: &Note) -> Ordering {
    a.start
        .partial_cmp(&b.start)
        .unwrap_or(Ordering::Equal)
        .then(a.pitch.partial_cmp(&b.pitch).unwrap_or(Ordering::Equal))
}

/// Notes in time order, then low to high.
pub fn sort_notes(notes: &[Note]) -> Vec<Note> {
    let mut v = notes.to_vec();
    v.sort_by(note_order);
    v
}

pub fn add_note(notes: &[Note], n: &Note) -> Vec<Note> {
    let mut v = notes.to_vec();
    v.push(Note {
        pitch: clamp_pitch(n.pitch),
        start: n.start.max(0.0),
        length: n.length.max(MIN_NOTE),
        velocity: n.velocity,
    });
    sort_notes(&v)
}

fn picked<'a>(notes: &'a [Note], idx: &BTreeSet<usize>) -> Vec<&'a Note> {
    notes
        .iter()
        .enumerate()
        .filter(|(i, _)| idx.contains(i))
        .map(|(_, n)| n)
        .collect()
}

/// Moves the notes at `indexes` by `beats` and `semitones`, kept inside the clip's start and
/// the pitch range.
pub fn move_notes(notes: &[Note], idx: &BTreeSet<usize>, beats: f64, semitones: f64) -> Vec<Note> {
    let p = picked(notes, idx);
    if p.is_empty() {
        return notes.to_vec();
    }
    let earliest = least(p.iter().map(|n| n.start));
    let low = least(p.iter().map(|n| n.pitch));
    let high = p.iter().map(|n| n.pitch).fold(f64::NEG_INFINITY, f64::max);
    let db = (-earliest).max(beats);
    let ds = (127.0 - high).min((-low).max(js_round(semitones)));
    notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            if idx.contains(&i) {
                Note {
                    start: n.start + db,
                    pitch: n.pitch + ds,
                    ..n.clone()
                }
            } else {
                n.clone()
            }
        })
        .collect()
}

/// Changes the length of the notes at `indexes` by `beats` (each at least MIN_NOTE).
pub fn resize_notes(notes: &[Note], idx: &BTreeSet<usize>, beats: f64) -> Vec<Note> {
    notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            if idx.contains(&i) {
                Note {
                    length: MIN_NOTE.max(n.length + beats),
                    ..n.clone()
                }
            } else {
                n.clone()
            }
        })
        .collect()
}

pub fn remove_notes(notes: &[Note], idx: &BTreeSet<usize>) -> Vec<Note> {
    notes
        .iter()
        .enumerate()
        .filter(|(i, _)| !idx.contains(i))
        .map(|(_, n)| n.clone())
        .collect()
}

pub fn set_velocity(notes: &[Note], idx: &BTreeSet<usize>, velocity: f64) -> Vec<Note> {
    let v = 127.0_f64.min(1.0_f64.max(js_round(velocity)));
    notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            if idx.contains(&i) {
                Note {
                    velocity: v,
                    ..n.clone()
                }
            } else {
                n.clone()
            }
        })
        .collect()
}

/// The lowest and highest pitch in use, or a two-octave range around middle C.
pub fn pitch_range(notes: &[Note]) -> (f64, f64) {
    if notes.is_empty() {
        return (48.0, 72.0);
    }
    (
        least(notes.iter().map(|n| n.pitch)),
        notes
            .iter()
            .map(|n| n.pitch)
            .fold(f64::NEG_INFINITY, f64::max),
    )
}

/// Moves the start of the notes at `indexes` by `beats`, keeping their ends where they are
/// (each stays at least MIN_NOTE long and starts at 0 or later).
pub fn resize_notes_start(notes: &[Note], idx: &BTreeSet<usize>, beats: f64) -> Vec<Note> {
    notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            if !idx.contains(&i) {
                return n.clone();
            }
            let end = n.start + n.length;
            let start = (end - MIN_NOTE).min(0.0_f64.max(n.start + beats));
            Note {
                start,
                length: end - start,
                ..n.clone()
            }
        })
        .collect()
}

/// The notes at `indexes`, moved so the earliest starts at 0: what Copy keeps.
pub fn copy_notes(notes: &[Note], idx: &BTreeSet<usize>) -> Vec<Note> {
    let p = picked(notes, idx);
    let first = least(p.iter().map(|n| n.start));
    p.iter()
        .map(|n| Note {
            start: n.start - first,
            ..(*n).clone()
        })
        .collect()
}

/// How long a copied set of notes runs, in beats.
pub fn span_of(copied: &[Note]) -> f64 {
    copied
        .iter()
        .map(|n| n.start + n.length)
        .fold(0.0, f64::max)
}

/// Pastes `copied` (from `copy_notes`) at `at` beats. Gives the notes and the indexes of the
/// pasted ones.
pub fn paste_notes(notes: &[Note], copied: &[Note], at: f64) -> (Vec<Note>, BTreeSet<usize>) {
    let mut all: Vec<(Note, bool)> = notes.iter().map(|n| (n.clone(), false)).collect();
    all.extend(copied.iter().map(|n| {
        (
            Note {
                start: n.start + at.max(0.0),
                ..n.clone()
            },
            true,
        )
    }));
    all.sort_by(|a, b| note_order(&a.0, &b.0));
    let pasted = all
        .iter()
        .enumerate()
        .filter(|(_, (_, new))| *new)
        .map(|(i, _)| i)
        .collect();
    (all.into_iter().map(|(n, _)| n).collect(), pasted)
}
