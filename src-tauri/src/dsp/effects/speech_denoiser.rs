// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Speech Denoiser: takes steady and fluctuating background noise out of
//! speech with RNNoise, a small recurrent neural network trained on voice
//! and noise (the nnnoiseless crate, a Rust port of Xiph.Org's RNNoise,
//! BSD-3-Clause; the notice is in docs/THIRD-PARTY-nnnoiseless.md). It
//! works on 48 kHz audio in 10 ms frames, so other session rates are
//! resampled in and back out. It adds 10 ms of delay plus the resampler's,
//! which the renderer compensates for, and it keeps the original timing.
//! Each channel is cleaned on its own.

use std::collections::VecDeque;

use audioadapter_buffers::owned::InterleavedOwned;
use deep_filter_rt::tract::DfTract;
use ndarray::Array2;
use nnnoiseless::DenoiseState;
use rubato::{Async, FixedAsync, Resampler, SincInterpolationParameters};

use super::{num, EffectInfo, Params, Processor};

/// RNNoise works at this rate, in frames of [`FRAME`] samples.
const RATE: u32 = 48_000;
const FRAME: usize = DenoiseState::FRAME_SIZE;
/// RNNoise expects sample values on the 16-bit scale.
const SCALE: f32 = 32768.0;

pub const ID: &str = "altaudio.speech_denoiser";
/// The larger network: DeepFilterNet3 (MIT or Apache-2.0, see docs/THIRD-PARTY-deepfilternet.md).
pub const DEEP_ID: &str = "altaudio.speech_denoiser_deep";

pub fn info() -> EffectInfo {
    EffectInfo {
        id: ID,
        name: "Speech Denoiser",
        category: "Restoration",
        description: "Removes background noise from speech: room hiss, fans, hum and traffic. Lower Amount if the voice sounds thin or watery; at 100 % it removes the most noise. Works on one voice at a time, so run it on each speaker's track.",
        params: vec![num("amount", "Amount", "%", 0.0, 100.0, 80.0, 1.0)],
    }
}

pub fn deep_info() -> EffectInfo {
    EffectInfo {
        id: DEEP_ID,
        name: "Speech Denoiser (High Quality)",
        category: "Restoration",
        description: "A larger network than Speech Denoiser, for harder noise: keyboards, traffic, a second voice far away. It uses more of the processor and adds about 40 ms of delay, which the session makes up for. Lower Amount if the voice sounds thin. Works on one voice at a time, so run it on each speaker's track.",
        params: vec![num("amount", "Amount", "%", 0.0, 100.0, 100.0, 1.0)],
    }
}

/// The network that cleans each 10 ms frame.
enum Engine {
    /// RNNoise: small and quick.
    Rnn(Box<DenoiseState<'static>>),
    /// DeepFilterNet3.
    Deep(Box<Deep>),
}

struct Deep {
    df: DfTract,
    input: Array2<f32>,
    output: Array2<f32>,
    /// Samples held back between input and output, at 48 kHz.
    delay: usize,
}

impl Deep {
    fn new() -> Self {
        let df = DfTract::default();
        let shape = (df.ch, df.hop_size);
        Self {
            delay: df.delay(),
            df,
            input: Array2::zeros(shape),
            output: Array2::zeros(shape),
        }
    }
}

impl Engine {
    /// Cleans one 10 ms frame at 48 kHz (sample values from -1 to 1).
    fn frame(&mut self, input: &[f32; FRAME], output: &mut [f32; FRAME]) {
        match self {
            Engine::Rnn(state) => {
                let mut scaled = [0.0f32; FRAME];
                for (o, i) in scaled.iter_mut().zip(input) {
                    *o = i * SCALE;
                }
                state.process_frame(output, &scaled);
                for v in output.iter_mut() {
                    *v /= SCALE;
                }
            }
            Engine::Deep(d) => {
                for (o, i) in d.input.row_mut(0).iter_mut().zip(input) {
                    *o = *i;
                }
                if d.df.process(d.input.view(), d.output.view_mut()).is_err() {
                    output.copy_from_slice(input);
                    return;
                }
                for (o, i) in output.iter_mut().zip(d.output.row(0)) {
                    *o = *i;
                }
            }
        }
    }
}

/// One channel's path: resample to 48 kHz, clean, resample back.
struct Lane {
    engine: Engine,
    down: Option<Resampler48>,
    up: Option<Resampler48>,
    /// Input at the session rate waiting for the first resampler.
    pending: Vec<f32>,
    /// Audio at 48 kHz waiting for RNNoise.
    at48: Vec<f32>,
    /// Cleaned audio at 48 kHz waiting for the second resampler.
    clean48: Vec<f32>,
    /// Finished audio at the session rate, ready to hand back.
    out: VecDeque<f32>,
    /// The untouched input, delayed to line up with `out`, for the dry mix.
    dry: VecDeque<f32>,
}

struct Resampler48 {
    r: Async<f32>,
    /// Output samples to drop so the delay line's start does not show.
    skip: usize,
}

impl Resampler48 {
    fn new(ratio: f64, chunk: usize) -> Option<Self> {
        let r = Async::<f32>::new_sinc(
            ratio,
            1.0,
            &SincInterpolationParameters::default(),
            chunk,
            1,
            FixedAsync::Input,
        )
        .ok()?;
        let skip = r.output_delay();
        Some(Self { r, skip })
    }

    /// Resamples whole chunks from `queue`, appending to `out`.
    fn run(&mut self, queue: &mut Vec<f32>, out: &mut Vec<f32>) {
        loop {
            let need = self.r.input_frames_next();
            if queue.len() < need {
                return;
            }
            let chunk: Vec<f32> = queue.drain(..need).collect();
            let Ok(input) = InterleavedOwned::new_from(chunk, 1, need) else {
                return;
            };
            let Ok(result) = self.r.process(&input, None) else {
                return;
            };
            let data = result.take_data();
            let drop = self.skip.min(data.len());
            self.skip -= drop;
            out.extend_from_slice(&data[drop..]);
        }
    }
}

impl Lane {
    /// Samples at 48 kHz the network holds audio back between input and output.
    fn held(&self) -> usize {
        match &self.engine {
            Engine::Deep(d) => d.delay,
            Engine::Rnn(_) => FRAME,
        }
    }

    fn new(rate: u32, priming: usize, deep: bool) -> Option<Self> {
        let chunk_in = (rate as usize / 100).max(16);
        let (down, up) = if rate == RATE {
            (None, None)
        } else {
            (
                Some(Resampler48::new(
                    f64::from(RATE) / f64::from(rate),
                    chunk_in,
                )?),
                Some(Resampler48::new(f64::from(rate) / f64::from(RATE), FRAME)?),
            )
        };
        let mut lane = Self {
            engine: if deep {
                Engine::Deep(Box::new(Deep::new()))
            } else {
                Engine::Rnn(DenoiseState::new())
            },
            down,
            up,
            pending: Vec::new(),
            at48: Vec::new(),
            clean48: Vec::new(),
            out: VecDeque::new(),
            dry: VecDeque::new(),
        };
        // Silence up front keeps the output from ever running dry between chunks.
        lane.out.resize(priming, 0.0);
        Some(lane)
    }

    /// Pushes `input` through the chain, adding whatever finished audio comes out.
    fn feed(&mut self, input: &[f32]) {
        match self.down.as_mut() {
            Some(d) => {
                self.pending.extend_from_slice(input);
                d.run(&mut self.pending, &mut self.at48);
            }
            None => self.at48.extend_from_slice(input),
        }
        let mut frame = [0.0f32; FRAME];
        let mut cleaned = [0.0f32; FRAME];
        while self.at48.len() >= FRAME {
            for (o, i) in frame.iter_mut().zip(self.at48.drain(..FRAME)) {
                *o = i;
            }
            self.engine.frame(&frame, &mut cleaned);
            self.clean48.extend_from_slice(&cleaned);
        }
        match self.up.as_mut() {
            Some(u) => {
                let mut produced = Vec::new();
                u.run(&mut self.clean48, &mut produced);
                self.out.extend(produced);
            }
            None => self.out.extend(self.clean48.drain(..)),
        }
    }

    /// Feeds `input` and returns the same number of finished samples.
    fn run(&mut self, input: &[f32], amount: f32, delay: usize) -> Vec<f32> {
        self.feed(input);
        // The dry copy is held back by the same delay as the cleaned audio.
        let mut res = Vec::with_capacity(input.len());
        for &x in input {
            self.dry.push_back(x);
            let dry = if self.dry.len() > delay {
                self.dry.pop_front().unwrap_or(0.0)
            } else {
                0.0
            };
            let wet = self.out.pop_front().unwrap_or(0.0);
            res.push(dry + (wet - dry) * amount);
        }
        res
    }
}

/// How many samples of silence must lead the output so it never runs dry:
/// the most the chain's finished audio ever trails what has gone in, found
/// by feeding it silence one sample at a time (the worst case).
fn startup_need(rate: u32) -> Option<usize> {
    let mut lane = Lane::new(rate, 0, false)?;
    let span = rate as usize / 100 * 12;
    let mut need = 0usize;
    for t in 1..=span {
        lane.feed(&[0.0]);
        need = need.max(t.saturating_sub(lane.out.len()));
    }
    Some(need + 8)
}

pub struct SpeechDenoiser {
    lanes: [Lane; 2],
    amount: f32,
    latency: usize,
    priming: usize,
    rate: u32,
    deep: bool,
}

impl SpeechDenoiser {
    /// The quick network (RNNoise).
    pub fn new(p: &Params, rate: u32) -> Option<Self> {
        Self::with_engine(p, rate, false)
    }

    /// The larger network (DeepFilterNet3).
    pub fn deep(p: &Params, rate: u32) -> Option<Self> {
        Self::with_engine(p, rate, true)
    }

    fn with_engine(p: &Params, rate: u32, deep: bool) -> Option<Self> {
        let rate = rate.clamp(8_000, 384_000);
        let priming = startup_need(rate)?;
        let a = Lane::new(rate, priming, deep)?;
        let b = Lane::new(rate, priming, deep)?;
        // The resamplers drop their own filter delay, so what is left is the
        // priming silence and what the network holds back (one frame for the
        // quick one, its whole look-ahead for the larger), at the session rate.
        let held = a.held();
        let fr = held as f64 * f64::from(rate) / f64::from(RATE);
        let latency = priming + fr.round() as usize;
        Some(Self {
            lanes: [a, b],
            amount: (p.get("amount") / 100.0) as f32,
            latency,
            priming,
            rate,
            deep,
        })
    }

    fn split(buf: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut l = Vec::with_capacity(buf.len() / 2);
        let mut r = Vec::with_capacity(buf.len() / 2);
        for f in buf.as_chunks::<2>().0 {
            l.push(f[0]);
            r.push(f[1]);
        }
        (l, r)
    }
}

impl Processor for SpeechDenoiser {
    fn process(&mut self, buf: &mut [f32]) {
        if buf.len() < 2 {
            return;
        }
        let (l, r) = Self::split(buf);
        let delay = self.latency;
        let lo = self.lanes[0].run(&l, self.amount, delay);
        let ro = self.lanes[1].run(&r, self.amount, delay);
        for (i, f) in buf.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            f[0] = lo[i];
            f[1] = ro[i];
        }
    }

    fn latency(&self) -> usize {
        self.latency
    }

    fn reset(&mut self) {
        if let (Some(a), Some(b)) = (
            Lane::new(self.rate, self.priming, self.deep),
            Lane::new(self.rate, self.priming, self.deep),
        ) {
            self.lanes = [a, b];
        }
    }

    fn update(&mut self, p: &Params) -> bool {
        self.amount = (p.get("amount") / 100.0) as f32;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn params(amount: f64) -> Params {
        let saved: BTreeMap<String, f64> = [("amount".to_string(), amount)].into_iter().collect();
        Params::resolve(&info(), &saved)
    }

    /// A voice-like tone: a 140 Hz buzz with harmonics, swelling at 3 Hz.
    fn voice(frames: usize, rate: u32) -> Vec<f32> {
        let tau = 2.0 * std::f32::consts::PI;
        (0..frames)
            .map(|i| {
                let t = i as f32 / rate as f32;
                let env = 0.5 + 0.5 * (tau * 3.0 * t).sin();
                let v: f32 = (1..25)
                    .map(|h| (tau * 140.0 * h as f32 * t).sin() / h as f32)
                    .sum();
                v * env * 0.12
            })
            .collect()
    }

    fn noise(n: usize, level: f32, mut seed: u32) -> Vec<f32> {
        (0..n)
            .map(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                ((seed >> 8) as f32 / 8_388_608.0 - 1.0) * level
            })
            .collect()
    }

    fn stereo(l: &[f32], r: &[f32]) -> Vec<f32> {
        l.iter().zip(r).flat_map(|(a, b)| [*a, *b]).collect()
    }

    fn run_blocks(d: &mut SpeechDenoiser, input: &[f32], block_frames: usize) -> Vec<f32> {
        let mut out = input.to_vec();
        for c in out.chunks_mut(block_frames * 2) {
            d.process(c);
        }
        out
    }

    fn rms(v: &[f32]) -> f32 {
        (v.iter().map(|x| x * x).sum::<f32>() / v.len().max(1) as f32).sqrt()
    }

    fn left(buf: &[f32]) -> Vec<f32> {
        buf.as_chunks::<2>().0.iter().map(|f| f[0]).collect()
    }

    /// The lag, in frames, at which `b` best matches `a`.
    fn lag(a: &[f32], b: &[f32], from: usize, span: i32) -> i32 {
        let n = a.len().min(b.len()) as i32;
        let c = |l: i32| -> f32 {
            (from as i32..n - span - 2)
                .map(|i| a[i as usize] * b[(i + l) as usize])
                .sum()
        };
        (0..=span)
            .max_by(|x, y| c(*x).partial_cmp(&c(*y)).unwrap())
            .unwrap()
    }

    #[test]
    fn dry_path_is_delayed_by_the_reported_latency() {
        for rate in [48_000u32, 44_100] {
            let mut d = SpeechDenoiser::new(&params(0.0), rate).unwrap();
            let x = voice(rate as usize * 2, rate);
            let input = stereo(&x, &x);
            let out = run_blocks(&mut d, &input, 512);
            let l = d.latency();
            let y = left(&out);
            for i in (l + 100)..(x.len() - 100) {
                assert!((y[i] - x[i - l]).abs() < 1e-6, "rate {rate} frame {i}");
            }
        }
    }

    #[test]
    fn cleaned_audio_lines_up_with_the_reported_latency() {
        for rate in [48_000u32, 44_100, 32_000, 96_000] {
            let mut d = SpeechDenoiser::new(&params(100.0), rate).unwrap();
            let x = voice(rate as usize * 4, rate);
            let out = run_blocks(&mut d, &stereo(&x, &x), 480);
            let y = left(&out);
            let got = lag(&x, &y, rate as usize, 6 * rate as i32 / 100);
            let want = d.latency() as i32;
            assert!(
                (got - want).abs() <= 6,
                "rate {rate}: measured {got}, reported {want}"
            );
        }
    }

    #[test]
    fn block_size_does_not_change_the_result() {
        for rate in [48_000u32, 44_100] {
            let x = voice(rate as usize, rate);
            let n = noise(x.len(), 0.01, 5);
            let sum: Vec<f32> = x.iter().zip(&n).map(|(a, b)| a + b).collect();
            let input = stereo(&sum, &sum);
            let mut a = SpeechDenoiser::new(&params(100.0), rate).unwrap();
            let mut b = SpeechDenoiser::new(&params(100.0), rate).unwrap();
            let big = run_blocks(&mut a, &input, 4096);
            let small = run_blocks(&mut b, &input, 97);
            for (u, v) in big.iter().zip(&small) {
                assert!((u - v).abs() < 1e-4, "rate {rate}");
            }
        }
    }

    #[test]
    fn steady_noise_goes_down() {
        for rate in [48_000u32, 44_100] {
            let n = noise(rate as usize * 3, 0.05, 9);
            let mut d = SpeechDenoiser::new(&params(100.0), rate).unwrap();
            let out = run_blocks(&mut d, &stereo(&n, &n), 480);
            let y = left(&out);
            assert!(
                rms(&y[rate as usize..]) < rms(&n[rate as usize..]) * 0.6,
                "rate {rate}"
            );
        }
    }

    #[test]
    fn tiny_and_odd_blocks_are_safe() {
        let mut d = SpeechDenoiser::new(&params(100.0), 44_100).unwrap();
        let x = voice(44_100, 44_100);
        let input = stereo(&x, &x);
        for size in [1usize, 2, 3, 7, 441, 1000] {
            let _ = run_blocks(&mut d, &input[..size * 2 * 20], size);
        }
        let mut empty: [f32; 0] = [];
        d.process(&mut empty);
    }

    #[test]
    fn reset_and_update_work() {
        let mut d = SpeechDenoiser::new(&params(100.0), 48_000).unwrap();
        assert!(d.update(&params(50.0)));
        d.reset();
        let mut buf = vec![0.0f32; 960];
        d.process(&mut buf);
        assert!(buf.iter().all(|v| v.abs() < 1e-6));
    }

    fn deep_params(amount: f64) -> Params {
        let saved: BTreeMap<String, f64> = [("amount".to_string(), amount)].into_iter().collect();
        Params::resolve(&deep_info(), &saved)
    }

    #[test]
    fn deep_dry_path_is_delayed_by_the_reported_latency() {
        for rate in [48_000u32, 44_100] {
            let mut d = SpeechDenoiser::deep(&deep_params(0.0), rate).unwrap();
            let x = voice(rate as usize, rate);
            let out = run_blocks(&mut d, &stereo(&x, &x), 512);
            let l = d.latency();
            assert!(l > 1_800, "latency {l}");
            let y = left(&out);
            for i in (l + 100)..(x.len() - 100) {
                assert!((y[i] - x[i - l]).abs() < 1e-6, "rate {rate} frame {i}");
            }
        }
    }

    /// With the network's attenuation limited to nothing it passes audio
    /// through, held back by its delay: a way to measure the whole chain's
    /// lag against the latency the effect reports.
    #[test]
    fn deep_audio_lines_up_with_the_reported_latency() {
        for rate in [48_000u32, 44_100, 96_000] {
            let mut d = SpeechDenoiser::deep(&deep_params(100.0), rate).unwrap();
            for lane in d.lanes.iter_mut() {
                if let Engine::Deep(deep) = &mut lane.engine {
                    deep.df.set_atten_lim(0.0);
                }
            }
            let x = noise(rate as usize * 3, 0.2, 21);
            let out = run_blocks(&mut d, &stereo(&x, &x), 480);
            let y = left(&out);
            let got = lag(&x, &y, rate as usize, 8 * rate as i32 / 100);
            let want = d.latency() as i32;
            assert!(
                (got - want).abs() <= 8,
                "rate {rate}: measured {got}, reported {want}"
            );
        }
    }

    #[test]
    fn deep_block_size_does_not_change_the_result() {
        let rate = 44_100;
        let x = voice(rate as usize, rate);
        let n = noise(x.len(), 0.01, 5);
        let sum: Vec<f32> = x.iter().zip(&n).map(|(a, b)| a + b).collect();
        let input = stereo(&sum, &sum);
        let mut a = SpeechDenoiser::deep(&deep_params(100.0), rate).unwrap();
        let mut b = SpeechDenoiser::deep(&deep_params(100.0), rate).unwrap();
        let big = run_blocks(&mut a, &input, 4096);
        let small = run_blocks(&mut b, &input, 97);
        for (u, v) in big.iter().zip(&small) {
            assert!((u - v).abs() < 1e-4);
        }
    }

    #[test]
    fn deep_steady_noise_goes_down() {
        let rate = 48_000;
        let n = noise(rate as usize * 3, 0.05, 9);
        let mut d = SpeechDenoiser::deep(&deep_params(100.0), rate).unwrap();
        let out = run_blocks(&mut d, &stereo(&n, &n), 480);
        let y = left(&out);
        assert!(rms(&y[rate as usize..]) < rms(&n[rate as usize..]) * 0.5);
    }

    #[test]
    fn deep_reset_update_and_silence_are_safe() {
        let mut d = SpeechDenoiser::deep(&deep_params(100.0), 48_000).unwrap();
        assert!(d.update(&deep_params(50.0)));
        d.reset();
        let mut buf = vec![0.0f32; 4800];
        d.process(&mut buf);
        assert!(buf.iter().all(|v| v.abs() < 1e-6));
        let mut empty: [f32; 0] = [];
        d.process(&mut empty);
    }
}
