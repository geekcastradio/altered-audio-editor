// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The spectrum view's frequency axis, zoom, scale marks, colors and the
//! rasterizing of spectral selections, in Rust. Mirrors
//! `src/panels/editor/spectrum.ts`, tested against
//! `shared-fixtures/spectrum-view.json`. The edit axis is log spaced from
//! `MIN_HZ` to the Nyquist frequency and matches `dsp/spectral.rs`.

use crate::js_compat::js_round;
use serde::{Deserialize, Serialize};

pub const MIN_HZ: f64 = 30.0;
/// Lowest frequency a log scale shows.
const LOG_FLOOR: f64 = 10.0;
/// The narrowest range zooming in goes to, in Hz.
pub const MIN_SPAN: f64 = 40.0;

/// Frequency at `frac` up the axis (0 bottom, 1 top).
pub fn freq_at(frac: f64, nyquist: f64) -> f64 {
    let top = nyquist.max(MIN_HZ * 2.0);
    MIN_HZ * (top / MIN_HZ).powf(frac.clamp(0.0, 1.0))
}

/// How far up the axis `hz` sits, 0 to 1.
pub fn frac_of(hz: f64, nyquist: f64) -> f64 {
    let top = nyquist.max(MIN_HZ * 2.0);
    if hz <= MIN_HZ {
        return 0.0;
    }
    ((hz / MIN_HZ).ln() / (top / MIN_HZ).ln()).min(1.0)
}

/// How the spectrum view shows frequency: the range in view (`lo` to `hi` Hz;
/// `hi` None is the Nyquist frequency) and how logarithmic the scale is, from
/// 0 (linear) to 1 (log).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FreqView {
    pub lo: f64,
    pub hi: Option<f64>,
    pub log: f64,
}

pub const FULL_VIEW: FreqView = FreqView {
    lo: MIN_HZ,
    hi: None,
    log: 1.0,
};

/// The view's range, settled for a file at `nyquist`.
pub fn view_range(v: &FreqView, nyquist: f64) -> (f64, f64) {
    let hi = nyquist.min(v.hi.unwrap_or(nyquist));
    let lo = 0.0_f64.max(v.lo.min(hi - MIN_SPAN));
    (lo, hi.max(lo + MIN_SPAN))
}

/// Frequency at `frac` up the view (0 bottom, 1 top).
pub fn view_freq(frac: f64, v: &FreqView, nyquist: f64) -> f64 {
    let (lo, hi) = view_range(v, nyquist);
    let t = frac.clamp(0.0, 1.0);
    let k = v.log.clamp(0.0, 1.0);
    let l0 = lo.max(LOG_FLOOR);
    let top = hi.max(l0 + 1.0);
    // A curve that bends from straight (0) to fully logarithmic (1) by degrees.
    let a = k * (top / l0).ln();
    let start = lo + (l0 - lo) * k;
    if a < 1e-6 {
        return start + (top - start) * t;
    }
    start + ((top - start) * (a * t).exp_m1()) / a.exp_m1()
}

/// How far up the view `hz` sits, 0 to 1 (outside the range: below 0 or above 1).
pub fn view_frac(hz: f64, v: &FreqView, nyquist: f64) -> f64 {
    let (lo, hi) = view_range(v, nyquist);
    if hz <= lo {
        return if hz < lo { -0.001 } else { 0.0 };
    }
    if hz >= hi {
        return if hz > hi { 1.001 } else { 1.0 };
    }
    let (mut a, mut b) = (0.0, 1.0);
    // The scale only ever rises, so halving finds the spot.
    for _ in 0..40 {
        let m = (a + b) / 2.0;
        if view_freq(m, v, nyquist) < hz {
            a = m;
        } else {
            b = m;
        }
    }
    (a + b) / 2.0
}

/// Each row's edges, top row first: `rows + 1` frequencies.
pub fn row_edges(rows: usize, v: &FreqView, nyquist: f64) -> Vec<f64> {
    (0..=rows)
        .map(|r| view_freq(1.0 - r as f64 / rows as f64, v, nyquist))
        .collect()
}

/// Zooms the frequency axis by `factor` (below 1 closer) around `hz`, so that
/// frequency stays where it is. Never past 0 Hz or the Nyquist frequency,
/// never narrower than `MIN_SPAN`.
pub fn zoom_freq(v: &FreqView, hz: f64, factor: f64, nyquist: f64) -> FreqView {
    let (lo, hi) = view_range(v, nyquist);
    let at = hi.min(lo.max(hz));
    let (mut nlo, mut nhi);
    if v.log >= 0.5 {
        // Even in pitch: zoom in ratios.
        let l0 = lo.max(LOG_FLOOR);
        let a = at.max(l0);
        nlo = a * (l0 / a).powf(factor);
        nhi = a * (hi / a).powf(factor);
        if lo < LOG_FLOOR && factor >= 1.0 {
            nlo = nlo.min(lo);
        }
    } else {
        nlo = at - (at - lo) * factor;
        nhi = at + (hi - at) * factor;
    }
    if nhi - nlo < MIN_SPAN {
        let mid = (nyquist - MIN_SPAN / 2.0).min((MIN_SPAN / 2.0).max((nlo + nhi) / 2.0));
        nlo = mid - MIN_SPAN / 2.0;
        nhi = mid + MIN_SPAN / 2.0;
    }
    let floor = if v.log >= 0.5 { MIN_HZ } else { 0.0 };
    nlo = (if v.log >= 0.5 { MIN_HZ.min(nlo) } else { 0.0 }).max(nlo);
    nhi = nhi.min(nyquist);
    // Zoomed all the way out: back to the full range.
    let full = nlo <= floor + 1.0 && nhi >= nyquist - 1.0;
    FreqView {
        log: v.log,
        lo: if full { floor } else { 0.0_f64.max(nlo) },
        hi: if full { None } else { Some(nhi) },
    }
}

/// Scale marks for the view: well-spaced frequencies.
pub fn freq_ticks(v: &FreqView, nyquist: f64, height: f64, min_gap: f64) -> Vec<f64> {
    let (lo, hi) = view_range(v, nyquist);
    let mut out: Vec<f64> = vec![];
    if v.log >= 0.5 {
        let mut d = 1.0;
        while d <= 100_000.0 {
            for m in [1.0, 2.0, 3.0, 5.0] {
                out.push(m * d);
            }
            d *= 10.0;
        }
    } else {
        let span = hi - lo;
        let want = 2.0_f64.max((height / (min_gap * 2.0)).floor());
        let raw = span / want;
        let p = 10.0_f64.powf(raw.log10().floor());
        let step = [1.0, 2.0, 5.0, 10.0]
            .iter()
            .map(|m| m * p)
            .find(|s| *s >= raw)
            .unwrap_or(raw);
        let mut f = (lo / step).ceil() * step;
        while f <= hi {
            out.push(js_round(f * 1000.0) / 1000.0);
            f += step;
        }
    }
    let mut inside: Vec<f64> = out.into_iter().filter(|f| *f > lo && *f < hi).collect();
    inside.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut kept = vec![];
    let mut last_y = f64::INFINITY;
    for f in inside {
        let y = (1.0 - view_frac(f, v, nyquist)) * height;
        if last_y - y >= min_gap {
            kept.push(f);
            last_y = y;
        }
    }
    kept
}

/// Color stops from silent to loud: near black, indigo, teal, amber, warm white.
const STOPS: [(f64, [f64; 3]); 5] = [
    (0.0, [8.0, 8.0, 13.0]),
    (0.3, [42.0, 26.0, 94.0]),
    (0.55, [38.0, 128.0, 140.0]),
    (0.8, [222.0, 178.0, 70.0]),
    (1.0, [255.0, 250.0, 228.0]),
];

/// 256 RGB triples, flattened.
pub fn spectrum_lut() -> Vec<u8> {
    let mut out = vec![0u8; 256 * 3];
    for i in 0..256 {
        let t = i as f64 / 255.0;
        let mut k = 0;
        while k < STOPS.len() - 2 && t > STOPS[k + 1].0 {
            k += 1;
        }
        let (t0, c0) = STOPS[k];
        let (t1, c1) = STOPS[k + 1];
        let u = (t - t0) / (t1 - t0);
        for j in 0..3 {
            out[i * 3 + j] = js_round(c0[j] + (c1[j] - c0[j]) * u) as u8;
        }
    }
    out
}

/// One part of a spectral selection. Times are samples; heights are axis
/// positions (0 bottom, 1 top).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SpectralShape {
    Box {
        start: f64,
        end: f64,
        bottom: f64,
        top: f64,
    },
    Lasso {
        points: Vec<(f64, f64)>,
    },
    Brush {
        points: Vec<(f64, f64)>,
        rx: f64,
        ry: f64,
    },
}

/// What the core's spectral edit takes: weights 0 to 255 over time and the axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpectralMask {
    pub start: f64,
    pub spc: f64,
    pub cols: usize,
    pub rows: usize,
    pub frac_top: f64,
    pub frac_per_row: f64,
    pub data: Vec<u8>,
}

/// Time and axis extent of shapes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub t0: f64,
    pub t1: f64,
    pub f0: f64,
    pub f1: f64,
}

pub fn shapes_bounds(shapes: &[SpectralShape]) -> Option<Bounds> {
    let (mut t0, mut t1, mut f0, mut f1) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for s in shapes {
        match s {
            SpectralShape::Box {
                start,
                end,
                bottom,
                top,
            } => {
                t0 = t0.min(*start);
                t1 = t1.max(*end);
                f0 = f0.min(*bottom);
                f1 = f1.max(*top);
            }
            SpectralShape::Lasso { points } => {
                for (t, f) in points {
                    t0 = t0.min(*t);
                    t1 = t1.max(*t);
                    f0 = f0.min(*f);
                    f1 = f1.max(*f);
                }
            }
            SpectralShape::Brush { points, rx, ry } => {
                for (t, f) in points {
                    t0 = t0.min(t - rx);
                    t1 = t1.max(t + rx);
                    f0 = f0.min(f - ry);
                    f1 = f1.max(f + ry);
                }
            }
        }
    }
    if t1 > t0 && f1 > f0 {
        Some(Bounds {
            t0: t0.max(0.0),
            t1,
            f0: f0.max(0.0),
            f1: f1.min(1.0),
        })
    } else {
        None
    }
}

fn in_polygon(x: f64, y: f64, pts: &[(f64, f64)]) -> bool {
    let mut inside = false;
    let mut j = pts.len().wrapping_sub(1);
    for i in 0..pts.len() {
        let (xi, yi) = pts[i];
        let (xj, yj) = pts[j];
        if (yi > y) != (yj > y) && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Turns shapes into a weight grid for the core. None when nothing is selected.
pub fn rasterize(
    shapes: &[SpectralShape],
    max_cols: usize,
    max_rows: usize,
) -> Option<SpectralMask> {
    let b = shapes_bounds(shapes)?;
    let cols = 1.max(max_cols.min((b.t1 - b.t0).ceil() as usize));
    let spc = (b.t1 - b.t0) / cols as f64;
    let rows = 1.max(max_rows.min(((b.f1 - b.f0) * 1000.0).ceil() as usize));
    let fpr = (b.f1 - b.f0) / rows as f64;
    let mut data = vec![0u8; cols * rows];
    let t_at = |c: usize| b.t0 + (c as f64 + 0.5) * spc;
    let f_at = |r: usize| b.f1 - (r as f64 + 0.5) * fpr;
    // Index range clamped to the grid; None when the range misses it.
    let span = |lo: f64, hi: f64, n: usize| -> Option<(usize, usize)> {
        let hi = hi.min(n as f64 - 1.0);
        let lo = lo.max(0.0);
        (hi >= lo).then_some((lo as usize, hi as usize))
    };
    for s in shapes {
        match s {
            SpectralShape::Box {
                start,
                end,
                bottom,
                top,
            } => {
                let c0 = ((start - b.t0) / spc).floor();
                let c1 = ((end - b.t0) / spc).ceil() - 1.0;
                let r0 = ((b.f1 - top) / fpr).floor();
                let r1 = ((b.f1 - bottom) / fpr).ceil() - 1.0;
                if let (Some((c0, c1)), Some((r0, r1))) = (span(c0, c1, cols), span(r0, r1, rows)) {
                    for r in r0..=r1 {
                        for c in c0..=c1 {
                            data[r * cols + c] = 255;
                        }
                    }
                }
            }
            SpectralShape::Lasso { points } => {
                if points.len() < 3 {
                    continue;
                }
                for r in 0..rows {
                    for c in 0..cols {
                        if in_polygon(t_at(c), f_at(r), points) {
                            data[r * cols + c] = 255;
                        }
                    }
                }
            }
            SpectralShape::Brush { points, rx, ry } => {
                // An ellipse swept along each segment.
                let pts: Vec<(f64, f64)> = if points.len() == 1 {
                    vec![points[0], points[0]]
                } else {
                    points.clone()
                };
                for i in 0..pts.len().saturating_sub(1) {
                    let (ax, ay) = pts[i];
                    let (bx, by) = pts[i + 1];
                    let c0 = ((ax.min(bx) - rx - b.t0) / spc).floor();
                    let c1 = ((ax.max(bx) + rx - b.t0) / spc).ceil();
                    let r0 = ((b.f1 - ay.max(by) - ry) / fpr).floor();
                    let r1 = ((b.f1 - ay.min(by) + ry) / fpr).ceil();
                    let (Some((c0, c1)), Some((r0, r1))) = (span(c0, c1, cols), span(r0, r1, rows))
                    else {
                        continue;
                    };
                    // Work in units where the brush is a unit circle.
                    let ux = (bx - ax) / rx;
                    let uy = (by - ay) / ry;
                    let len2 = ux * ux + uy * uy;
                    for r in r0..=r1 {
                        for c in c0..=c1 {
                            let px = (t_at(c) - ax) / rx;
                            let py = (f_at(r) - ay) / ry;
                            let t = if len2 > 0.0 {
                                ((px * ux + py * uy) / len2).clamp(0.0, 1.0)
                            } else {
                                0.0
                            };
                            let dx = px - t * ux;
                            let dy = py - t * uy;
                            if dx * dx + dy * dy <= 1.0 {
                                data[r * cols + c] = 255;
                            }
                        }
                    }
                }
            }
        }
    }
    if !data.iter().any(|v| *v > 0) {
        return None;
    }
    Some(SpectralMask {
        start: b.t0,
        spc,
        cols,
        rows,
        frac_top: b.f1,
        frac_per_row: fpr,
        data,
    })
}
