// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Clip > Phase Align, interface side: the start a moved clip gets and the
//! notice line. The Rust copy of `alignOps` (its start rule) and
//! `describeAlign` in `src/panels/editor/align.ts`; both are tested against
//! `shared-fixtures/align-graph.json`. Finding the clip pair and building the
//! edit read the session and stay in TypeScript.

use crate::js_compat::{js_round, js_to_fixed};

/// What the core measured for a pair of clips.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alignment {
    /// Samples the moving clip trails the reference by (negative: early).
    pub lag: f64,
    /// 0 to 1.
    pub score: f64,
    pub inverted: bool,
}

/// Search either way, in ms.
pub const ALIGN_MAX_MS: f64 = 100.0;

/// Where the moving clip starts after the alignment.
pub fn aligned_start(clip_start: f64, lag: f64) -> f64 {
    (clip_start - lag).max(0.0)
}

/// One line for the notice.
pub fn describe_align(a: Alignment, rate: f64, name: &str, track_name: &str) -> String {
    let ms = (a.lag / rate).abs() * 1000.0;
    let mv = if a.lag == 0.0 {
        format!("{name} was already in line")
    } else {
        format!(
            "Moved {name} {} ms {}",
            js_to_fixed(ms, 2),
            if a.lag > 0.0 { "earlier" } else { "later" }
        )
    };
    let flip = if a.inverted {
        format!(", and flipped the polarity of {track_name}")
    } else {
        String::new()
    };
    let weak = if a.score < 0.3 {
        " The match is weak: these may not be the same sound."
    } else {
        ""
    };
    format!("{mv}{flip} ({}% match).{weak}", js_round(a.score * 100.0))
}
