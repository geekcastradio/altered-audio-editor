// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Analysis of the mix: average spectrum, amplitude statistics and a
//! histogram of how loud the samples are. Uses the same render as
//! mixdowns and loudness checks, so the numbers describe what is heard.

use std::sync::atomic::AtomicBool;

use realfft::num_complex::Complex32;
use realfft::RealFftPlanner;
use serde::Serialize;

use super::mixdown::{render_mix, MixdownError, MixdownRequest};
use crate::session::model::Session;

/// FFT size of the frequency analysis.
const FFT: usize = 8192;
/// Points returned for the spectrum, log spaced from `LOW_HZ` up.
const POINTS: usize = 480;
const LOW_HZ: f64 = 20.0;
/// Histogram: this many one-dB bins below full scale.
pub const HIST_BINS: usize = 96;
/// Statistics windows, seconds.
const WINDOW_SECONDS: f64 = 0.05;
/// A sample this close to full scale counts as clipped.
const CLIP_LEVEL: f32 = 0.9999;

/// Level of one side, or of both mixed for the "all" figures.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStats {
    pub min_sample: f32,
    pub max_sample: f32,
    pub peak_db: f64,
    pub rms_db: f64,
    pub dc_offset: f64,
    pub clipped: u64,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisReport {
    pub seconds: f64,
    pub sample_rate: u32,
    pub left: ChannelStats,
    pub right: ChannelStats,
    /// Quietest, average and loudest 50 ms RMS window (of both sides), dB.
    pub min_window_db: f64,
    pub avg_window_db: f64,
    pub max_window_db: f64,
    /// Highest peak over the average level, dB.
    pub crest_db: f64,
    /// Samples that are exactly zero, counted over both sides.
    pub silent_samples: u64,
    /// Samples at each one-dB level, from 0 dB down: bin `i` holds levels
    /// in (-(i+1), -i] dBFS. Both sides together.
    pub histogram: Vec<u64>,
    /// Frequencies and levels (dB) of the average spectrum.
    pub spectrum_hz: Vec<f32>,
    pub spectrum_db: Vec<f32>,
    pub cancelled: bool,
}

fn db(x: f64) -> f64 {
    if x <= 0.0 {
        -200.0
    } else {
        20.0 * x.log10()
    }
}

#[derive(Default)]
struct Side {
    min: f32,
    max: f32,
    sum: f64,
    sum_sq: f64,
    clipped: u64,
}

impl Side {
    fn add(&mut self, x: f32) {
        self.min = self.min.min(x);
        self.max = self.max.max(x);
        self.sum += f64::from(x);
        self.sum_sq += f64::from(x) * f64::from(x);
        if x.abs() >= CLIP_LEVEL {
            self.clipped += 1;
        }
    }

    fn stats(&self, frames: u64) -> ChannelStats {
        let n = frames.max(1) as f64;
        ChannelStats {
            min_sample: self.min,
            max_sample: self.max,
            peak_db: db(f64::from(self.min.abs().max(self.max.abs()))),
            rms_db: db((self.sum_sq / n).sqrt()),
            dc_offset: self.sum / n,
            clipped: self.clipped,
        }
    }
}

/// The spectrum's accumulator: Hann-windowed blocks, powers averaged.
struct Spectrum {
    plan: std::sync::Arc<dyn realfft::RealToComplex<f32>>,
    window: Vec<f32>,
    block: Vec<f32>,
    power: Vec<f64>,
    blocks: u64,
    hold: Vec<f32>,
}

impl Spectrum {
    fn new() -> Self {
        let plan = RealFftPlanner::<f32>::new().plan_fft_forward(FFT);
        let window = (0..FFT)
            .map(|i| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / FFT as f32).cos())
            .collect();
        Self {
            plan,
            window,
            block: vec![0.0; FFT],
            power: vec![0.0; FFT / 2 + 1],
            blocks: 0,
            hold: Vec::new(),
        }
    }

    fn feed(&mut self, mono: f32) {
        self.hold.push(mono);
        if self.hold.len() == FFT {
            self.run();
            // Half-overlapped blocks.
            self.hold.drain(..FFT / 2);
        }
    }

    fn run(&mut self) {
        for ((o, x), w) in self.block.iter_mut().zip(&self.hold).zip(&self.window) {
            *o = x * w;
        }
        let mut out = vec![Complex32::new(0.0, 0.0); FFT / 2 + 1];
        if self.plan.process(&mut self.block, &mut out).is_ok() {
            for (p, c) in self.power.iter_mut().zip(&out) {
                *p += f64::from(c.norm_sqr());
            }
            self.blocks += 1;
        }
    }

    /// Log-spaced points: the loudest bin within each point's span.
    fn points(&self, rate: u32) -> (Vec<f32>, Vec<f32>) {
        if self.blocks == 0 {
            return (Vec::new(), Vec::new());
        }
        let nyquist = f64::from(rate) / 2.0;
        let bin_hz = f64::from(rate) / FFT as f64;
        // A full-scale sine reads about 0 dB: its peak bin's amplitude is
        // N/4 with a Hann window.
        let norm = (FFT as f64 / 4.0).powi(2);
        let level = |bin: usize| {
            let p = self.power[bin] / self.blocks as f64 / norm;
            (10.0 * p.max(1e-20).log10()) as f32
        };
        let top = nyquist.clamp(LOW_HZ * 2.0, 20_000.0);
        let ratio = (top / LOW_HZ).powf(1.0 / (POINTS - 1) as f64);
        let (mut hz, mut dbs) = (Vec::new(), Vec::new());
        let mut f = LOW_HZ;
        for _ in 0..POINTS {
            let lo = ((f / ratio.sqrt()) / bin_hz).floor().max(1.0) as usize;
            let hi = (((f * ratio.sqrt()) / bin_hz).ceil() as usize).min(FFT / 2);
            let v = (lo..=hi.max(lo))
                .map(level)
                .fold(f32::NEG_INFINITY, f32::max);
            hz.push(f as f32);
            dbs.push(v);
            f *= ratio;
        }
        (hz, dbs)
    }
}

/// Analyzes the requested part of the session.
pub fn analyze(
    session: &Session,
    req: &MixdownRequest,
    cancel: &AtomicBool,
    progress: impl FnMut(u64, u64),
) -> Result<AnalysisReport, MixdownError> {
    let rate = session.sample_rate;
    let window = ((WINDOW_SECONDS * f64::from(rate)) as usize).max(1);
    let (mut l, mut r) = (Side::default(), Side::default());
    let mut hist = vec![0u64; HIST_BINS];
    let mut silent = 0u64;
    let mut spectrum = Spectrum::new();
    let (mut w_sum, mut w_n) = (0f64, 0usize);
    let (mut w_min, mut w_max, mut w_total, mut w_count) =
        (f64::INFINITY, f64::NEG_INFINITY, 0f64, 0u64);
    let mut frames = 0u64;
    let rendered = render_mix(session, req, cancel, progress, |block| {
        for f in block.as_chunks::<2>().0 {
            l.add(f[0]);
            r.add(f[1]);
            for x in f {
                let a = x.abs();
                if a == 0.0 {
                    silent += 1;
                } else {
                    let d = -db(f64::from(a));
                    let bin = (d.max(0.0) as usize).min(HIST_BINS - 1);
                    hist[bin] += 1;
                }
            }
            spectrum.feed((f[0] + f[1]) * 0.5);
            w_sum += f64::from(f[0]) * f64::from(f[0]) + f64::from(f[1]) * f64::from(f[1]);
            w_n += 1;
            if w_n == window {
                let level = db((w_sum / (2.0 * window as f64)).sqrt());
                w_min = w_min.min(level);
                w_max = w_max.max(level);
                w_total += level;
                w_count += 1;
                w_sum = 0.0;
                w_n = 0;
            }
            frames += 1;
        }
        Ok(())
    })?;
    if w_count == 0 {
        // Shorter than one window: use what there is.
        let level = db((w_sum / (2.0 * w_n.max(1) as f64)).sqrt());
        w_min = level;
        w_max = level;
        w_total = level;
        w_count = 1;
    }
    let (left, right) = (l.stats(frames), r.stats(frames));
    let all_rms = db(((l.sum_sq + r.sum_sq) / (2.0 * frames.max(1) as f64)).sqrt());
    let (spectrum_hz, spectrum_db) = spectrum.points(rate);
    Ok(AnalysisReport {
        seconds: rendered.frames as f64 / f64::from(rate.max(1)),
        sample_rate: rate,
        crest_db: left.peak_db.max(right.peak_db) - all_rms,
        left,
        right,
        min_window_db: if w_min.is_finite() { w_min } else { -200.0 },
        avg_window_db: w_total / w_count as f64,
        max_window_db: if w_max.is_finite() { w_max } else { -200.0 },
        silent_samples: silent,
        histogram: hist,
        spectrum_hz,
        spectrum_db,
        cancelled: rendered.cancelled,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::model::{new_id, ChannelLayout, MediaRef, Track};
    use crate::session::ops::{apply, ClipSpec, Op};
    use std::path::Path;

    /// A 1 kHz sine at `peak` on the left and a steady 0.1 on the right.
    fn tone(dir: &Path, peak: f32, seconds: u32) -> std::path::PathBuf {
        let path = dir.join("tone.wav");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        for i in 0..48_000 * seconds {
            let t = i as f32 / 48_000.0;
            w.write_sample(peak * (2.0 * std::f32::consts::PI * 1000.0 * t).sin())
                .unwrap();
            w.write_sample(0.1f32).unwrap();
        }
        w.finalize().unwrap();
        path
    }

    fn session_of(path: std::path::PathBuf, frames: u64) -> Session {
        let mut s = Session::new("t", 48_000);
        let m = MediaRef {
            id: new_id(),
            path,
            relative_path: None,
            sample_rate: 48_000,
            channels: 2,
            length: frames,
        };
        let mid = m.id;
        s.media.push(m);
        s.tracks.push(Track::new("A", ChannelLayout::Stereo));
        let tid = s.tracks[0].id;
        apply(
            &mut s,
            &Op::AddClips {
                track_id: tid,
                clips: vec![ClipSpec {
                    id: new_id(),
                    name: "c".into(),
                    media_id: mid,
                    start: 0,
                    source_in: 0,
                    source_out: frames,
                }],
            },
        )
        .unwrap();
        s
    }

    #[test]
    fn reads_levels_dc_spectrum_and_histogram_of_a_known_file() {
        let dir = tempfile::tempdir().unwrap();
        let s = session_of(tone(dir.path(), 0.5, 4), 192_000);
        let r = analyze(
            &s,
            &MixdownRequest::default(),
            &AtomicBool::new(false),
            |_, _| {},
        )
        .unwrap();
        assert!((r.seconds - 4.0).abs() < 1e-6);
        // Left: a -6 dBFS peak sine, RMS 3 dB lower, no DC.
        assert!((r.left.peak_db + 6.02).abs() < 0.1, "{}", r.left.peak_db);
        assert!((r.left.rms_db + 9.03).abs() < 0.1, "{}", r.left.rms_db);
        assert!(r.left.dc_offset.abs() < 1e-3);
        // Right: a steady 0.1 is a 0.1 DC offset at -20 dB.
        assert!((r.right.dc_offset - 0.1).abs() < 1e-4);
        assert!((r.right.rms_db + 20.0).abs() < 0.1);
        assert_eq!(r.left.clipped, 0);
        assert!(r.silent_samples <= 2);
        // The spectrum peaks near 1 kHz at about the sine's mixed level.
        let (i, top) =
            r.spectrum_db
                .iter()
                .enumerate()
                .fold(
                    (0, f32::NEG_INFINITY),
                    |m, (i, v)| if *v > m.1 { (i, *v) } else { m },
                );
        assert!(
            (r.spectrum_hz[i] - 1000.0).abs() < 60.0,
            "{}",
            r.spectrum_hz[i]
        );
        assert!(top > -14.0 && top < -4.0, "{top}");
        // Histogram: every sample of the steady side sits at -20 dB.
        assert_eq!(r.histogram.len(), HIST_BINS);
        assert!(r.histogram[19] > 190_000, "{:?}", &r.histogram[15..25]);
        assert_eq!(
            r.histogram.iter().sum::<u64>() + r.silent_samples,
            2 * 192_000
        );
        assert!(r.max_window_db >= r.avg_window_db && r.avg_window_db >= r.min_window_db);
        assert!(r.crest_db > 0.0);
    }

    #[test]
    fn counts_clipped_samples() {
        let dir = tempfile::tempdir().unwrap();
        let s = session_of(tone(dir.path(), 1.0, 1), 48_000);
        let r = analyze(
            &s,
            &MixdownRequest::default(),
            &AtomicBool::new(false),
            |_, _| {},
        )
        .unwrap();
        assert!(r.left.clipped > 0 && r.left.clipped < 3000);
        assert!(r.left.peak_db > -0.05);
    }
}
