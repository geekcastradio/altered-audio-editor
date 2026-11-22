// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The number rules under clip editing: a clip's length and end on the
//! timeline, stretch limits, snapping, MIDI clip spans and fade shapes.
//! Matches the matching parts of `src/panels/editor/clip-edit.ts` and
//! `src/stores/session.ts`; both are checked against
//! `shared-fixtures/clip-math.json`.

use crate::js_compat::js_round;
use crate::tempo_map::{beats_to_seconds, Tempo};

/// Shortest clip a trim can leave, in session samples (about 10 ms at 48 kHz).
pub const MIN_CLIP: f64 = 480.0;
/// The highest layer a clip can be given.
pub const TOP_LAYER: f64 = 4_294_967_295.0;
pub const STRETCH_MIN: f64 = 0.25;
pub const STRETCH_MAX: f64 = 4.0;

/// How a clip's speed changes: 1 when looped or not stretched.
pub fn stretch_of(looped: bool, ratio: Option<f64>) -> f64 {
    match (looped, ratio) {
        (false, Some(r)) => STRETCH_MIN.max(STRETCH_MAX.min(r)),
        _ => 1.0,
    }
}

/// The parts of a clip its length depends on.
#[derive(Debug, Clone, Copy)]
pub struct ClipSpan {
    pub start: f64,
    pub source_in: f64,
    pub source_out: f64,
    /// Length of the loop, when the clip is looped.
    pub looped: Option<f64>,
    pub stretch: Option<f64>,
}

/// A clip's length in session samples. `media_rate` is the file's rate (0 or None when unknown).
pub fn clip_length(c: &ClipSpan, media_rate: Option<f64>, session_rate: f64) -> f64 {
    if let Some(len) = c.looped {
        return len;
    }
    let src = match media_rate {
        Some(r) if r != 0.0 => r,
        _ => session_rate,
    };
    let natural = js_round(((c.source_out - c.source_in) * session_rate) / src);
    let r = stretch_of(false, c.stretch);
    if r == 1.0 {
        natural
    } else {
        js_round(natural * r)
    }
}

pub fn clip_end(c: &ClipSpan, media_rate: Option<f64>, session_rate: f64) -> f64 {
    c.start + clip_length(c, media_rate, session_rate)
}

/// A MIDI clip's span in session samples. Its start and length are in beats.
pub fn midi_span(tempo: Option<&Tempo>, session_rate: f64, start: f64, length: f64) -> (f64, f64) {
    let default = Tempo {
        bpm: 120.0,
        beats_per_bar: 4.0,
        beat_unit: None,
        changes: None,
    };
    let t = tempo.unwrap_or(&default);
    let at = |beats: f64| js_round(beats_to_seconds(t, beats) * session_rate);
    (at(start), at(start + length))
}

/// The target nearest `value` within `tolerance`, or None.
pub fn nearest(value: f64, targets: &[f64], tolerance: f64) -> Option<f64> {
    let mut best: Option<f64> = None;
    for &t in targets {
        let d = (t - value).abs();
        if d <= tolerance && best.is_none_or(|b| d < (b - value).abs()) {
            best = Some(t);
        }
    }
    best
}

/// Where a position lands when it snaps to a marker (its start, or its end for a range).
pub fn snap_to_markers(position: f64, markers: &[(f64, f64)], tolerance: f64) -> f64 {
    let mut targets = Vec::new();
    for &(p, d) in markers {
        targets.push(p);
        if d > 0.0 {
            targets.push(p + d);
        }
    }
    nearest(position, &targets, tolerance).unwrap_or(position)
}

/// The fade shapes a clip can have, with the names the menu shows.
pub const FADE_SHAPES: [(&str, &str); 5] = [
    ("cosine", "S Curve"),
    ("linear", "Linear"),
    ("logarithmic", "Logarithmic"),
    ("exponential", "Exponential"),
    ("equal_power", "Equal Power"),
];

/// A fade: its length and shape.
#[derive(Debug, Clone, PartialEq)]
pub struct Fade {
    pub length: f64,
    pub shape: Option<String>,
}

/// Which fades of a clip take the new shape. Clips with no fades, or already that shape, give nothing.
pub fn fade_shape_change(fade_in: &Fade, fade_out: &Fade, shape: &str) -> (bool, bool) {
    let differs = |f: &Fade| f.length > 0.0 && f.shape.as_deref() != Some(shape);
    (differs(fade_in), differs(fade_out))
}
