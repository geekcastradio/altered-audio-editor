// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Dynamics window's graph, in Rust: output level against input level for
//! the compressor's settings.
//!
//! These mirror `src/panels/voice-chain/dynamics.ts`. Both are tested against
//! `shared-fixtures/dynamics-curve.json`, so the two copies cannot drift. When
//! the interface moves to Rust, the TypeScript copy goes away.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve {
    pub threshold: f64,
    pub ratio: f64,
    pub knee: f64,
    pub makeup: f64,
}

/// The graph's usual range and detail in the TypeScript copy.
pub const DEFAULT_LO: f64 = -60.0;
pub const DEFAULT_HI: f64 = 0.0;
pub const DEFAULT_STEPS: usize = 60;

/// Output level (dB) for an input level (dB), with a soft knee centred on the threshold.
pub fn transfer(in_db: f64, c: &Curve) -> f64 {
    let Curve {
        threshold: t,
        ratio: r,
        knee: k,
        makeup: m,
    } = *c;
    let over = in_db - t;
    let out = if k > 0.0 && over.abs() <= k / 2.0 {
        let x = over + k / 2.0;
        in_db + ((1.0 / r - 1.0) * x * x) / (2.0 * k)
    } else if over > 0.0 {
        t + over / r
    } else {
        in_db
    };
    out + m
}

/// Points along the curve from `lo` to `hi` dB in, for drawing.
pub fn curve_points(c: &Curve, lo: f64, hi: f64, steps: usize) -> Vec<(f64, f64)> {
    (0..=steps)
        .map(|i| {
            let x = lo + ((hi - lo) * i as f64) / steps as f64;
            (x, transfer(x, c))
        })
        .collect()
}

/// The ratio that gives a stated threshold-to-output slope, clamped to the control's range.
/// The TypeScript copy defaults to a range of 1 to 20.
pub fn ratio_from_slope(slope: f64, min: f64, max: f64) -> f64 {
    let r = if slope <= 0.0001 { max } else { 1.0 / slope };
    r.max(min).min(max)
}
