// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Clip level handle and Time Info view rules. The Rust copy of the maths in
//! `src/panels/editor/clip-gain.ts` and `src/panels/time-info/time-info.ts`
//! (not the canvas drawing); all are tested against
//! `shared-fixtures/clip-gain.json`.

use crate::js_compat::{js_round, js_to_fixed1};

/// Lowest and highest clip level, dB (the core clamps to the same).
pub const GAIN_MIN: f64 = -96.0;
pub const GAIN_MAX: f64 = 24.0;

const STEP: f64 = 0.1;
const FINE: f64 = 0.02;

/// The handle's box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GainBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Where the handle sits for a clip drawn from `x0` to `x1` in a lane whose
/// clip body ends at `bottom`. None when the clip is too small to hold it.
pub fn gain_box(x0: f64, x1: f64, bottom: f64, top: f64) -> Option<GainBox> {
    if x1 - x0 < 40.0 || bottom - top < 30.0 {
        return None;
    }
    Some(GainBox {
        x: x0.max(0.0) + 4.0,
        y: bottom - 16.0,
        w: 14.0,
        h: 12.0,
    })
}

/// The level after dragging `dx` right and `dy` down from `start` dB.
pub fn drag_gain(start: f64, dx: f64, dy: f64, fine: bool) -> f64 {
    let v = start + (dx - dy) * if fine { FINE } else { STEP };
    js_round(v.clamp(GAIN_MIN, GAIN_MAX) * 10.0) / 10.0
}

/// "+3.5 dB", "-12.0 dB", or "" at 0.
pub fn gain_text(db: f64) -> String {
    if db == 0.0 {
        return String::new();
    }
    format!("{}{} dB", if db > 0.0 { "+" } else { "" }, js_to_fixed1(db))
}

/// The view: where it starts and how many samples one pixel covers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewDoc {
    pub scroll: f64,
    pub samples_per_pixel: f64,
}

/// The visible range for a view `width` pixels wide.
pub fn view_range(v: ViewDoc, width: f64) -> (f64, f64) {
    (v.scroll, v.scroll + v.samples_per_pixel * width.max(1.0))
}

/// A view showing `start..end` across `width` pixels.
pub fn view_for(start: f64, end: f64, width: f64) -> ViewDoc {
    let s = js_round(start).max(0.0);
    let len = (end - s).max(1.0);
    ViewDoc {
        scroll: s,
        samples_per_pixel: (len / width.max(1.0)).max(1.0 / 64.0),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionPart {
    Start,
    End,
    Duration,
}

/// A selection with one edge or the length changed, kept in order and at or after 0.
pub fn edit_selection(sel: (f64, f64), part: SelectionPart, value: f64) -> (f64, f64) {
    let v = js_round(value).max(0.0);
    if part == SelectionPart::Duration {
        return (sel.0, sel.0 + v);
    }
    let a = if part == SelectionPart::Start {
        v
    } else {
        sel.0
    };
    let b = if part == SelectionPart::End { v } else { sel.1 };
    (a.min(b), a.max(b))
}
