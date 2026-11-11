// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! True-peak limiting, for the Limiter effect's True peak switch.
//!
//! Keeps the peaks a digital-to-analog converter would reconstruct, not just
//! the stored samples, under a ceiling. Peaks are estimated with an 8x
//! oversampled look at the signal, the gain is lowered ahead of each peak (so
//! the audio is delayed by `latency()` frames), and it recovers with a
//! release time. All channels share one gain so the stereo image stays put.

use std::collections::VecDeque;
use std::f64::consts::PI;

/// Half-width of the interpolation filter, in samples.
const HALF: usize = 24;
/// How many interpolated points are checked between two samples.
const PHASES: usize = 7;

/// Settings for the limiter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LimiterSettings {
    /// Highest allowed true peak, in dBTP (zero or below).
    pub ceiling_db: f32,
    /// How long the gain takes to come back after a peak, in milliseconds.
    pub release_ms: f32,
    /// How far ahead the limiter looks, in milliseconds.
    pub lookahead_ms: f32,
}

impl Default for LimiterSettings {
    fn default() -> Self {
        Self {
            ceiling_db: -1.0,
            release_ms: 100.0,
            lookahead_ms: 1.5,
        }
    }
}

impl LimiterSettings {
    /// Brings every value into the range the limiter accepts.
    pub fn clamped(self) -> Self {
        Self {
            ceiling_db: self.ceiling_db.clamp(-24.0, 0.0),
            release_ms: self.release_ms.clamp(10.0, 1000.0),
            lookahead_ms: self.lookahead_ms.clamp(0.5, 5.0),
        }
    }
}

fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let q = x * x / 4.0;
    for k in 1..60 {
        term *= q / (k as f64 * k as f64);
        sum += term;
        if term < 1e-14 * sum {
            break;
        }
    }
    sum
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

/// Windowed-sinc weights for the point `frac` of the way between two samples,
/// over the `2 * half` samples around it.
pub fn interpolation_taps(frac: f64, half: usize) -> Vec<f32> {
    let beta = 8.0;
    let norm = bessel_i0(beta);
    (0..2 * half)
        .map(|i| {
            let d = i as f64 - (half as f64 - 1.0) - frac;
            let r = d / half as f64;
            let w = if r.abs() >= 1.0 {
                0.0
            } else {
                bessel_i0(beta * (1.0 - r * r).sqrt()) / norm
            };
            (sinc(d) * w) as f32
        })
        .collect()
}

/// The limiter. Feed it one frame at a time with `push`, or a whole file with
/// `process_offline`.
pub struct TruePeakLimiter {
    channels: usize,
    ceiling: f32,
    look: usize,
    release: f64,
    taps: [Vec<f32>; PHASES],
    ring: Vec<Vec<f32>>,
    mask: usize,
    k: usize,
    window: VecDeque<(usize, f32)>,
    previous: f32,
    released: f64,
    avg: Vec<f64>,
    avg_pos: usize,
    avg_sum: f64,
}

impl TruePeakLimiter {
    /// Makes a limiter for `channels` channels at `sample_rate`.
    pub fn new(channels: usize, sample_rate: u32, settings: LimiterSettings) -> Self {
        let s = settings.clamped();
        let sr = sample_rate.max(8000) as f64;
        let look = ((s.lookahead_ms as f64 / 1000.0 * sr).round() as usize).max(HALF);
        let release = 1.0 - (-1.0 / (s.release_ms as f64 / 1000.0 * sr)).exp();
        let taps: [Vec<f32>; PHASES] =
            std::array::from_fn(|j| interpolation_taps((j + 1) as f64 / (PHASES + 1) as f64, HALF));
        let latency = HALF + look - 1;
        let size = (latency + 2 * HALF + 8).next_power_of_two();
        let mut lim = Self {
            channels: channels.max(1),
            ceiling: 10f32.powf(s.ceiling_db / 20.0),
            look,
            release,
            taps,
            ring: vec![vec![0.0; size]; channels.max(1)],
            mask: size - 1,
            k: 0,
            window: VecDeque::new(),
            previous: 1.0,
            released: 1.0,
            avg: vec![1.0; look],
            avg_pos: 0,
            avg_sum: look as f64,
        };
        lim.reset();
        lim
    }

    /// Back to the start: no gain reduction, silence in the delay line.
    pub fn reset(&mut self) {
        for ch in &mut self.ring {
            ch.fill(0.0);
        }
        self.k = 0;
        self.window.clear();
        self.previous = 1.0;
        self.released = 1.0;
        self.avg.fill(1.0);
        self.avg_pos = 0;
        self.avg_sum = self.look as f64;
        // Prime with silence so the first real frame comes out `latency` frames later.
        let silence = vec![0.0; self.channels];
        let mut out = vec![0.0; self.channels];
        for _ in 0..self.latency() {
            self.push(&silence, &mut out);
        }
    }

    /// Frames of delay between what goes in and what comes out.
    pub fn latency(&self) -> usize {
        HALF + self.look - 1
    }

    fn at(&self, c: usize, i: i64) -> f32 {
        if i < 0 {
            0.0
        } else {
            self.ring[c][i as usize & self.mask]
        }
    }

    /// The gain needed so the true peak around sample `n` stays under the ceiling.
    fn required_gain(&self, n: i64) -> f32 {
        let mut peak = 0.0f32;
        for c in 0..self.channels {
            peak = peak.max(self.at(c, n).abs());
            for taps in &self.taps {
                let mut acc = 0.0f32;
                for (i, t) in taps.iter().enumerate() {
                    acc += self.at(c, n - HALF as i64 + 1 + i as i64) * t;
                }
                peak = peak.max(acc.abs());
            }
        }
        // The short interpolation filter reads slightly low near half the sample rate.
        let peak = peak * 1.02;
        if peak > self.ceiling {
            self.ceiling / peak
        } else {
            1.0
        }
    }

    /// Takes one frame (one value per channel) and writes the delayed, limited frame.
    pub fn push(&mut self, frame: &[f32], out: &mut [f32]) {
        let k = self.k;
        self.k += 1;
        for c in 0..self.channels {
            self.ring[c][k & self.mask] = frame.get(c).copied().unwrap_or(0.0);
        }
        let n = k as i64 - HALF as i64;
        let here = if n >= 0 { self.required_gain(n) } else { 1.0 };
        // A peak between two samples limits the gain at both of them.
        let want = here.min(self.previous);
        self.previous = here;
        while self.window.back().is_some_and(|&(_, v)| v >= want) {
            self.window.pop_back();
        }
        self.window.push_back((k, want));
        while self
            .window
            .front()
            .is_some_and(|&(i, _)| i + self.look <= k)
        {
            self.window.pop_front();
        }
        let lowest = self.window.front().map_or(1.0, |&(_, v)| v) as f64;
        self.released = if lowest < self.released {
            lowest
        } else {
            self.released + (lowest - self.released) * self.release
        };
        self.avg_sum += self.released - self.avg[self.avg_pos];
        self.avg[self.avg_pos] = self.released;
        self.avg_pos = (self.avg_pos + 1) % self.look;
        let gain = (self.avg_sum / self.look as f64) as f32;
        let latency = self.latency();
        for (c, o) in out.iter_mut().enumerate().take(self.channels) {
            *o = if k >= latency {
                self.ring[c][(k - latency) & self.mask] * gain
            } else {
                0.0
            };
        }
    }

    /// Limits whole channels at once and returns them lined up with the input.
    #[cfg(test)]
    pub fn process_offline(&mut self, input: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let len = input.first().map_or(0, Vec::len);
        let latency = self.latency();
        let mut result = vec![Vec::with_capacity(len); self.channels];
        let mut frame = vec![0.0; self.channels];
        let mut out = vec![0.0; self.channels];
        for i in 0..len + latency {
            for (c, f) in frame.iter_mut().enumerate() {
                *f = if i < len {
                    input.get(c).map_or(0.0, |ch| ch[i])
                } else {
                    0.0
                };
            }
            self.push(&frame, &mut out);
            if i >= latency {
                for c in 0..self.channels {
                    result[c].push(out[c]);
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent true-peak meter: 16x oversampling with a longer filter.
    fn true_peak(x: &[f32], from: usize) -> f32 {
        let half = 96usize;
        let taps: Vec<Vec<f32>> = (1..16)
            .map(|j| interpolation_taps(j as f64 / 16.0, half))
            .collect();
        let mut peak = 0.0f32;
        for n in from.max(half)..x.len().saturating_sub(half + 1) {
            peak = peak.max(x[n].abs());
            for t in &taps {
                let mut acc = 0.0f32;
                for (i, w) in t.iter().enumerate() {
                    acc += x[n + 1 + i - half] * w;
                }
                peak = peak.max(acc.abs());
            }
        }
        peak
    }

    fn db(x: f32) -> f32 {
        20.0 * x.log10()
    }

    /// Noise filtered like real program material, with little left near half the sample rate,
    /// scaled so its loudest sample is `amp`.
    fn noise(len: usize, amp: f32) -> Vec<f32> {
        let mut s = 12345u32;
        let mut x: Vec<f32> = (0..len)
            .map(|_| {
                s = s.wrapping_mul(1664525).wrapping_add(1013904223);
                (s >> 8) as f32 / 8388608.0 - 1.0
            })
            .collect();
        for _ in 0..3 {
            let mut y = 0.0f32;
            for v in x.iter_mut() {
                y += (*v - y) * 0.6;
                *v = y;
            }
        }
        let top = x.iter().fold(0.0f32, |a, b| a.max(b.abs())).max(1e-9);
        x.iter().map(|v| v / top * amp).collect()
    }

    #[test]
    fn quiet_audio_passes_unchanged_and_lines_up() {
        let x: Vec<f32> = (0..48000).map(|i| 0.1 * (i as f32 * 0.05).sin()).collect();
        let mut lim = TruePeakLimiter::new(1, 48000, LimiterSettings::default());
        let y = lim.process_offline(std::slice::from_ref(&x));
        assert_eq!(y[0].len(), x.len());
        let worst = x
            .iter()
            .zip(&y[0])
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(worst < 1e-6, "worst difference {worst}");
    }

    #[test]
    fn catches_a_peak_that_falls_between_samples() {
        // A tone at a quarter of the sample rate, 45 degrees off, stores samples
        // near 1.06 but reconstructs to a 1.5 peak.
        let x: Vec<f32> = (0..96000)
            .map(|i| {
                1.5 * (std::f32::consts::FRAC_PI_2 * i as f32 + std::f32::consts::FRAC_PI_4).sin()
            })
            .collect();
        assert!(true_peak(&x, 2000) > 1.45);
        let settings = LimiterSettings {
            ceiling_db: -1.0,
            ..Default::default()
        };
        let mut lim = TruePeakLimiter::new(1, 48000, settings);
        let y = lim.process_offline(&[x]);
        let tp = db(true_peak(&y[0], 4000));
        assert!(tp <= -0.85, "true peak {tp} dBTP is above the ceiling");
        assert!(tp > -1.6, "limited far more than needed: {tp} dBTP");
    }

    #[test]
    fn keeps_loud_noise_under_the_ceiling_on_every_channel() {
        let left = noise(96000, 1.9);
        let right = noise(96000, 1.4);
        let mut lim = TruePeakLimiter::new(
            2,
            44100,
            LimiterSettings {
                ceiling_db: -2.0,
                ..Default::default()
            },
        );
        let y = lim.process_offline(&[left, right]);
        for ch in &y {
            let tp = db(true_peak(ch, 2000));
            assert!(tp <= -1.8, "true peak {tp} dBTP is above -2");
        }
    }

    #[test]
    fn gain_comes_back_after_a_loud_burst() {
        let mut x = vec![0.0f32; 48000 * 2];
        for (i, v) in x.iter_mut().enumerate() {
            let tone = (i as f32 * 0.05).sin();
            *v = if i < 4800 { 1.8 * tone } else { 0.1 * tone };
        }
        let mut lim = TruePeakLimiter::new(
            1,
            48000,
            LimiterSettings {
                release_ms: 100.0,
                ..Default::default()
            },
        );
        let y = lim.process_offline(&[x.clone()]);
        let last = x.len() - 1000..x.len();
        let worst = x[last.clone()]
            .iter()
            .zip(&y[0][last])
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst < 2e-3,
            "gain did not recover, worst difference {worst}"
        );
    }

    #[test]
    fn reports_its_delay_and_keeps_an_impulse_in_place() {
        let mut x = vec![0.0f32; 4000];
        x[1000] = 0.5;
        let mut lim = TruePeakLimiter::new(1, 48000, LimiterSettings::default());
        let y = lim.process_offline(&[x]);
        let at = y[0]
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .unwrap()
            .0;
        assert_eq!(at, 1000);
        assert!(lim.latency() > 0);
    }

    #[test]
    fn settings_are_pulled_into_range() {
        let s = LimiterSettings {
            ceiling_db: 6.0,
            release_ms: 1.0,
            lookahead_ms: 50.0,
        }
        .clamped();
        assert_eq!(s.ceiling_db, 0.0);
        assert_eq!(s.release_ms, 10.0);
        assert_eq!(s.lookahead_ms, 5.0);
    }
}
