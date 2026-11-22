// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Pitch editing choices: reading saved choices back, the edit they make, and
//! the report line. The Rust copy of `src/app/pitch-edit.ts`; both are tested
//! against `shared-fixtures/app-rules.json`.

use crate::js_compat::js_round;
use serde_json::Value;

pub const MAX_CENTS: f64 = 700.0;
pub const SCALES: [&str; 5] = [
    "chromatic",
    "major",
    "minor",
    "majorPentatonic",
    "minorPentatonic",
];

#[derive(Debug, Clone, PartialEq)]
pub struct Choices {
    /// "tune" or "bend".
    pub bend: bool,
    pub key: f64,
    pub scale: String,
    pub strength: f64,
    pub speed_ms: f64,
    pub max_cents: f64,
    pub bend_start: f64,
    pub bend_end: f64,
    pub bend_middle: Option<f64>,
}

fn num(v: Option<&Value>, lo: f64, hi: f64, d: f64) -> f64 {
    match v.and_then(Value::as_f64) {
        Some(x) if x.is_finite() => hi.min(lo.max(x)),
        _ => d,
    }
}

/// Reads saved choices back, clamping each and defaulting what is missing.
pub fn sanitize_choices(raw: &Value) -> Choices {
    let o = |k: &str| raw.get(k);
    Choices {
        bend: o("mode").and_then(Value::as_str) == Some("bend"),
        key: js_round(num(o("key"), 0.0, 11.0, 0.0)),
        scale: match o("scale").and_then(Value::as_str) {
            Some(s) if SCALES.contains(&s) => s.to_string(),
            _ => "chromatic".to_string(),
        },
        strength: num(o("strength"), 0.0, 100.0, 100.0),
        speed_ms: num(o("speedMs"), 0.0, 500.0, 40.0),
        max_cents: num(o("maxCents"), 10.0, MAX_CENTS, 200.0),
        bend_start: num(o("bendStart"), -MAX_CENTS, MAX_CENTS, 0.0),
        bend_end: num(o("bendEnd"), -MAX_CENTS, MAX_CENTS, 0.0),
        bend_middle: match o("bendMiddle") {
            None | Some(Value::Null) => None,
            v => Some(num(v, -MAX_CENTS, MAX_CENTS, 0.0)),
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Tune {
        key: f64,
        scale: String,
        strength: f64,
        speed_ms: f64,
        max_cents: f64,
    },
    /// (position 0..1, cents) points.
    Bend { points: Vec<(f64, f64)> },
}

/// The edit the choices make; tune strength becomes 0..1.
pub fn edit_for(c: &Choices) -> Edit {
    if !c.bend {
        return Edit::Tune {
            key: c.key,
            scale: c.scale.clone(),
            strength: c.strength / 100.0,
            speed_ms: c.speed_ms,
            max_cents: c.max_cents,
        };
    }
    let mut points = vec![(0.0, c.bend_start)];
    if let Some(m) = c.bend_middle {
        points.push((0.5, m));
    }
    points.push((1.0, c.bend_end));
    Edit::Bend { points }
}

/// True when applying the choices would change nothing.
pub fn is_no_change(c: &Choices) -> bool {
    if !c.bend {
        return c.strength <= 0.0;
    }
    [c.bend_start, c.bend_end, c.bend_middle.unwrap_or(0.0)]
        .iter()
        .all(|v| v.abs() < 1.0)
}

/// What the analysis found and moved, in plain words.
pub fn describe_report(frames: f64, voiced: f64, moved: f64) -> String {
    let share = if frames > 0.0 {
        js_round((100.0 * voiced) / frames)
    } else {
        0.0
    };
    let share = crate::js_compat::js_number_to_string(share);
    if moved == 0.0 {
        return format!(
            "Clear pitch was found in {share} percent of the audio, but none of it needed to move."
        );
    }
    let pct = crate::js_compat::js_number_to_string(js_round((100.0 * moved) / frames.max(1.0)));
    format!("Clear pitch was found in {share} percent of the audio, and the pitch was moved in {pct} percent of it. The rest was left as it was.")
}
