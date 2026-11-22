// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Built-in presets for the Dynamics Curve effect: a segment table turned into
//! the curve's points. The Rust copy of `src/panels/voice-chain/curve-presets.ts`,
//! tested against `shared-fixtures/curve-presets.json`.

use crate::js_compat::js_round;
use std::collections::BTreeMap;

/// Compressor, expander, neutral (no change), or an expander so steep it closes
/// the sound off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegKind {
    C,
    E,
    N,
    X,
}

/// One stretch of the curve: its kind, its ratio and the level (dB) it starts at.
#[derive(Debug, Clone, Copy)]
pub struct Seg(pub SegKind, pub f64, pub f64);

const CLOSED_SLOPE: f64 = 40.0;
const FLOOR: f64 = -120.0;
const CEILING: f64 = 12.0;

fn slope_of(s: &Seg) -> f64 {
    match s.0 {
        SegKind::N => 1.0,
        SegKind::C => 1.0 / s.1.max(1.0),
        SegKind::E => s.1.max(1.0),
        SegKind::X => CLOSED_SLOPE,
    }
}

fn clamp(v: f64) -> f64 {
    v.clamp(FLOOR, CEILING)
}

/// The curve's points, lowest input first, from a segment table.
pub fn curve_points(segs: &[Seg]) -> Vec<(f64, f64)> {
    if segs.len() < 2 {
        return vec![(-60.0, -60.0), (0.0, 0.0)];
    }
    let edges: Vec<f64> = segs[1..].iter().map(|s| s.2).collect();
    let mut out: Vec<f64> = vec![edges[0]];
    for k in 1..edges.len() {
        let prev = out[k - 1];
        out.push(prev - (edges[k - 1] - edges[k]) * slope_of(&segs[k]));
    }
    let mut pts: Vec<(f64, f64)> = edges
        .iter()
        .enumerate()
        .map(|(k, x)| (*x, clamp(out[k])))
        .collect();
    // The top end, at 0 dB in.
    pts.push((0.0, clamp(out[0] + (0.0 - edges[0]) * slope_of(&segs[0]))));
    // The bottom end carries the last stretch's slope past the lowest edge.
    let last = segs[segs.len() - 1];
    let edge = edges[edges.len() - 1];
    let base = out[out.len() - 1];
    if last.0 == SegKind::X {
        pts.push((edge - 1.0, clamp(base - CLOSED_SLOPE)));
    } else if edge > -100.0 {
        pts.push((-100.0, clamp(base - (edge + 100.0) * slope_of(&last))));
    }
    pts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    pts.into_iter()
        .map(|(x, y)| (js_round(x * 100.0) / 100.0, js_round(y * 100.0) / 100.0))
        .collect()
}

/// A curve preset's settings: the segment table, the makeup gain, and a knee in
/// dB (0 for sharp corners).
pub fn curve_params(makeup: f64, segs: &[Seg], knee: f64) -> BTreeMap<String, f64> {
    let pts = curve_points(segs);
    let mut params = BTreeMap::new();
    params.insert("points".to_string(), pts.len() as f64);
    params.insert("knee".to_string(), knee);
    params.insert("makeup".to_string(), makeup);
    params.insert("detect_attack".to_string(), 1.0);
    params.insert("detect_release".to_string(), 300.0);
    params.insert("rms".to_string(), 1.0);
    params.insert("gain_attack".to_string(), 0.1);
    params.insert("gain_release".to_string(), 150.0);
    params.insert("lookahead".to_string(), 3.0);
    for (i, (x, y)) in pts.iter().enumerate() {
        params.insert(format!("p{}_in", i + 1), *x);
        params.insert(format!("p{}_out", i + 1), *y);
    }
    params
}

use SegKind::{C, E, N, X};

/// Every built-in curve preset: name, makeup, segments, knee.
pub fn builtin_curve_presets() -> Vec<(&'static str, f64, Vec<Seg>, f64)> {
    let s = |k, r, l| Seg(k, r, l);
    vec![
        (
            "Narration Curve",
            0.0,
            vec![s(C, 2.5, -15.0), s(C, 2.0, -15.0)],
            0.0,
        ),
        (
            "Vocal, Smooth Curve",
            0.0,
            vec![s(C, 2.0, -20.0), s(N, 1.0, -20.0)],
            0.0,
        ),
        (
            "Vocal, Driven Curve",
            2.0,
            vec![s(C, 3.5, -24.0), s(N, 1.0, -24.0)],
            0.0,
        ),
        (
            "Vocal, Presence Lift",
            0.0,
            vec![s(C, 3.0, -20.0), s(E, 1.5, -20.0), s(X, 1.0, -73.3)],
            0.0,
        ),
        (
            "Vocal, Peak Guard Curve",
            0.0,
            vec![s(C, 6.15, -8.0), s(N, 1.0, -8.0)],
            0.0,
        ),
        (
            "Podcast Host Curve",
            0.0,
            vec![
                s(C, 3.5, -18.9),
                s(E, 1.2, -18.9),
                s(C, 1.3, -27.8),
                s(E, 2.7, -44.1),
                s(X, 1.0, -68.9),
            ],
            4.0,
        ),
        (
            "Sibilance Curve, Light",
            0.0,
            vec![s(C, 1.5, -24.0), s(N, 1.0, -24.0)],
            0.0,
        ),
        (
            "Sibilance Curve, Medium",
            0.0,
            vec![s(C, 2.0, -30.0), s(N, 1.0, -30.0)],
            0.0,
        ),
        (
            "Sibilance Curve, Hard",
            0.0,
            vec![s(C, 4.0, -35.0), s(N, 1.0, -35.0)],
            0.0,
        ),
        (
            "Ceiling Curve, -12 dB",
            0.0,
            vec![s(C, 20.0, -12.0), s(N, 1.0, -12.0)],
            0.0,
        ),
        (
            "Ceiling Curve, -24 dB",
            0.0,
            vec![s(C, 20.0, -24.0), s(N, 1.0, -24.0)],
            4.0,
        ),
        (
            "Broadcast Ceiling Curve",
            0.0,
            vec![s(C, 20.0, -30.0), s(N, 1.0, -30.0)],
            0.0,
        ),
        (
            "Gate Curve, -10 dB",
            0.0,
            vec![s(N, 1.0, -10.0), s(E, 8.9, -10.0), s(C, 18.7, -19.6)],
            0.0,
        ),
        (
            "Gate Curve, -20 dB",
            0.0,
            vec![
                s(N, 1.0, -20.0),
                s(E, 7.1, -20.0),
                s(C, 19.0, -30.8),
                s(X, 1.0, -93.7),
            ],
            0.0,
        ),
        (
            "Expand Below -10 dB, Tail Lift",
            0.0,
            vec![
                s(N, 1.0, -10.0),
                s(E, 3.0, -10.0),
                s(C, 15.3, -38.8),
                s(X, 1.0, -93.9),
            ],
            0.0,
        ),
        (
            "Gradual Squeeze",
            0.0,
            vec![
                s(C, 5.0, -10.0),
                s(C, 2.5, -10.0),
                s(C, 1.5, -24.0),
                s(E, 1.2, -40.0),
            ],
            0.0,
        ),
        (
            "Compander, Gentle",
            0.0,
            vec![
                s(C, 3.75, -9.0),
                s(C, 1.1, -9.0),
                s(E, 2.0, -40.0),
                s(X, 1.0, -71.6),
            ],
            0.0,
        ),
        (
            "Mix Bus, Glue Curve",
            2.0,
            vec![s(C, 2.0, -10.0), s(N, 1.0, -10.0)],
            0.0,
        ),
        (
            "Pump, Maximum",
            20.0,
            vec![s(C, 21.9, -27.6), s(C, 2.4, -27.6), s(E, 3.0, -76.0)],
            0.0,
        ),
    ]
}
