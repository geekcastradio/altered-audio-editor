// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Center Channel Extractor: keeps or removes what sits in the middle of
//! a stereo mix (a lead voice, a mono source) by looking at each
//! frequency bin of each short frame. A bin whose left and right parts
//! have the same size and the same phase is "in the center"; the more the
//! two differ, the further from the center it is. Each bin then gets a
//! gain from how central it is, the same for both channels.
//!
//! The centered part is measured as `2 * Re(L * conj(R)) / (|L|^2 +
//! |R|^2)`: 1 for identical parts, 0 when they have nothing in common
//! and negative when they are opposite. Width sets how far from 1 still
//! counts as center. The gains are smoothed from frame to frame, which
//! keeps the "watery" sound of spectral editing down. Frames are 75%
//! overlapped and the output is delayed by one frame, reported as latency.
//!
//! All original code built from textbook techniques.

use std::sync::Arc;

use realfft::num_complex::Complex32;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};

use super::{choice, num, EffectInfo, Params, Processor};

const MODES: [&str; 2] = ["Keep the center", "Remove the center"];

pub fn info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.center_extract",
        name: "Center Channel Extractor",
        category: "Stereo Imagery",
        description: "Keeps or removes what sits in the middle of a stereo mix, such as a lead voice, bin by bin.",
        params: vec![
            choice("mode", "Mode", &MODES, 0),
            num("amount", "Amount", "%", 0.0, 100.0, 100.0, 1.0),
            num("width", "Center width", "%", 0.0, 100.0, 50.0, 1.0),
            num("smooth", "Smoothing", "%", 0.0, 100.0, 50.0, 1.0),
            num("low", "Lowest frequency", "Hz", 20.0, 2000.0, 20.0, 1.0).log(),
            num(
                "high",
                "Highest frequency",
                "Hz",
                500.0,
                20_000.0,
                20_000.0,
                10.0,
            )
            .log(),
        ],
    }
}

/// How central a bin is, from its two parts. 1 is dead center.
fn centered(l: Complex32, r: Complex32) -> f32 {
    let power = l.norm_sqr() + r.norm_sqr();
    if power <= 1e-20 {
        return 1.0;
    }
    2.0 * (l.re * r.re + l.im * r.im) / power
}

fn smoothstep(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub struct CenterExtract {
    n: usize,
    hop: usize,
    rate: u32,
    win: Vec<f32>,
    fwd: Arc<dyn RealToComplex<f32>>,
    inv: Arc<dyn ComplexToReal<f32>>,
    input: [Vec<f32>; 2],
    acc: [Vec<f32>; 2],
    out: [Vec<f32>; 2],
    pos: usize,
    frame: Vec<f32>,
    spec: [Vec<Complex32>; 2],
    scratch: Vec<Complex32>,
    gains: Vec<f32>,
    keep: bool,
    amount: f32,
    /// Centered value at which a bin starts to count.
    threshold: f32,
    smooth: f32,
    lo_bin: usize,
    hi_bin: usize,
}

impl CenterExtract {
    pub fn new(p: &Params, rate: u32) -> Self {
        let n = (((0.04 * f64::from(rate)) as usize).next_power_of_two()).clamp(256, 8192);
        let hop = n / 4;
        let mut planner = RealFftPlanner::<f32>::new();
        let fwd = planner.plan_fft_forward(n);
        let inv = planner.plan_fft_inverse(n);
        let scratch_len = fwd.get_scratch_len().max(inv.get_scratch_len());
        let win = (0..n)
            .map(|i| (0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / n as f32).cos()).sqrt())
            .collect();
        let bins = n / 2 + 1;
        let mut s = Self {
            n,
            hop,
            rate,
            win,
            fwd,
            inv,
            input: [vec![0.0; n], vec![0.0; n]],
            acc: [vec![0.0; n], vec![0.0; n]],
            out: [vec![0.0; hop], vec![0.0; hop]],
            pos: 0,
            frame: vec![0.0; n],
            spec: [
                vec![Complex32::default(); bins],
                vec![Complex32::default(); bins],
            ],
            scratch: vec![Complex32::default(); scratch_len],
            gains: vec![1.0; bins],
            keep: true,
            amount: 1.0,
            threshold: 0.5,
            smooth: 0.45,
            lo_bin: 0,
            hi_bin: bins,
        };
        s.configure(p);
        s
    }

    fn configure(&mut self, p: &Params) {
        self.keep = p.index("mode") == 0;
        self.amount = (p.get("amount") / 100.0) as f32;
        // Width 0 counts only the nearly identical; width 100 counts anything with some in common.
        self.threshold = (0.98 - 0.78 * p.get("width") / 100.0) as f32;
        self.smooth = (p.get("smooth") / 100.0 * 0.9) as f32;
        let bins = self.n / 2 + 1;
        let bin_of =
            |hz: f64| ((hz * self.n as f64 / f64::from(self.rate)).round() as usize).min(bins);
        self.lo_bin = bin_of(p.get("low"));
        self.hi_bin = bin_of(p.get("high")).max(self.lo_bin);
    }

    fn frame(&mut self) {
        let n = self.n;
        for c in 0..2 {
            for (o, (x, w)) in self
                .frame
                .iter_mut()
                .zip(self.input[c].iter().zip(&self.win))
            {
                *o = x * w;
            }
            let _ = self.fwd.process_with_scratch(
                &mut self.frame,
                &mut self.spec[c],
                &mut self.scratch,
            );
        }
        let span = (1.0 - self.threshold) * 0.5;
        for k in 0..self.gains.len() {
            let target = if k < self.lo_bin || k >= self.hi_bin {
                1.0
            } else {
                let c = smoothstep(
                    (centered(self.spec[0][k], self.spec[1][k]) - self.threshold) / span.max(1e-3),
                );
                if self.keep {
                    1.0 - self.amount * (1.0 - c)
                } else {
                    1.0 - self.amount * c
                }
            };
            self.gains[k] = self.gains[k] * self.smooth + target * (1.0 - self.smooth);
        }
        let scale = 1.0 / (n as f32 * 2.0);
        let hop = self.hop;
        for c in 0..2 {
            for (s, g) in self.spec[c].iter_mut().zip(&self.gains) {
                *s *= *g;
            }
            self.spec[c][0].im = 0.0;
            let last = self.spec[c].len() - 1;
            self.spec[c][last].im = 0.0;
            let _ = self.inv.process_with_scratch(
                &mut self.spec[c],
                &mut self.frame,
                &mut self.scratch,
            );
            for ((a, x), w) in self.acc[c].iter_mut().zip(&self.frame).zip(&self.win) {
                *a += x * w * scale;
            }
            self.out[c].copy_from_slice(&self.acc[c][..hop]);
            self.acc[c].copy_within(hop.., 0);
            self.acc[c][n - hop..].fill(0.0);
            self.input[c].copy_within(hop.., 0);
        }
    }
}

impl Processor for CenterExtract {
    fn process(&mut self, buf: &mut [f32]) {
        let start = self.n - self.hop;
        for f in buf.as_chunks_mut::<2>().0 {
            for (c, v) in f.iter_mut().enumerate() {
                self.input[c][start + self.pos] = *v;
                *v = self.out[c][self.pos];
            }
            self.pos += 1;
            if self.pos == self.hop {
                self.pos = 0;
                self.frame();
            }
        }
    }

    fn latency(&self) -> usize {
        self.n
    }

    fn reset(&mut self) {
        for c in 0..2 {
            self.input[c].fill(0.0);
            self.acc[c].fill(0.0);
            self.out[c].fill(0.0);
        }
        self.gains.fill(1.0);
        self.pos = 0;
    }

    fn update(&mut self, p: &Params) -> bool {
        self.configure(p);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const RATE: u32 = 48_000;

    fn build(set: &[(&str, f64)]) -> CenterExtract {
        let saved: BTreeMap<String, f64> =
            set.iter().map(|(k, v)| ((*k).to_string(), *v)).collect();
        CenterExtract::new(&Params::resolve(&info(), &saved), RATE)
    }

    /// A 1 kHz tone in the center plus a 3 kHz tone only on the left, then a second of audio.
    fn mix() -> Vec<f32> {
        (0..RATE as usize * 2)
            .flat_map(|i| {
                let t = i as f32 / RATE as f32;
                let mid = (std::f32::consts::TAU * 1000.0 * t).sin() * 0.3;
                let side = (std::f32::consts::TAU * 3000.0 * t).sin() * 0.3;
                [mid + side, mid]
            })
            .collect()
    }

    /// Level (peak) of one tone in one channel over the last second, by correlating with sine and cosine.
    fn tone(buf: &[f32], ch: usize, hz: f32) -> f32 {
        let (mut s, mut c, mut n) = (0f32, 0f32, 0f32);
        for i in RATE as usize..RATE as usize * 2 {
            let t = i as f32 / RATE as f32;
            let x = buf[i * 2 + ch];
            s += x * (std::f32::consts::TAU * hz * t).sin();
            c += x * (std::f32::consts::TAU * hz * t).cos();
            n += 1.0;
        }
        2.0 * (s * s + c * c).sqrt() / n
    }

    fn run(mut fx: CenterExtract) -> Vec<f32> {
        let mut b = mix();
        for ch in b.chunks_mut(2 * 1000) {
            fx.process(ch);
        }
        b
    }

    #[test]
    fn keeping_the_center_drops_the_side() {
        let b = run(build(&[]));
        assert!(
            tone(&b, 0, 1000.0) > 0.25,
            "center kept: {}",
            tone(&b, 0, 1000.0)
        );
        assert!(
            tone(&b, 0, 3000.0) < 0.08,
            "side gone: {}",
            tone(&b, 0, 3000.0)
        );
    }

    #[test]
    fn removing_the_center_keeps_the_side() {
        let b = run(build(&[("mode", 1.0)]));
        assert!(
            tone(&b, 0, 1000.0) < 0.08,
            "center gone: {}",
            tone(&b, 0, 1000.0)
        );
        assert!(
            tone(&b, 0, 3000.0) > 0.25,
            "side kept: {}",
            tone(&b, 0, 3000.0)
        );
    }

    #[test]
    fn amount_zero_passes_the_mix_through_delayed() {
        let b = run(build(&[("amount", 0.0)]));
        assert!((tone(&b, 0, 1000.0) - 0.3).abs() < 0.02);
        assert!((tone(&b, 0, 3000.0) - 0.3).abs() < 0.02);
    }

    #[test]
    fn bins_outside_the_range_are_left_alone() {
        // Only 500 Hz to 2 kHz is worked on, so the 3 kHz side stays.
        let b = run(build(&[("low", 500.0), ("high", 2000.0)]));
        assert!(tone(&b, 0, 3000.0) > 0.25);
        assert!(tone(&b, 0, 1000.0) > 0.25);
    }

    #[test]
    fn reports_a_frame_of_latency_and_is_repeatable() {
        let fx = build(&[]);
        assert_eq!(fx.latency(), 2048);
        assert_eq!(run(build(&[])), run(build(&[])));
    }

    #[test]
    fn defaults_sit_inside_their_ranges() {
        for p in info().params {
            assert!(p.default >= p.min && p.default <= p.max, "{}", p.key);
            if p.log {
                assert!(p.min > 0.0, "{}", p.key);
            }
        }
    }
}
