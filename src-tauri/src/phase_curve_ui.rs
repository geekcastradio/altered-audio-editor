// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Phase Curve's points and curve for its graph editor, in Rust.
//!
//! This mirrors `src/panels/voice-chain/phase-curve.ts` and is tested against
//! `shared-fixtures/band-phase-fit.json`. The sound itself is made by
//! `dsp/effects/phase_curve.rs`; this is only what the editor draws and the
//! control changes it sends. A slot is the effect slot as JSON.

use crate::js_compat::js_round;
use serde_json::{Map, Value};

pub const PHASE_CURVE: &str = "altaudio.phase_curve";
pub const MAX_POINTS: usize = 8;

pub fn freq_key(i: usize) -> String {
    format!("p{}_freq", i + 1)
}

pub fn phase_key(i: usize) -> String {
    format!("p{}_phase", i + 1)
}

/// Default spots for each point, matching the core's.
const SPREAD: [f64; 8] = [
    20.0, 20_000.0, 100.0, 500.0, 1_000.0, 2_000.0, 5_000.0, 10_000.0,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurvePoint {
    /// Which point (0 to 7): its controls are `p{index + 1}_freq` and `_phase`.
    pub index: usize,
    pub hz: f64,
    pub deg: f64,
}

fn get(slot: &Value, key: &str, def: f64) -> f64 {
    slot.get("params")
        .and_then(|p| p.get(key))
        .and_then(Value::as_f64)
        .unwrap_or(def)
}

pub fn point_count(slot: &Value) -> usize {
    js_round(get(slot, "points", 2.0)).clamp(2.0, MAX_POINTS as f64) as usize
}

pub fn range_of(slot: &Value) -> f64 {
    if js_round(get(slot, "range", 0.0)) == 1.0 {
        360.0
    } else {
        180.0
    }
}

/// The points in use, by frequency.
pub fn curve_points(slot: &Value) -> Vec<CurvePoint> {
    let limit = range_of(slot);
    let mut pts: Vec<CurvePoint> = (0..point_count(slot))
        .map(|i| CurvePoint {
            index: i,
            hz: get(slot, &freq_key(i), SPREAD[i]).max(1.0),
            deg: get(slot, &phase_key(i), 0.0).clamp(-limit, limit),
        })
        .collect();
    pts.sort_by(|a, b| a.hz.total_cmp(&b.hz));
    pts
}

/// The curve's angle at `hz`: straight on a log scale, or a smooth curve
/// through the points.
pub fn curve_at(pts: &[CurvePoint], spline: bool, hz: f64) -> f64 {
    let (Some(first), Some(last)) = (pts.first(), pts.last()) else {
        return 0.0;
    };
    if hz <= first.hz {
        return first.deg;
    }
    if hz >= last.hz {
        return last.deg;
    }
    let mut i = 0;
    for (k, p) in pts.iter().enumerate() {
        if p.hz <= hz {
            i = k;
        }
    }
    i = i.min(pts.len() - 2);
    let x1 = pts[i].hz.ln();
    let x2 = pts[i + 1].hz.ln();
    let span = (x2 - x1).max(1e-12);
    let t = ((hz.max(1.0).ln() - x1) / span).clamp(0.0, 1.0);
    let (y1, y2) = (pts[i].deg, pts[i + 1].deg);
    if !spline || pts.len() < 3 {
        return y1 + (y2 - y1) * t;
    }
    let slope = |k: usize| {
        let a = &pts[k.saturating_sub(1)];
        let b = &pts[(k + 1).min(pts.len() - 1)];
        let dx = b.hz.ln() - a.hz.ln();
        if dx <= 1e-12 {
            0.0
        } else {
            (b.deg - a.deg) / dx
        }
    };
    let m1 = slope(i) * span;
    let m2 = slope(i + 1) * span;
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * y1
        + (t3 - 2.0 * t2 + t) * m1
        + (-2.0 * t3 + 3.0 * t2) * y2
        + (t3 - t2) * m2
}

/// The control changes that add a point (the next unused one) at `hz`, `deg`.
/// None when all eight are in use.
pub fn add_point(slot: &Value, hz: f64, deg: f64) -> Option<Map<String, Value>> {
    let n = point_count(slot);
    if n >= MAX_POINTS {
        return None;
    }
    let mut out = Map::new();
    out.insert("points".into(), (n as f64 + 1.0).into());
    out.insert(freq_key(n), hz.into());
    out.insert(phase_key(n), deg.into());
    Some(out)
}

/// The changes that remove point `index`: the last point in use takes its
/// place. None below three points.
pub fn remove_point(slot: &Value, index: usize) -> Option<Map<String, Value>> {
    let n = point_count(slot);
    if n <= 2 || index >= n {
        return None;
    }
    let last = n - 1;
    let mut out = Map::new();
    out.insert("points".into(), (n as f64 - 1.0).into());
    if index != last {
        out.insert(
            freq_key(index),
            get(slot, &freq_key(last), SPREAD[last]).into(),
        );
        out.insert(phase_key(index), get(slot, &phase_key(last), 0.0).into());
    }
    Some(out)
}

/// Whether a group of controls shows below the graph: unused points stay out
/// of the way.
pub fn group_shown(effect: &str, slot: &Value, group: &str) -> bool {
    if effect != PHASE_CURVE {
        return true;
    }
    match group.strip_prefix("Point ") {
        Some(d) if !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()) => {
            d.parse::<f64>().unwrap_or(f64::INFINITY) <= point_count(slot) as f64
        }
        _ => true,
    }
}
