// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Filling a selected gap (a click, pop or burst) from the sound on both sides.
//!
//! Textbook least-squares autoregressive interpolation: a short model of the
//! sound around the gap is fitted, then the missing samples are chosen to
//! fit that model as well as they can, which joins both sides smoothly with no
//! jump. Alone that leaves a quiet dip in the middle of a long gap (the best
//! guess for noise is nothing), so noise at the level the model measured
//! outside the gap is added back, drawn from the same model so it joins
//! cleanly. The noise comes from a fixed seed, so the same selection always
//! gives the same result.
//!
//! The texture choice decides whether that noise goes in: Natural adds it
//! (the gap sounds like the room around it), Smooth leaves it out (the
//! cleanest curve, which can sound like a short quiet dip on long gaps but
//! never adds hiss of its own).

use super::fill_span;

/// How the middle of a filled gap sounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillTexture {
    /// Matched noise from the sound around the gap is added back.
    #[default]
    Natural,
    /// Only the fitted curve: no noise is added.
    Smooth,
}

impl FillTexture {
    /// Reads the name the interface sends; anything unknown is Natural.
    pub fn from_name(name: Option<&str>) -> Self {
        match name {
            Some(n) if n.eq_ignore_ascii_case("smooth") => Self::Smooth,
            _ => Self::Natural,
        }
    }
}

/// Samples of context read on each side, at 48 kHz (scaled for other rates).
pub const CONTEXT_48K: usize = 1024;

/// How many samples either side the fit needs: four times the model order.
pub fn context_needed(rate: u32, gap: usize) -> usize {
    4 * order_for(rate, gap)
}

/// How many samples either side are worth reading, so callers can fetch them once.
pub fn context_wanted(rate: u32, gap: usize) -> usize {
    ((rate as usize * CONTEXT_48K) / 48_000).max(context_needed(rate, gap))
}

fn order_for(rate: u32, gap: usize) -> usize {
    // 32 up to 25 ms, 48 beyond.
    if gap * 1000 <= 25 * rate as usize {
        32
    } else {
        48
    }
}

struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        // splitmix64
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }
    /// A normal draw (Box-Muller).
    fn normal(&mut self) -> f64 {
        let (u, v) = (self.unit(), self.unit());
        (-2.0 * u.ln()).sqrt() * (2.0 * std::f64::consts::PI * v).cos()
    }
}

/// Windowed autocorrelation of one stretch of context, lags 0..=p.
fn autocorr(seg: &[f64], p: usize) -> Vec<f64> {
    let m = seg.len();
    let w: Vec<f64> = (0..m)
        .map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (m as f64 - 1.0)).cos())
        .collect();
    let s: Vec<f64> = seg.iter().zip(&w).map(|(a, b)| a * b).collect();
    (0..=p)
        .map(|l| (0..m - l).map(|i| s[i] * s[i + l]).sum())
        .collect()
}

/// Model coefficients `a[0] = 1, a[1..=p]` from autocorrelation `r` (Levinson-Durbin).
fn levinson(r: &[f64], p: usize) -> Option<Vec<f64>> {
    let mut a = vec![0.0; p + 1];
    a[0] = 1.0;
    let mut err = r[0];
    for i in 1..=p {
        let mut acc = r[i];
        for j in 1..i {
            acc += a[j] * r[i - j];
        }
        let k = -acc / err;
        let prev = a.clone();
        for j in 1..i {
            a[j] = prev[j] + k * prev[i - j];
        }
        a[i] = k;
        err *= 1.0 - k * k;
        // Written positively so a NaN also fails the check.
        if err.is_nan() || err <= 0.0 || !err.is_finite() {
            return None;
        }
    }
    Some(a)
}

/// Fills `x[s..e]` from the sound on both sides. Returns false (and leaves `x`
/// alone) when there is nothing to fit from: no room on a side, or silence.
/// Short of context the fill falls back to the plain curve of `fill_span`.
pub fn fill_gap(
    x: &mut [f32],
    s: usize,
    e: usize,
    rate: u32,
    seed: u64,
    texture: FillTexture,
) -> bool {
    if e <= s || s < 2 || e + 2 > x.len() {
        return false;
    }
    let n = e - s;
    let p = order_for(rate, n);
    let want = context_wanted(rate, n);
    let (lead, tail) = (s.min(want), (x.len() - e).min(want));
    if lead < 4 * p || tail < 4 * p {
        return fill_span(x, s, e);
    }
    let left: Vec<f64> = x[s - lead..s].iter().map(|&v| f64::from(v)).collect();
    let right: Vec<f64> = x[e..e + tail].iter().map(|&v| f64::from(v)).collect();

    let (ra, rb) = (autocorr(&left, p), autocorr(&right, p));
    let mut r: Vec<f64> = ra.iter().zip(&rb).map(|(a, b)| a + b).collect();
    if r[0].is_nan() || r[0] <= 1e-18 {
        // Digital silence on both sides: the plain curve is as good as anything.
        return fill_span(x, s, e);
    }
    r[0] = r[0] * (1.0 + 1e-9) + 1e-18;
    let Some(a) = levinson(&r, p) else {
        return fill_span(x, s, e);
    };

    // How loud the model's leftover (the noise) is outside the gap.
    let mut sq = 0.0;
    let mut cnt = 0usize;
    for seg in [&left, &right] {
        for t in p..seg.len() {
            let e_t: f64 = (0..=p).map(|k| a[k] * seg[t - k]).sum();
            sq += e_t * e_t;
            cnt += 1;
        }
    }
    let sigma = (sq / cnt.max(1) as f64).sqrt();

    // Autocorrelation of the model's own coefficients: the system's matrix.
    let rl: Vec<f64> = (0..=p)
        .map(|l| (0..=p - l).map(|k| a[k] * a[k + l]).sum())
        .collect();

    // Right-hand side: what the known samples on each side pull the gap towards.
    let mut b = vec![0.0f64; n];
    for (i, bi) in b.iter_mut().enumerate() {
        let gi = s + i;
        let lo = gi.saturating_sub(p);
        let hi = (gi + p).min(x.len() - 1);
        for (m, &xm) in x.iter().enumerate().take(hi + 1).skip(lo) {
            if m >= s && m < e {
                continue;
            }
            *bi -= rl[gi.abs_diff(m)] * f64::from(xm);
        }
    }

    // Banded Cholesky, M = L Lt, band[i * (p + 1) + k] = L[i][i - k].
    let w = p + 1;
    let mut band = vec![0.0f64; n * w];
    for i in 0..n {
        for j in i.saturating_sub(p)..=i {
            let mut sum = rl[i - j];
            for m in i.saturating_sub(p).max(j.saturating_sub(p))..j {
                sum -= band[i * w + (i - m)] * band[j * w + (j - m)];
            }
            if j == i {
                if sum.is_nan() || sum <= 0.0 {
                    return fill_span(x, s, e);
                }
                band[i * w] = sum.sqrt();
            } else {
                band[i * w + (i - j)] = sum / band[j * w];
            }
        }
    }
    // L y = b, then Lt u = y.
    let mut u = vec![0.0f64; n];
    for i in 0..n {
        let mut sum = b[i];
        for m in i.saturating_sub(p)..i {
            sum -= band[i * w + (i - m)] * u[m];
        }
        u[i] = sum / band[i * w];
    }
    let back = |v: &mut [f64]| {
        for i in (0..n).rev() {
            let mut sum = v[i];
            for k in i + 1..=(i + p).min(n - 1) {
                sum -= band[k * w + (k - i)] * v[k];
            }
            v[i] = sum / band[i * w];
        }
    };
    back(&mut u);

    // Noise at the level measured outside, joined to both sides by the same model.
    let ms = n as f64 * 1000.0 / f64::from(rate.max(1));
    let gain = match texture {
        FillTexture::Natural => ((ms - 1.0) / 4.0).clamp(0.0, 1.0),
        FillTexture::Smooth => 0.0,
    };
    if gain > 0.0 && sigma.is_finite() {
        let mut rng = Rng(seed ^ 0xA5A5_5A5A_1234_5678);
        let mut z: Vec<f64> = (0..n).map(|_| rng.normal()).collect();
        back(&mut z);
        for (ui, zi) in u.iter_mut().zip(&z) {
            *ui += gain * sigma * zi;
        }
    }
    if u.iter().any(|v| !v.is_finite()) {
        return fill_span(x, s, e);
    }
    for (xi, ui) in x[s..e].iter_mut().zip(&u) {
        *xi = *ui as f32;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A quiet, steady sound with some body to it (filtered noise), repeatable.
    fn bed(len: usize, seed: u64) -> Vec<f32> {
        let mut rng = Rng(seed);
        let mut y1 = 0.0f64;
        let mut y2 = 0.0f64;
        (0..len)
            .map(|_| {
                let v = rng.normal() * 0.01 + 1.6 * y1 - 0.8 * y2;
                y2 = y1;
                y1 = v;
                (v * 0.2) as f32
            })
            .collect()
    }

    fn rms(v: &[f32]) -> f64 {
        (v.iter().map(|&a| f64::from(a).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    }

    fn click(x: &mut [f32], s: usize, e: usize) {
        for (i, v) in x[s..e].iter_mut().enumerate() {
            *v += if i % 2 == 0 { 0.4 } else { -0.35 };
        }
    }

    #[test]
    fn fills_a_long_gap_at_the_level_of_the_sound_around_it() {
        let rate = 48_000;
        for ms in [5usize, 25, 40, 60, 100] {
            let n = rate as usize * ms / 1000;
            let clean = bed(20_000 + n, 7);
            let (s, e) = (10_000, 10_000 + n);
            let mut x = clean.clone();
            click(&mut x, s, e);
            assert!(fill_gap(&mut x, s, e, rate, 1, FillTexture::Natural));
            let ratio = rms(&x[s..e]) / rms(&clean[s..e]);
            assert!((0.5..1.8).contains(&ratio), "{ms} ms: level {ratio}");
            let peak = x[s..e].iter().fold(0f32, |m, v| m.max(v.abs()));
            let clean_peak = clean[s - 2000..s].iter().fold(0f32, |m, v| m.max(v.abs()));
            assert!(
                peak < 3.0 * clean_peak,
                "{ms} ms: arch {peak} vs {clean_peak}"
            );
            assert_eq!(x[..s], clean[..s]);
            assert_eq!(x[e..], clean[e..]);
        }
    }

    #[test]
    fn joins_both_sides_without_a_jump() {
        let rate = 48_000;
        let n = 1920;
        let clean = bed(20_000, 3);
        let (s, e) = (9_000, 9_000 + n);
        let mut x = clean.clone();
        click(&mut x, s, e);
        assert!(fill_gap(&mut x, s, e, rate, 9, FillTexture::Natural));
        let step = |a: f32, b: f32| (a - b).abs();
        let typical = (1..2000)
            .map(|i| step(clean[s - i], clean[s - i - 1]))
            .fold(0f32, f32::max);
        assert!(step(x[s - 1], x[s]) < 3.0 * typical, "left join");
        assert!(step(x[e - 1], x[e]) < 3.0 * typical, "right join");
    }

    #[test]
    fn the_same_selection_gives_the_same_fill() {
        let clean = bed(20_000, 5);
        let run = |seed| {
            let mut x = clean.clone();
            click(&mut x, 9_000, 10_200);
            assert!(fill_gap(
                &mut x,
                9_000,
                10_200,
                48_000,
                seed,
                FillTexture::Natural
            ));
            x
        };
        assert_eq!(run(4), run(4));
        assert_ne!(run(4), run(5));
    }

    #[test]
    fn a_very_short_click_is_smoothed_without_noise() {
        let clean = bed(20_000, 11);
        let mut x = clean.clone();
        click(&mut x, 9_000, 9_048);
        assert!(fill_gap(
            &mut x,
            9_000,
            9_048,
            48_000,
            1,
            FillTexture::Natural
        ));
        let err = (0..48)
            .map(|i| (x[9_000 + i] - clean[9_000 + i]).abs())
            .fold(0f32, f32::max);
        assert!(err < 0.06, "1 ms click left {err}");
    }

    #[test]
    fn smooth_adds_no_noise_and_natural_does() {
        let rate = 48_000;
        let n = 2_880; // 60 ms
        let clean = bed(20_000 + n, 21);
        let (s, e) = (10_000, 10_000 + n);
        let run = |t: FillTexture, seed: u64| {
            let mut x = clean.clone();
            click(&mut x, s, e);
            assert!(fill_gap(&mut x, s, e, rate, seed, t));
            x
        };
        // Smooth ignores the seed: no noise is drawn.
        assert_eq!(run(FillTexture::Smooth, 1), run(FillTexture::Smooth, 2));
        // Natural fills the middle of a long gap louder than Smooth's dip.
        let nat = run(FillTexture::Natural, 1);
        let smo = run(FillTexture::Smooth, 1);
        let mid = s + n / 4..e - n / 4;
        assert!(
            rms(&nat[mid.clone()]) > 1.5 * rms(&smo[mid]),
            "natural should carry the room noise"
        );
        // Both leave the audio outside the gap alone and join without a jump.
        for x in [&nat, &smo] {
            assert_eq!(x[..s], clean[..s]);
            assert_eq!(x[e..], clean[e..]);
            assert!(x[s..e].iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn texture_names_read_as_expected() {
        assert_eq!(FillTexture::from_name(Some("smooth")), FillTexture::Smooth);
        assert_eq!(FillTexture::from_name(Some("Smooth")), FillTexture::Smooth);
        assert_eq!(
            FillTexture::from_name(Some("natural")),
            FillTexture::Natural
        );
        assert_eq!(FillTexture::from_name(Some("wobbly")), FillTexture::Natural);
        assert_eq!(FillTexture::from_name(None), FillTexture::Natural);
    }

    #[test]
    fn silence_and_short_context_fall_back_instead_of_failing() {
        let mut quiet = vec![0.0f32; 20_000];
        quiet[9_000..9_100].fill(0.5);
        assert!(fill_gap(
            &mut quiet,
            9_000,
            9_100,
            48_000,
            1,
            FillTexture::Natural
        ));
        assert!(quiet.iter().all(|v| v.is_finite()));
        // Too little context for the fit: the plain curve is used.
        let mut short = bed(300, 2);
        click(&mut short, 150, 160);
        assert!(fill_gap(
            &mut short,
            150,
            160,
            48_000,
            1,
            FillTexture::Natural
        ));
        assert!(short.iter().all(|v| v.is_finite()));
        // No room at all.
        let mut edge = bed(300, 2);
        assert!(!fill_gap(&mut edge, 1, 5, 48_000, 1, FillTexture::Natural));
        assert!(!fill_gap(
            &mut edge,
            10,
            10,
            48_000,
            1,
            FillTexture::Natural
        ));
    }
}
