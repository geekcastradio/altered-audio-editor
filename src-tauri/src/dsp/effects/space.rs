// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Room Reverb (an algorithmic reverb) and Tape Delay (an analog-style
//! delay). Both are original code from textbook methods: the reverb is a
//! bank of parallel damped feedback combs into a few all-pass diffusers
//! (the Schroeder layout, with comb lengths in the same spread the classic
//! designs use); the delay is a feedback delay line with a low-pass,
//! soft saturation and a slow wobble in the loop.

use std::f64::consts::TAU;

use super::{num, EffectInfo, Params, Processor};

pub fn room_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.room-reverb",
        name: "Room Reverb",
        category: "Delay and Reverb",
        description: "A built-in reverb that needs no impulse file. Size sets how big the space feels, Decay how long it rings, Damping how fast the highs die away.",
        params: vec![
            num("size", "Size", "%", 20.0, 200.0, 100.0, 1.0),
            num("decay", "Decay", "s", 0.2, 12.0, 1.8, 0.1).log(),
            num("predelay", "Pre-delay", "ms", 0.0, 200.0, 15.0, 1.0),
            num("damping", "Damping", "%", 0.0, 100.0, 40.0, 1.0),
            num("width", "Width", "%", 0.0, 100.0, 100.0, 1.0),
            num("mix", "Mix", "%", 0.0, 100.0, 25.0, 1.0),
        ],
    }
}

pub fn tape_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.tape-delay",
        name: "Tape Delay",
        category: "Delay and Reverb",
        description: "An echo that gets darker, dirtier and a little wobbly each time it repeats, like an old tape or bucket-brigade delay.",
        params: vec![
            num("time", "Time", "ms", 20.0, 1500.0, 350.0, 1.0).log(),
            num("feedback", "Feedback", "%", 0.0, 95.0, 40.0, 1.0),
            num("tone", "Tone", "Hz", 500.0, 16000.0, 3500.0, 1.0).log(),
            num("saturation", "Saturation", "%", 0.0, 100.0, 30.0, 1.0),
            num("wobble", "Wobble", "%", 0.0, 100.0, 20.0, 1.0),
            num("offset", "Right side time", "%", 50.0, 150.0, 100.0, 1.0),
            num("mix", "Mix", "%", 0.0, 100.0, 30.0, 1.0),
        ],
    }
}

// --- Room Reverb ---

/// Comb and all-pass lengths for 44.1 kHz, in frames.
const COMBS: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS: [usize; 4] = [556, 441, 341, 225];
/// How much longer the right side's lines are, in frames at 44.1 kHz.
const SPREAD: usize = 23;

struct Comb {
    buf: Vec<f32>,
    at: usize,
    g: f32,
    /// Input scale that keeps the loudness the same at any decay.
    k: f32,
    damp: f32,
    held: f32,
}

impl Comb {
    fn new(len: usize, rate: u32, decay: f64, damp: f32) -> Self {
        let secs = len as f64 / f64::from(rate);
        let g = 10f64.powf(-3.0 * secs / decay).min(0.9995);
        Self {
            buf: vec![0.0; len.max(2)],
            at: 0,
            g: g as f32,
            k: ((1.0 - g * g) / COMBS.len() as f64).sqrt() as f32,
            damp,
            held: 0.0,
        }
    }

    fn run(&mut self, x: f32) -> f32 {
        let out = self.buf[self.at];
        self.held = out * (1.0 - self.damp) + self.held * self.damp;
        if self.held.abs() < 1e-20 {
            self.held = 0.0;
        }
        self.buf[self.at] = x * self.k + self.held * self.g;
        self.at = (self.at + 1) % self.buf.len();
        out
    }
}

struct Diffuser {
    buf: Vec<f32>,
    at: usize,
}

impl Diffuser {
    fn new(len: usize) -> Self {
        Self {
            buf: vec![0.0; len.max(2)],
            at: 0,
        }
    }

    fn run(&mut self, x: f32) -> f32 {
        let held = self.buf[self.at];
        self.buf[self.at] = x + held * 0.5;
        self.at = (self.at + 1) % self.buf.len();
        held - x * 0.5
    }
}

pub struct Room {
    combs: [Vec<Comb>; 2],
    diffusers: [Vec<Diffuser>; 2],
    pre: Vec<f32>,
    pre_at: usize,
    wet1: f32,
    wet2: f32,
    mix: f32,
}

impl Room {
    pub fn new(p: &Params, rate: u32) -> Self {
        let scale = f64::from(rate) / 44_100.0 * p.get("size") / 100.0;
        let decay = p.get("decay").max(0.05);
        // More damping means a stronger low-pass in each loop.
        let damp = (p.get("damping") / 100.0 * 0.85) as f32;
        let side = |extra: usize| -> (Vec<Comb>, Vec<Diffuser>) {
            (
                COMBS
                    .iter()
                    .map(|&n| Comb::new(((n + extra) as f64 * scale) as usize, rate, decay, damp))
                    .collect(),
                ALLPASS
                    .iter()
                    .map(|&n| Diffuser::new(((n + extra) as f64 * scale) as usize))
                    .collect(),
            )
        };
        let (cl, dl) = side(0);
        let (cr, dr) = side(SPREAD);
        let width = (p.get("width") / 100.0) as f32;
        Self {
            combs: [cl, cr],
            diffusers: [dl, dr],
            pre: vec![0.0; (p.get("predelay") / 1000.0 * f64::from(rate)) as usize + 1],
            pre_at: 0,
            wet1: width / 2.0 + 0.5,
            wet2: (1.0 - width) / 2.0,
            mix: (p.get("mix") / 100.0) as f32,
        }
    }
}

impl Processor for Room {
    fn process(&mut self, buf: &mut [f32]) {
        for f in buf.as_chunks_mut::<2>().0 {
            let input = (f[0] + f[1]) * 0.5;
            let x = self.pre[self.pre_at];
            self.pre[self.pre_at] = input;
            self.pre_at = (self.pre_at + 1) % self.pre.len();
            let mut wet = [0f32; 2];
            for (c, w) in wet.iter_mut().enumerate() {
                let mut s: f32 = self.combs[c].iter_mut().map(|k| k.run(x)).sum();
                for d in &mut self.diffusers[c] {
                    s = d.run(s);
                }
                *w = s;
            }
            let l = wet[0] * self.wet1 + wet[1] * self.wet2;
            let r = wet[1] * self.wet1 + wet[0] * self.wet2;
            f[0] = f[0] * (1.0 - self.mix) + l * self.mix;
            f[1] = f[1] * (1.0 - self.mix) + r * self.mix;
        }
    }

    fn reset(&mut self) {
        for side in &mut self.combs {
            for c in side {
                c.buf.fill(0.0);
                c.held = 0.0;
            }
        }
        for side in &mut self.diffusers {
            for d in side {
                d.buf.fill(0.0);
            }
        }
        self.pre.fill(0.0);
    }
}

// --- Tape Delay ---

pub struct Tape {
    rings: [Vec<f32>; 2],
    at: usize,
    times: [f64; 2],
    feedback: f32,
    coef: f32,
    tone: [f32; 2],
    drive: f32,
    wobble: f64,
    phase: f64,
    step: f64,
    mix: f32,
}

impl Tape {
    pub fn new(p: &Params, rate: u32) -> Self {
        let time = p.get("time") / 1000.0 * f64::from(rate);
        let right = time * p.get("offset") / 100.0;
        // Room for the longest time plus the wobble's reach.
        let len = (1.5 * 1.5 * f64::from(rate)) as usize + 64;
        let hz = p.get("tone").min(f64::from(rate) * 0.45);
        Self {
            rings: [vec![0.0; len], vec![0.0; len]],
            at: 0,
            times: [time, right],
            feedback: (p.get("feedback") / 100.0) as f32,
            coef: (1.0 - (-TAU * hz / f64::from(rate)).exp()) as f32,
            tone: [0.0; 2],
            drive: 1.0 + (p.get("saturation") / 100.0 * 3.0) as f32,
            // Up to about a quarter of a percent of speed, 6 ms of drift at most.
            wobble: p.get("wobble") / 100.0 * 0.004 * f64::from(rate),
            phase: 0.0,
            step: 0.7 / f64::from(rate),
            mix: (p.get("mix") / 100.0) as f32,
        }
    }

    fn read(ring: &[f32], at: usize, delay: f64) -> f32 {
        let n = ring.len();
        let delay = delay.clamp(1.0, (n - 2) as f64);
        let pos = at as f64 + n as f64 - delay;
        let whole = pos.floor();
        let frac = (pos - whole) as f32;
        let i0 = whole as usize % n;
        ring[i0] * (1.0 - frac) + ring[(i0 + 1) % n] * frac
    }
}

impl Processor for Tape {
    fn process(&mut self, buf: &mut [f32]) {
        let n = self.rings[0].len();
        for f in buf.as_chunks_mut::<2>().0 {
            // A slow sine plus a faster, smaller one, like a worn transport.
            let drift =
                ((self.phase * TAU).sin() + 0.3 * (self.phase * 7.3 * TAU).sin()) * self.wobble;
            self.phase = (self.phase + self.step).fract();
            #[allow(clippy::needless_range_loop)]
            for c in 0..2 {
                let echo = Self::read(&self.rings[c], self.at, self.times[c] + drift);
                self.tone[c] += self.coef * (echo - self.tone[c]);
                if self.tone[c].abs() < 1e-20 {
                    self.tone[c] = 0.0;
                }
                // Unity gain for small sounds, bounded for big ones.
                let soft = (self.tone[c] * self.drive).tanh() / self.drive;
                let x = f[c];
                self.rings[c][self.at] = x + soft * self.feedback;
                f[c] = x * (1.0 - self.mix) + soft * self.mix;
            }
            self.at = (self.at + 1) % n;
        }
    }

    fn reset(&mut self) {
        for r in &mut self.rings {
            r.fill(0.0);
        }
        self.tone = [0.0; 2];
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::super::build;
    use super::super::tests::slot;

    fn run(effect: &str, params: &[(&str, f64)], mut b: Vec<f32>) -> Vec<f32> {
        let mut p = build(&slot(effect, params), 48_000).unwrap();
        for chunk in b.chunks_mut(2 * 777) {
            p.process(chunk);
        }
        b
    }

    fn noise(frames: usize) -> Vec<f32> {
        let mut s = 12345u32;
        (0..frames * 2)
            .map(|_| {
                s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                ((s >> 8) as f32 / 8_388_608.0 - 1.0) * 0.3
            })
            .collect()
    }

    fn impulse(frames: usize) -> Vec<f32> {
        let mut v = vec![0.0; frames * 2];
        v[0] = 1.0;
        v[1] = 1.0;
        v
    }

    fn rms(b: &[f32]) -> f32 {
        (b.iter().map(|x| x * x).sum::<f32>() / b.len() as f32).sqrt()
    }

    #[test]
    fn with_no_mix_both_pass_the_sound_untouched() {
        for fx in ["altaudio.room-reverb", "altaudio.tape-delay"] {
            let input = noise(4000);
            assert_eq!(run(fx, &[("mix", 0.0)], input.clone()), input, "{fx}");
        }
    }

    #[test]
    fn the_reverb_rings_for_about_its_decay_and_keeps_its_level() {
        let set = [("mix", 100.0), ("predelay", 0.0), ("decay", 1.0)];
        let wet = run("altaudio.room-reverb", &set, noise(48_000 * 3));
        // Steady noise in gives roughly the same level out.
        let level = rms(&wet[2 * 48_000..2 * 96_000]);
        assert!(level > 0.1 && level < 0.25, "level {level}");
        // A click rings, then dies: much quieter a decay later than just after.
        let tail = run("altaudio.room-reverb", &set, impulse(48_000 * 3));
        let early = rms(&tail[2 * 2000..2 * 6000]);
        let late = rms(&tail[2 * 96_000..2 * 100_000]);
        assert!(early > 0.0 && late < early * 0.03, "{early} {late}");
        assert!(tail.iter().all(|s| s.is_finite()));
    }

    #[test]
    fn a_longer_decay_rings_longer_and_zero_width_is_mono() {
        let rest = |d: f64| {
            let t = run(
                "altaudio.room-reverb",
                &[("mix", 100.0), ("predelay", 0.0), ("decay", d)],
                impulse(48_000 * 2),
            );
            rms(&t[2 * 60_000..2 * 70_000])
        };
        assert!(rest(4.0) > rest(0.5) * 5.0);
        let mono = run(
            "altaudio.room-reverb",
            &[("mix", 100.0), ("width", 0.0)],
            impulse(20_000),
        );
        assert!(mono.chunks(2).all(|f| (f[0] - f[1]).abs() < 1e-6));
    }

    #[test]
    fn the_delay_repeats_after_its_time_and_fades() {
        let set = [
            ("time", 100.0),
            ("feedback", 50.0),
            ("tone", 16000.0),
            ("saturation", 0.0),
            ("wobble", 0.0),
            ("mix", 100.0),
        ];
        let out = run("altaudio.tape-delay", &set, impulse(48_000));
        let at = |ms: usize| {
            let c = ms * 48;
            out[2 * (c - 30)..2 * (c + 30)]
                .iter()
                .fold(0f32, |m, s| m.max(s.abs()))
        };
        let (a, b, c) = (at(100), at(200), at(300));
        assert!(a > 0.2 && b < a && c < b && b > 0.05, "{a} {b} {c}");
    }

    #[test]
    fn the_delay_stays_bounded_at_high_feedback_with_saturation() {
        let out = run(
            "altaudio.tape-delay",
            &[("feedback", 95.0), ("saturation", 100.0), ("mix", 100.0)],
            noise(48_000 * 4),
        );
        assert!(out.iter().all(|s| s.is_finite() && s.abs() < 4.0));
    }
}
