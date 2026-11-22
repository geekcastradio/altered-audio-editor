// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Voice Chain and clip-splitting math: the EQ graph's width handles, the
//! phase curve, and band split edges. The Rust copy of the pure parts of
//! `src/panels/voice-chain/eq-graph.ts`, `phase-curve.ts` and
//! `src/panels/properties/band-split.ts`; all are tested against
//! `shared-fixtures/panel-rules.json`.

use crate::js_compat::{
    is_js_space, js_number_to_string, js_round, js_string_to_number, js_to_fixed,
};
use std::collections::BTreeMap;

// ---- EQ graph ----

/// A peaking band's width in octaves for a Q.
pub fn octaves_of(q: f64) -> f64 {
    (2.0 / std::f64::consts::LN_2) * (1.0 / (2.0 * q.max(1e-3))).asinh()
}

/// The Q for a width in octaves.
pub fn q_of(octaves: f64) -> f64 {
    let r = 2f64.powf(octaves.max(0.01));
    r.sqrt() / (r - 1.0)
}

/// Where a band's width handles sit: the frequencies half its width either side.
pub fn width_edges(freq: f64, q: f64) -> (f64, f64) {
    let half = 2f64.powf(octaves_of(q) / 2.0);
    (freq / half, freq * half)
}

/// The Q a width handle dragged to `at` Hz gives a band centered on `freq`.
pub fn q_from_edge(freq: f64, at: f64) -> f64 {
    q_of(2.0 * (at.max(1.0) / freq.max(1.0)).log2().abs())
}

/// "1.2 kHz" or "80 Hz".
pub fn format_hz(hz: f64) -> String {
    if hz >= 1000.0 {
        let fixed = js_to_fixed(hz / 1000.0, if hz >= 10_000.0 { 1 } else { 2 });
        let n: f64 = fixed.parse().unwrap();
        format!("{} kHz", js_number_to_string(n))
    } else {
        format!("{} Hz", js_number_to_string(js_round(hz)))
    }
}

// ---- phase curve ----

pub const MAX_POINTS: usize = 8;
const SPREAD: [f64; 8] = [
    20.0, 20_000.0, 100.0, 500.0, 1_000.0, 2_000.0, 5_000.0, 10_000.0,
];

pub fn freq_key(i: usize) -> String {
    format!("p{}_freq", i + 1)
}

pub fn phase_key(i: usize) -> String {
    format!("p{}_phase", i + 1)
}

/// A slot's control values by name.
pub type Params = BTreeMap<String, f64>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurvePoint {
    pub index: usize,
    pub hz: f64,
    pub deg: f64,
}

fn get(p: &Params, key: &str, def: f64) -> f64 {
    p.get(key).copied().unwrap_or(def)
}

pub fn point_count(p: &Params) -> usize {
    js_round(get(p, "points", 2.0)).clamp(2.0, MAX_POINTS as f64) as usize
}

pub fn range_of(p: &Params) -> f64 {
    if js_round(get(p, "range", 0.0)) == 1.0 {
        360.0
    } else {
        180.0
    }
}

/// The points in use, by frequency.
pub fn curve_points(p: &Params) -> Vec<CurvePoint> {
    let limit = range_of(p);
    let mut v: Vec<CurvePoint> = (0..point_count(p))
        .map(|i| CurvePoint {
            index: i,
            hz: get(p, &freq_key(i), SPREAD[i]).max(1.0),
            deg: get(p, &phase_key(i), 0.0).clamp(-limit, limit),
        })
        .collect();
    v.sort_by(|a, b| a.hz.partial_cmp(&b.hz).unwrap());
    v
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
    let t = (((hz.max(1.0)).ln() - x1) / span).clamp(0.0, 1.0);
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
    let (m1, m2) = (slope(i) * span, slope(i + 1) * span);
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * y1
        + (t3 - 2.0 * t2 + t) * m1
        + (-2.0 * t3 + 3.0 * t2) * y2
        + (t3 - t2) * m2
}

/// The control changes that add a point (the next unused one) at `hz`, `deg`.
/// None when all eight are in use.
pub fn add_point(p: &Params, hz: f64, deg: f64) -> Option<Params> {
    let n = point_count(p);
    if n >= MAX_POINTS {
        return None;
    }
    Some(Params::from([
        ("points".to_string(), (n + 1) as f64),
        (freq_key(n), hz),
        (phase_key(n), deg),
    ]))
}

/// The changes that remove point `index`: the last point in use takes its
/// place. None below three points.
pub fn remove_point(p: &Params, index: usize) -> Option<Params> {
    let n = point_count(p);
    if n <= 2 || index >= n {
        return None;
    }
    let last = n - 1;
    let mut out = Params::from([("points".to_string(), (n - 1) as f64)]);
    if index != last {
        out.insert(freq_key(index), get(p, &freq_key(last), SPREAD[last]));
        out.insert(phase_key(index), get(p, &phase_key(last), 0.0));
    }
    Some(out)
}

/// Whether a group of controls shows below the graph: unused points stay out
/// of the way. `is_phase_curve` says the effect is the phase curve.
pub fn group_shown(is_phase_curve: bool, p: &Params, group: &str) -> bool {
    if !is_phase_curve {
        return true;
    }
    match group.strip_prefix("Point ") {
        Some(d) if !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()) => {
            d.parse::<f64>().unwrap() <= point_count(p) as f64
        }
        _ => true,
    }
}

// ---- band split ----

pub const BAND_MIN_HZ: f64 = 40.0;
pub const BAND_MAX_HZ: f64 = 16_000.0;
/// Two edges closer than this ratio make the bands add up unevenly.
pub const BAND_MIN_RATIO: f64 = 6.0;

/// Reads "200, 2000" (commas, spaces or semicolons) into edges in Hz; None
/// when it is not numbers.
pub fn parse_edges(text: &str) -> Option<Vec<f64>> {
    let parts: Vec<&str> = text
        .split(|c: char| is_js_space(c) || c == ',' || c == ';')
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for p in parts {
        let cut = p
            .len()
            .checked_sub(2)
            .filter(|i| p.is_char_boundary(*i))
            .filter(|i| p[*i..].eq_ignore_ascii_case("hz"));
        let n = js_string_to_number(&p[..cut.unwrap_or(p.len())])?;
        if !n.is_finite() {
            return None;
        }
        out.push(n);
    }
    Some(out)
}

/// What is wrong with the edges, or None when they can be used.
pub fn check_edges(edges: &[f64]) -> Option<String> {
    if edges.is_empty() || edges.len() > 2 {
        return Some("Give one frequency for two bands, or two for three bands.".to_string());
    }
    if edges.iter().any(|e| *e < BAND_MIN_HZ || *e > BAND_MAX_HZ) {
        return Some(format!(
            "Each frequency must be between {} and {} Hz.",
            BAND_MIN_HZ, BAND_MAX_HZ
        ));
    }
    if edges.len() == 2 {
        let (a, b) = (edges[0], edges[1]);
        if b <= a {
            return Some("Give the lower frequency first.".to_string());
        }
        if b / a < BAND_MIN_RATIO {
            return Some(format!("The two frequencies must be at least {} times apart, or the bands will not add back evenly.", BAND_MIN_RATIO));
        }
    }
    None
}

/// The Precision Filter settings of band `k` (0 is the lowest): a pair of
/// filters at each edge it touches, as (mode, frequency). Mode 0 is low-pass,
/// 1 is high-pass; each is Butterworth, 2nd order.
pub fn band_slot_specs(k: usize, edges: &[f64]) -> Vec<(u8, f64)> {
    let mut out = Vec::new();
    if k > 0 {
        out.extend([(1, edges[k - 1]); 2]);
    }
    if k < edges.len() {
        out.extend([(0, edges[k]); 2]);
    }
    out
}
