// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Fader travel to dB and back, in Rust.
//!
//! These mirror `src/panels/mixer/fader.ts`. Both are tested against
//! `shared-fixtures/fader.json`, so the two copies cannot drift. When the
//! interface moves to Rust, the TypeScript copy goes away.
//!
//! The top 70% of the travel covers -24 dB to +12 dB, where voice levels
//! live; the bottom 30% covers -24 dB down to silence. That gives fine
//! control where it matters.

use crate::js_compat::{js_round, js_to_fixed1};

pub const FADER_MAX_DB: f64 = 12.0;
pub const FADER_MIN_DB: f64 = -60.0;
const KNEE_DB: f64 = -24.0;
const KNEE_POS: f64 = 0.3;

/// Fader position (0 bottom, 1 top) for a level in dB.
pub fn db_to_fader(db: f64) -> f64 {
    if db <= FADER_MIN_DB {
        0.0
    } else if db >= FADER_MAX_DB {
        1.0
    } else if db >= KNEE_DB {
        KNEE_POS + ((db - KNEE_DB) / (FADER_MAX_DB - KNEE_DB)) * (1.0 - KNEE_POS)
    } else {
        ((db - FADER_MIN_DB) / (KNEE_DB - FADER_MIN_DB)) * KNEE_POS
    }
}

/// Level in dB for a fader position, rounded to 0.1 dB. The bottom means silence (-96 dB).
pub fn fader_to_db(pos: f64) -> f64 {
    let p = pos.clamp(0.0, 1.0);
    if p <= 0.001 {
        return -96.0;
    }
    let db = if p >= KNEE_POS {
        KNEE_DB + ((p - KNEE_POS) / (1.0 - KNEE_POS)) * (FADER_MAX_DB - KNEE_DB)
    } else {
        FADER_MIN_DB + (p / KNEE_POS) * (KNEE_DB - FADER_MIN_DB)
    };
    js_round(db * 10.0) / 10.0
}

pub fn format_db(db: f64) -> String {
    if db <= -95.9 {
        return "-inf".to_string();
    }
    if db > 0.0 {
        format!("+{}", js_to_fixed1(db))
    } else {
        js_to_fixed1(db)
    }
}

pub fn format_pan(p: f64) -> String {
    let r = js_round(p) as i64;
    match r {
        0 => "C".to_string(),
        r if r < 0 => format!("L{}", -r),
        r => format!("R{r}"),
    }
}
