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
/// Three or more samples in a row this close to full scale are a flattened peak.
const FLAT_LEVEL: f32 = 0.999;
const FLAT_RUN: u32 = 3;
/// A click is a jump this many times the recent average jump, and at least this big.
const CLICK_RATIO: f32 = 10.0;
const CLICK_MIN: f32 = 0.08;
/// Problem times listed in the report; the counts are always complete.
const LISTED: usize = 50;

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

/// One tone of a hum: its frequency and how far it stands above the sound around it.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HumTone {
    pub hz: f32,
    pub excess_db: f32,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hum {
    /// 50 or 60: the mains frequency the tones line up with.
    pub base_hz: u32,
    /// The strongest tone's height over its surroundings.
    pub strength_db: f32,
    pub tones: Vec<HumTone>,
}

/// What looks wrong in the audio. Times are seconds from the start of what was analyzed.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Problems {
    /// Sudden jumps far above the surrounding sound.
    pub clicks: u64,
    pub click_times: Vec<f64>,
    /// Flattened peaks: places three or more samples in a row sit at full scale.
    pub clip_regions: u64,
    pub clip_samples: u64,
    pub clip_times: Vec<f64>,
    pub hum: Option<Hum>,
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
    pub problems: Problems,
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

/// Finds clicks and flattened peaks while the audio streams past.
struct Flaws {
    rate: f64,
    frame: u64,
    prev: [f32; 2],
    /// Recent average size of the sample-to-sample change.
    avg: f32,
    alpha: f32,
    last_click: Option<u64>,
    run: u32,
    out: Problems,
}

impl Flaws {
    fn new(rate: u32) -> Self {
        let rate = f64::from(rate.max(1));
        Self {
            rate,
            frame: 0,
            prev: [0.0; 2],
            avg: 0.0,
            alpha: (1.0 / (0.05 * rate)) as f32,
            last_click: None,
            run: 0,
            out: Problems::default(),
        }
    }

    fn secs(&self, frame: u64) -> f64 {
        frame as f64 / self.rate
    }

    fn feed(&mut self, f: [f32; 2]) {
        let step = (f[0] - self.prev[0]).abs().max((f[1] - self.prev[1]).abs());
        self.prev = f;
        // Not before the first tenth of a second: the average has to settle first.
        if self.frame as f64 > self.rate / 10.0
            && step > CLICK_MIN
            && step > CLICK_RATIO * self.avg
            && self
                .last_click
                .is_none_or(|c| (self.frame - c) as f64 > self.rate * 0.003)
        {
            self.out.clicks += 1;
            if self.out.click_times.len() < LISTED {
                self.out.click_times.push(self.secs(self.frame));
            }
            self.last_click = Some(self.frame);
        }
        // A spike must not teach the average that spikes are normal.
        self.avg += self.alpha * (step.min(self.avg * 4.0 + 1e-4) - self.avg);

        if f[0].abs() >= FLAT_LEVEL || f[1].abs() >= FLAT_LEVEL {
            self.run += 1;
            if self.run == FLAT_RUN {
                self.out.clip_regions += 1;
                self.out.clip_samples += u64::from(FLAT_RUN);
                if self.out.clip_times.len() < LISTED {
                    let start = self.frame + 1 - u64::from(FLAT_RUN);
                    self.out.clip_times.push(self.secs(start));
                }
            } else if self.run > FLAT_RUN {
                self.out.clip_samples += 1;
            }
        } else {
            self.run = 0;
        }
        self.frame += 1;
    }
}

impl Spectrum {
    /// Mains hum: tones at 50 or 60 Hz and their multiples standing well above the sound
    /// around them. Needs at least two such tones, so one bass note is not taken for hum.
    fn hum(&self, rate: u32) -> Option<Hum> {
        if self.blocks == 0 {
            return None;
        }
        let bin_hz = f64::from(rate) / FFT as f64;
        let nyquist = f64::from(rate) / 2.0;
        let power = |b: usize| self.power.get(b).copied().unwrap_or(0.0);
        let mut best: Option<(f32, Hum)> = None;
        for base in [50u32, 60] {
            let mut tones = Vec::new();
            for h in 1..=4u32 {
                let f = f64::from(base * h);
                if f + 40.0 >= nyquist {
                    break;
                }
                let c = (f / bin_hz).round() as usize;
                let peak = (c.saturating_sub(1)..=c + 1).map(power).fold(0.0, f64::max);
                // The level around the tone: the middle of the bins 10 to 30 Hz either side.
                let a = (((f - 30.0) / bin_hz).ceil() as usize).max(1);
                let b = ((f - 10.0) / bin_hz).floor() as usize;
                let c2 = ((f + 10.0) / bin_hz).ceil() as usize;
                let d = ((f + 30.0) / bin_hz).floor() as usize;
                let mut around: Vec<f64> =
                    (a..=b.max(a)).chain(c2..=d.max(c2)).map(power).collect();
                around.sort_by(f64::total_cmp);
                let floor = around
                    .get(around.len() / 2)
                    .copied()
                    .unwrap_or(0.0)
                    .max(1e-20);
                let excess = (10.0 * (peak.max(1e-20) / floor).log10()) as f32;
                tones.push(HumTone {
                    hz: f as f32,
                    excess_db: excess,
                });
            }
            let strong: Vec<f32> = tones
                .iter()
                .map(|t| t.excess_db)
                .filter(|e| *e >= 8.0)
                .collect();
            if strong.len() >= 2 {
                let score: f32 = strong.iter().sum();
                let strength = strong.iter().copied().fold(0.0, f32::max);
                if best.as_ref().is_none_or(|(s, _)| score > *s) {
                    best = Some((
                        score,
                        Hum {
                            base_hz: base,
                            strength_db: strength,
                            tones,
                        },
                    ));
                }
            }
        }
        best.map(|(_, h)| h)
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
    let mut flaws = Flaws::new(rate);
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
            flaws.feed([f[0], f[1]]);
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
        problems: Problems {
            hum: spectrum.hum(rate),
            ..flaws.out
        },
        cancelled: rendered.cancelled,
    })
}

/// The sample nearest `at` where the sound crosses zero, looking up to `reach` samples
/// either way. A crossing sits between two samples with opposite signs (or on an exact
/// zero); the one of the pair closer to zero is returned. None when the sound never
/// crosses inside the reach, such as silence or a held offset.
pub fn nearest_zero_crossing(mono: &[f32], at: usize, reach: usize) -> Option<usize> {
    if mono.is_empty() {
        return None;
    }
    let at = at.min(mono.len() - 1);
    let crosses = |i: usize| -> Option<usize> {
        let v = mono[i];
        if v == 0.0 {
            // A zero between two non-zero samples is a crossing only when it is not a flat run.
            return Some(i);
        }
        let prev = *mono.get(i.checked_sub(1)?)?;
        if prev != 0.0 && (prev < 0.0) != (v < 0.0) {
            Some(if prev.abs() < v.abs() { i - 1 } else { i })
        } else {
            None
        }
    };
    // Silence is not a crossing: ask for at least some movement around the point.
    let moving = |i: usize| {
        let lo = i.saturating_sub(2);
        let hi = (i + 2).min(mono.len() - 1);
        mono[lo..=hi].iter().any(|v| *v != 0.0)
    };
    for d in 0..=reach {
        for i in [at.checked_sub(d), Some(at + d)].into_iter().flatten() {
            if i < mono.len() && moving(i) {
                if let Some(c) = crosses(i) {
                    return Some(c);
                }
            }
        }
    }
    None
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

    #[test]
    fn zero_crossings_are_found_nearest_first() {
        // A 100 Hz sine at 8 kHz crosses zero every 40 samples, near 0, 40, 80...
        let s: Vec<f32> = (0..400)
            .map(|i| (i as f32 * 100.0 * std::f32::consts::TAU / 8_000.0 + 0.3).sin())
            .collect();
        for at in [57, 100, 213] {
            let c = nearest_zero_crossing(&s, at, 60).unwrap();
            assert!(c.abs_diff(at) <= 21, "{at} -> {c}");
            let flips = |i: usize| (s[i] < 0.0) != (s[i + 1] < 0.0);
            assert!(
                s[c].abs() < 0.2 && (flips(c) || (c > 0 && flips(c - 1))),
                "{at} -> {c}"
            );
        }
        assert_eq!(nearest_zero_crossing(&[0.0; 50], 25, 40), None, "silence");
        assert_eq!(
            nearest_zero_crossing(&[0.5; 50], 25, 40),
            None,
            "held offset"
        );
        assert_eq!(nearest_zero_crossing(&[], 0, 10), None);
    }

    /// Both sides play `f(sample index)`, `seconds` long.
    fn signal(dir: &Path, seconds: u32, f: impl Fn(usize) -> f32) -> Session {
        let path = dir.join("sig.wav");
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        for i in 0..48_000 * seconds as usize {
            let v = f(i);
            w.write_sample(v).unwrap();
            w.write_sample(v).unwrap();
        }
        w.finalize().unwrap();
        session_of(path, 48_000 * u64::from(seconds))
    }

    fn problems(s: &Session) -> Problems {
        analyze(
            s,
            &MixdownRequest::default(),
            &AtomicBool::new(false),
            |_, _| {},
        )
        .unwrap()
        .problems
    }

    fn sine(i: usize, hz: f32, amp: f32) -> f32 {
        amp * (std::f32::consts::TAU * hz * i as f32 / 48_000.0).sin()
    }

    /// Repeatable noise between -1 and 1.
    fn noise(i: usize) -> f32 {
        let mut z = (i as u64).wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 11) as f32 / (1u64 << 52) as f32 - 1.0
    }

    #[test]
    fn finds_a_click_and_not_a_clean_tone() {
        let dir = tempfile::tempdir().unwrap();
        let clean = problems(&signal(dir.path(), 3, |i| sine(i, 220.0, 0.1)));
        assert_eq!(clean.clicks, 0);
        let s = signal(dir.path(), 3, |i| {
            sine(i, 220.0, 0.1) + if i == 96_000 { 0.7 } else { 0.0 }
        });
        let p = problems(&s);
        assert_eq!(p.clicks, 1, "{p:?}");
        assert!((p.click_times[0] - 2.0).abs() < 0.01, "{:?}", p.click_times);
    }

    #[test]
    fn finds_flattened_peaks() {
        let dir = tempfile::tempdir().unwrap();
        // 100 Hz at 1.6 times full scale and cut off: two flat tops per cycle.
        let s = signal(dir.path(), 1, |i| sine(i, 100.0, 1.6).clamp(-1.0, 1.0));
        let p = problems(&s);
        assert!(p.clip_regions >= 190 && p.clip_regions <= 210, "{p:?}");
        assert!(p.clip_samples > 3 * p.clip_regions);
        assert_eq!(
            p.clip_times.len(),
            LISTED,
            "the list is capped, the count is not"
        );
        let none = problems(&signal(dir.path(), 1, |i| sine(i, 100.0, 0.5)));
        assert_eq!(none.clip_regions, 0);
    }

    #[test]
    fn finds_mains_hum_at_50_or_60_and_not_plain_noise() {
        let dir = tempfile::tempdir().unwrap();
        let hum = |base: f32| {
            move |i: usize| {
                sine(i, base, 0.02)
                    + sine(i, base * 2.0, 0.015)
                    + sine(i, base * 3.0, 0.01)
                    + noise(i) * 0.01
            }
        };
        let fifty = problems(&signal(dir.path(), 4, hum(50.0)));
        let h = fifty.hum.as_ref().expect("50 Hz hum");
        assert_eq!(h.base_hz, 50, "{h:?}");
        assert!(h.strength_db > 10.0, "{h:?}");
        let sixty = problems(&signal(dir.path(), 4, hum(60.0)));
        assert_eq!(
            sixty.hum.as_ref().map(|h| h.base_hz),
            Some(60),
            "{:?}",
            sixty.hum
        );
        let plain = problems(&signal(dir.path(), 4, |i| noise(i) * 0.05));
        assert_eq!(plain.hum, None, "{:?}", plain.hum);
        // One bass note is not hum.
        let bass = problems(&signal(dir.path(), 4, |i| {
            sine(i, 50.0, 0.2) + noise(i) * 0.01
        }));
        assert_eq!(bass.hum, None, "{:?}", bass.hum);
    }
}
