// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Where the playhead and the time selection go for the arrow keys, in Rust.
//!
//! These mirror `src/app/playhead-keys.ts` and are tested against
//! `shared-fixtures/playhead-update.json`, which the TypeScript test reads
//! too. All positions are in samples. Typed times in the bars format need the
//! session tempo and stay in TypeScript.

use crate::js_compat::js_round;
use crate::time_input::{parse_time, TimeFormat};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadMove {
    StepLeft,
    StepRight,
    PageLeft,
    PageRight,
    ToIn,
    ToOut,
}

impl HeadMove {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "step-left" => Self::StepLeft,
            "step-right" => Self::StepRight,
            "page-left" => Self::PageLeft,
            "page-right" => Self::PageRight,
            "to-in" => Self::ToIn,
            "to-out" => Self::ToOut,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeMove {
    StartLeft,
    StartRight,
    EndLeft,
    EndRight,
}

impl EdgeMove {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "start-left" => Self::StartLeft,
            "start-right" => Self::StartRight,
            "end-left" => Self::EndLeft,
            "end-right" => Self::EndRight,
            _ => return None,
        })
    }
}

/// What the keys know about the window.
#[derive(Clone, Copy, Debug)]
pub struct HeadContext {
    pub pos: f64,
    pub end: f64,
    /// Samples per pixel.
    pub spp: f64,
    /// Width of the lane in pixels.
    pub width: f64,
    /// The time selection, whose ends are the In and Out points.
    pub sel: Option<(f64, f64)>,
}

fn clamp(n: f64, end: f64) -> f64 {
    js_round(n).min(end.max(0.0)).max(0.0)
}

/// Where the playhead goes for a key, or None when there is nothing to go to.
pub fn head_target(mv: HeadMove, c: &HeadContext) -> Option<f64> {
    let step = js_round(c.spp).max(1.0);
    let page = step.max(js_round(c.width.max(1.0) * c.spp));
    let sel = c.sel.filter(|s| s.1 > s.0);
    match mv {
        HeadMove::StepLeft => Some(clamp(c.pos - step, c.end)),
        HeadMove::StepRight => Some(clamp(c.pos + step, c.end)),
        HeadMove::PageLeft => Some(clamp(c.pos - page, c.end)),
        HeadMove::PageRight => Some(clamp(c.pos + page, c.end)),
        HeadMove::ToIn => sel.map(|s| clamp(s.0, c.end)),
        HeadMove::ToOut => sel.map(|s| clamp(s.1, c.end)),
    }
}

/// Where a typed time sends the playhead, kept inside the content. The bars
/// format is not handled here.
pub fn jump_target(text: &str, rate: f64, format: TimeFormat, fps: f64, end: f64) -> Option<f64> {
    parse_time(text, rate, format, fps).map(|at| clamp(at, end))
}

/// The time selection with one edge moved by `step` samples, never empty.
pub fn adjust_selection(
    sel: Option<(f64, f64)>,
    mv: EdgeMove,
    step: f64,
    end: f64,
) -> Option<(f64, f64)> {
    let (start, stop) = sel?;
    if stop <= start {
        return None;
    }
    let n = js_round(step).max(1.0);
    let top = js_round(end).max(1.0);
    Some(match mv {
        EdgeMove::StartLeft => (clamp(start - n, top), stop),
        EdgeMove::StartRight => (clamp(start + n, top).min(stop - 1.0), stop),
        EdgeMove::EndLeft => (start, clamp(stop - n, top).max(start + 1.0)),
        EdgeMove::EndRight => (start, (start + 1.0).max(clamp(stop + n, top))),
    })
}
