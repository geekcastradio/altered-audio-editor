// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Match EQ: moves the tone of one voice toward another. The Rust copy of
//! `src/app/match-eq.ts`; both are tested against
//! `shared-fixtures/match-eq.json`.
//!
//! Both voices are measured as their long-term average spectrum. The
//! difference between them, smoothed and levelled so the overall volume does
//! not change, is the curve to add. That curve is fitted with what the
//! Parametric EQ has: a low shelf, a high shelf and three bands, so the
//! result is an ordinary, editable EQ. The method is a greedy least-squares
//! fit using the filter shapes the EQ runs (the published audio EQ cookbook
//! formulas).

use crate::js_compat::js_round;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Spectrum {
    pub hz: Vec<f64>,
    pub db: Vec<f64>,
}

/// Where the match is trusted. Outside, the curve is held flat.
pub const MATCH_LOW_HZ: f64 = 80.0;
pub const MATCH_HIGH_HZ: f64 = 12_000.0;
/// Largest boost or cut asked of any band, dB.
pub const MAX_GAIN_DB: f64 = 12.0;
/// Bands smaller than this are left at zero, dB.
const MIN_BAND_DB: f64 = 0.5;
const RATE: f64 = 48_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Peak,
    LowShelf,
    HighShelf,
}

/// Magnitude in dB at `f` of one filter, from the cookbook coefficients
/// (shelf slope 1 as the EQ uses).
pub fn response_db(shape: Shape, rate: f64, f0: f64, q: f64, gain_db: f64, f: f64) -> f64 {
    let w0 = (2.0 * std::f64::consts::PI * f0.min(rate * 0.49)) / rate;
    let (sw, cw) = (w0.sin(), w0.cos());
    let a = 10f64.powf(gain_db / 40.0);
    let (b, d) = if shape == Shape::Peak {
        let alpha = sw / (2.0 * q.max(0.01));
        (
            [1.0 + alpha * a, -2.0 * cw, 1.0 - alpha * a],
            [1.0 + alpha / a, -2.0 * cw, 1.0 - alpha / a],
        )
    } else {
        let alpha = (sw / 2.0) * 2f64.sqrt();
        let k = 2.0 * a.sqrt() * alpha;
        if shape == Shape::LowShelf {
            (
                [
                    a * (a + 1.0 - (a - 1.0) * cw + k),
                    2.0 * a * (a - 1.0 - (a + 1.0) * cw),
                    a * (a + 1.0 - (a - 1.0) * cw - k),
                ],
                [
                    a + 1.0 + (a - 1.0) * cw + k,
                    -2.0 * (a - 1.0 + (a + 1.0) * cw),
                    a + 1.0 + (a - 1.0) * cw - k,
                ],
            )
        } else {
            (
                [
                    a * (a + 1.0 + (a - 1.0) * cw + k),
                    -2.0 * a * (a - 1.0 + (a + 1.0) * cw),
                    a * (a + 1.0 + (a - 1.0) * cw - k),
                ],
                [
                    a + 1.0 - (a - 1.0) * cw + k,
                    2.0 * (a - 1.0 - (a + 1.0) * cw),
                    a + 1.0 - (a - 1.0) * cw - k,
                ],
            )
        }
    };
    let w = (2.0 * std::f64::consts::PI * f) / rate;
    let (c1, s1, c2, s2) = (w.cos(), w.sin(), (2.0 * w).cos(), (2.0 * w).sin());
    let nr = b[0] + b[1] * c1 + b[2] * c2;
    let ni = -(b[1] * s1 + b[2] * s2);
    let dr = d[0] + d[1] * c1 + d[2] * c2;
    let di = -(d[1] * s1 + d[2] * s2);
    10.0 * ((nr * nr + ni * ni) / (dr * dr + di * di))
        .max(1e-24)
        .log10()
}

/// Reference interpolated (in dB, on a log axis) onto `hz`.
fn resample(from: &Spectrum, hz: &[f64]) -> Vec<f64> {
    let mut out = Vec::with_capacity(hz.len());
    let mut j = 0;
    for &f in hz {
        while j + 2 < from.hz.len() && from.hz[j + 1] < f {
            j += 1;
        }
        let f0 = from.hz[j];
        let f1 = from.hz.get(j + 1).copied().unwrap_or(f0);
        let v0 = from.db[j];
        let v1 = from.db.get(j + 1).copied().unwrap_or(v0);
        let t = if f1 > f0 {
            ((f / f0).ln() / (f1 / f0).ln()).clamp(0.0, 1.0)
        } else {
            0.0
        };
        out.push(v0 + (v1 - v0) * t);
    }
    out
}

/// Moving average over +/- `oct` octaves on the log axis.
fn smooth(hz: &[f64], v: &[f64], oct: f64) -> Vec<f64> {
    (0..v.len())
        .map(|i| {
            let lo = hz[i] / 2f64.powf(oct);
            let hi = hz[i] * 2f64.powf(oct);
            let (mut s, mut n) = (0.0, 0.0);
            for k in 0..hz.len() {
                if hz[k] >= lo && hz[k] <= hi {
                    s += v[k];
                    n += 1.0;
                }
            }
            if n > 0.0 {
                s / n
            } else {
                v[i]
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchFit {
    /// Parametric EQ settings (keys of altaudio.parametric_eq).
    pub params: BTreeMap<String, f64>,
    /// The curve asked for on the source's grid, dB.
    pub target: Vec<f64>,
    /// What the EQ settings give on that grid, dB.
    pub fitted: Vec<f64>,
    pub hz: Vec<f64>,
    /// RMS difference from the wanted curve inside the trusted range: before
    /// and after the fit, dB.
    pub error_before: f64,
    pub error_after: f64,
}

/// Options for a gentler or capped match.
#[derive(Debug, Clone, Copy, Default)]
pub struct MatchOptions {
    /// Largest boost or cut asked of any band, dB. Default `MAX_GAIN_DB`.
    pub cap_db: Option<f64>,
    /// 0 to 1: how much of the difference to apply. Default 1.
    pub strength: Option<f64>,
}

const BAND_RANGES: [(f64, f64); 3] = [(120.0, 700.0), (500.0, 3500.0), (2000.0, 10_000.0)];
const Q_CHOICES: [f64; 3] = [0.7, 1.2, 2.2];

fn round1(x: f64) -> f64 {
    js_round(x * 10.0) / 10.0
}

/// The Parametric EQ settings that move `source` toward `reference`. None when
/// the source has fewer than eight points, its lists differ in length, or the
/// reference is empty.
pub fn match_eq(source: &Spectrum, reference: &Spectrum, opts: MatchOptions) -> Option<MatchFit> {
    let cap = opts.cap_db.unwrap_or(MAX_GAIN_DB);
    let strength = opts.strength.unwrap_or(1.0);
    let hz = source.hz.clone();
    if hz.len() < 8
        || source.db.len() != hz.len()
        || reference.hz.is_empty()
        || reference.db.len() != reference.hz.len()
    {
        return None;
    }
    let ref_db = resample(reference, &hz);
    let in_range = |f: f64| (MATCH_LOW_HZ..=MATCH_HIGH_HZ).contains(&f);
    // The wanted curve: what the reference has that the source lacks.
    let nearest = |f: f64| {
        let mut best = 0;
        for (k, h) in hz.iter().enumerate() {
            if (h / f).ln().abs() < (hz[best] / f).ln().abs() {
                best = k;
            }
        }
        best
    };
    let raw: Vec<f64> = hz
        .iter()
        .map(|&f| {
            // Outside the trusted range the curve holds the value at the edge.
            let j = nearest(f.clamp(MATCH_LOW_HZ, MATCH_HIGH_HZ));
            ref_db[j] - source.db[j]
        })
        .collect();
    let mut target = smooth(&hz, &raw, 1.0 / 6.0);
    // Keep the overall level: the speech band (200 Hz to 5 kHz) averages zero.
    let band: Vec<usize> = hz
        .iter()
        .enumerate()
        .filter(|(_, f)| **f >= 200.0 && **f <= 5000.0)
        .map(|(i, _)| i)
        .collect();
    let mean = if band.is_empty() {
        0.0
    } else {
        band.iter().fold(0.0, |s, &i| s + target[i]) / band.len() as f64
    };
    target = target
        .iter()
        .map(|v| ((v - mean) * strength).min(cap).max(-cap))
        .collect();

    let weight: Vec<f64> = hz
        .iter()
        .map(|&f| if in_range(f) { 1.0 } else { 0.0 })
        .collect();
    let rms = |r: &[f64]| {
        let (mut s, mut n) = (0.0, 0.0);
        for (i, v) in r.iter().enumerate() {
            s += weight[i] * v * v;
            n += weight[i];
        }
        if n > 0.0 {
            (s / n).sqrt()
        } else {
            0.0
        }
    };
    let error_before = rms(&target);
    let mut residual = target.clone();
    let mut fitted = vec![0.0; hz.len()];
    let mut take = |residual: &mut Vec<f64>, shape: Shape, f0: f64, q: f64, gain: f64| {
        for (i, &f) in hz.iter().enumerate() {
            let r = response_db(shape, RATE, f0, q, gain, f);
            fitted[i] += r;
            residual[i] -= r;
        }
    };

    let mut params: BTreeMap<String, f64> =
        BTreeMap::from([("hp_on".to_string(), 0.0), ("lp_on".to_string(), 0.0)]);
    // The shelves, one at a time: least squares on the unit-gain shape.
    let mut shelf = |residual: &mut Vec<f64>, shape: Shape, f0: f64, key: &str| {
        let unit: Vec<f64> = hz
            .iter()
            .map(|&f| response_db(shape, RATE, f0, 1.0, 6.0, f) / 6.0)
            .collect();
        let (mut num, mut den) = (0.0, 0.0);
        for i in 0..hz.len() {
            num += weight[i] * residual[i] * unit[i];
            den += weight[i] * unit[i] * unit[i];
        }
        let g = if den > 0.0 {
            (num / den).min(cap).max(-cap)
        } else {
            0.0
        };
        let gain = if g.abs() < MIN_BAND_DB {
            0.0
        } else {
            round1(g)
        };
        params.insert(format!("{key}_freq"), f0);
        params.insert(format!("{key}_gain"), gain);
        if gain != 0.0 {
            take(residual, shape, f0, 1.0, gain);
        }
    };
    shelf(&mut residual, Shape::LowShelf, 150.0, "ls");
    shelf(&mut residual, Shape::HighShelf, 7000.0, "hs");

    // The three bands: each lands on the biggest remaining difference in its range.
    for (n, (lo, hi)) in BAND_RANGES.iter().enumerate() {
        let key = format!("b{}", n + 1);
        let sm = smooth(&hz, &residual, 1.0 / 8.0);
        let mut best: Option<usize> = None;
        for (i, &f) in hz.iter().enumerate() {
            if f >= *lo && f <= *hi && best.is_none_or(|b| sm[i].abs() > sm[b].abs()) {
                best = Some(i);
            }
        }
        let Some(best) = best.filter(|&b| sm[b].abs() >= MIN_BAND_DB) else {
            params.insert(format!("{key}_gain"), 0.0);
            continue;
        };
        let f0 = hz[best];
        let gain = round1((sm[best] * 1.05).min(cap).max(-cap));
        // Pick the width that leaves the least error.
        let mut best_q = Q_CHOICES[0];
        let mut best_err = f64::INFINITY;
        for q in Q_CHOICES {
            let trial: Vec<f64> = residual
                .iter()
                .enumerate()
                .map(|(i, r)| r - response_db(Shape::Peak, RATE, f0, q, gain, hz[i]))
                .collect();
            let e = rms(&trial);
            if e < best_err {
                best_err = e;
                best_q = q;
            }
        }
        params.insert(format!("{key}_freq"), js_round(f0));
        params.insert(format!("{key}_gain"), gain);
        params.insert(format!("{key}_q"), best_q);
        take(&mut residual, Shape::Peak, f0, best_q, gain);
    }

    let error_after = rms(&residual);
    Some(MatchFit {
        params,
        target,
        fitted,
        hz,
        error_before,
        error_after,
    })
}

/// Voice AutoEQ: a gentle, capped suggestion, so a stronger fix is a choice
/// and not a default.
pub const VOICE_TARGET_OPTS: MatchOptions = MatchOptions {
    cap_db: Some(6.0),
    strength: Some(0.8),
};

/// Relative level of the built-in neutral speech target, dB re 1 kHz: a mild
/// speech tilt. It is a starting point, not tuned by ear.
const NEUTRAL: [(f64, f64); 11] = [
    (50.0, -9.0),
    (100.0, 6.0),
    (200.0, 6.0),
    (400.0, 3.0),
    (1000.0, 0.0),
    (2000.0, -2.0),
    (4000.0, -4.0),
    (8000.0, -8.0),
    (12000.0, -12.0),
    (16000.0, -18.0),
    (20000.0, -24.0),
];

/// The neutral speech target on a log grid from 50 Hz to 20 kHz.
pub fn voice_target(points: usize) -> Spectrum {
    let hz: Vec<f64> = (0..points)
        .map(|i| 50.0 * 400f64.powf(i as f64 / (points as f64 - 1.0)))
        .collect();
    let db = hz
        .iter()
        .map(|&f| {
            for k in 0..NEUTRAL.len() - 1 {
                let (f0, g0) = NEUTRAL[k];
                let (f1, g1) = NEUTRAL[k + 1];
                if f >= f0 && f <= f1 {
                    return g0 + ((g1 - g0) * (f / f0).ln()) / (f1 / f0).ln();
                }
            }
            NEUTRAL[0].1
        })
        .collect();
    Spectrum { hz, db }
}

/// The Parametric EQ settings that move `source` toward the neutral speech
/// target.
pub fn voice_target_eq(source: &Spectrum) -> Option<MatchFit> {
    match_eq(source, &voice_target(64), VOICE_TARGET_OPTS)
}
