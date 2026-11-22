// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Recording options and where a finished take lands. The Rust copy of
//! `src/app/recording.ts`; both are tested against
//! `shared-fixtures/loudness-recording.json`.
//!
//! The offset trims that much from the start of each take so it lines up with
//! what it was played against. A punch-in records a little before the time
//! selection, stops at its end, keeps only the selected range and replaces
//! what was there.

use crate::js_compat::js_round;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecordOptions {
    /// Milliseconds trimmed from the start of every take.
    pub offset_ms: f64,
    /// Record into the time selection when there is one.
    pub punch: bool,
    /// Seconds of playback before a punch-in.
    pub preroll_sec: f64,
    /// A punch-in replaces the audio already in that range on the recorded track.
    pub replace: bool,
}

pub const RECORD_DEFAULTS: RecordOptions = RecordOptions {
    offset_ms: 0.0,
    punch: true,
    preroll_sec: 2.0,
    replace: true,
};

fn num(v: Option<&Value>, lo: f64, hi: f64, d: f64) -> f64 {
    match v.and_then(Value::as_f64) {
        Some(x) if x.is_finite() => hi.min(lo.max(x)),
        _ => d,
    }
}

/// Reads saved options back, clamping each and defaulting what is missing.
pub fn sanitize_record_options(raw: &Value) -> RecordOptions {
    let get = |k: &str| raw.as_object().and_then(|o| o.get(k));
    RecordOptions {
        offset_ms: num(get("offsetMs"), 0.0, 1000.0, RECORD_DEFAULTS.offset_ms),
        punch: get("punch") != Some(&Value::Bool(false)),
        preroll_sec: num(get("prerollSec"), 0.0, 10.0, RECORD_DEFAULTS.preroll_sec),
        replace: get("replace") != Some(&Value::Bool(false)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecordPlan {
    /// Session sample where recording (and playback) starts.
    pub at: f64,
    /// The range a punch-in keeps (start, end), or None for an ordinary take.
    pub punch: Option<(f64, f64)>,
}

/// Where to start, given the playhead and the time selection (start, end).
pub fn record_plan(
    playhead: f64,
    selection: Option<(f64, f64)>,
    o: RecordOptions,
    rate: f64,
) -> RecordPlan {
    if let (true, Some((s, e))) = (o.punch, selection) {
        if e > s {
            return RecordPlan {
                at: 0.0_f64.max(s - js_round(o.preroll_sec * rate)),
                punch: Some((s, e)),
            };
        }
    }
    RecordPlan {
        at: playhead,
        punch: None,
    }
}

/// Seconds of recording after which a punch-in stops by itself.
pub fn punch_stop_after(p: RecordPlan, rate: f64) -> Option<f64> {
    p.punch.map(|(_, end)| (end - p.at) / rate)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub start: f64,
    pub source_in: f64,
    pub source_out: f64,
}

/// Where a take of `length` source frames at `take_rate` lands: its clip start
/// and source range. None when nothing usable is left.
pub fn place_take(
    p: RecordPlan,
    length: f64,
    take_rate: f64,
    session_rate: f64,
    offset_ms: f64,
) -> Option<Placement> {
    let to_src = |s: f64| js_round((s * take_rate) / session_rate);
    let offset = js_round((offset_ms / 1000.0) * take_rate);
    let (mut start, mut source_in, mut source_out) = (p.at, offset, length);
    if let Some((ps, pe)) = p.punch {
        start = ps;
        source_in = offset + to_src(ps - p.at);
        source_out = length.min(source_in + to_src(pe - ps));
    }
    (source_out > source_in).then_some(Placement {
        start,
        source_in,
        source_out,
    })
}
