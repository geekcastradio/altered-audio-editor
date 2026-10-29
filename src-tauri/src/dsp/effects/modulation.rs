// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Modulation effects: Chorus, Flanger and Phaser. All original code from
//! textbook techniques: a short delay whose time is swept by a slow
//! low-frequency oscillator (chorus and flanger), and a chain of all-pass
//! filters whose corner is swept the same way (phaser).

use std::f64::consts::TAU;

use super::{num, EffectInfo, Params, Processor};

/// The longest delay any of these needs, in seconds.
const RING_SECONDS: f64 = 0.12;

pub fn chorus_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.chorus",
        name: "Chorus",
        category: "Modulation",
        description: "Thickens a sound by mixing in slightly detuned, slightly late copies of it, as if several people were speaking or playing together.",
        params: vec![
            num("voices", "Voices", "", 1.0, 6.0, 3.0, 1.0),
            num("delay", "Delay", "ms", 5.0, 50.0, 20.0, 0.5),
            num("depth", "Depth", "%", 0.0, 100.0, 40.0, 1.0),
            num("rate", "Rate", "Hz", 0.05, 5.0, 0.8, 0.01).log(),
            num("spread", "Stereo spread", "%", 0.0, 100.0, 60.0, 1.0),
            num("mix", "Mix", "%", 0.0, 100.0, 40.0, 1.0),
        ],
    }
}

pub fn flanger_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.flanger",
        name: "Flanger",
        category: "Modulation",
        description: "A swooshing, jet-like sweep made by mixing the sound with a very short, moving delay of itself. Feedback makes it more pronounced.",
        params: vec![
            num("delay", "Delay", "ms", 0.1, 10.0, 1.5, 0.1),
            num("depth", "Depth", "%", 0.0, 100.0, 60.0, 1.0),
            num("rate", "Rate", "Hz", 0.05, 10.0, 0.3, 0.01).log(),
            num("feedback", "Feedback", "%", -95.0, 95.0, 50.0, 1.0),
            num("phase", "Stereo phase", "deg", 0.0, 180.0, 90.0, 1.0),
            num("mix", "Mix", "%", 0.0, 100.0, 50.0, 1.0),
        ],
    }
}

pub fn phaser_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.phaser",
        name: "Phaser",
        category: "Modulation",
        description: "A softer, swirling sweep made by shifting the timing of some pitches against others. More stages make more notches.",
        params: vec![
            num("stages", "Stages", "", 2.0, 12.0, 6.0, 2.0),
            num("center", "Center", "Hz", 100.0, 4000.0, 800.0, 1.0).log(),
            num("depth", "Depth", "%", 0.0, 100.0, 70.0, 1.0),
            num("rate", "Rate", "Hz", 0.05, 10.0, 0.4, 0.01).log(),
            num("feedback", "Feedback", "%", 0.0, 90.0, 40.0, 1.0),
            num("phase", "Stereo phase", "deg", 0.0, 180.0, 90.0, 1.0),
            num("mix", "Mix", "%", 0.0, 100.0, 50.0, 1.0),
        ],
    }
}

/// A sine oscillator that counts in cycles.
#[derive(Clone, Copy)]
struct Lfo {
    phase: f64,
    step: f64,
}

impl Lfo {
    fn new(hz: f64, rate: u32) -> Self {
        Self {
            phase: 0.0,
            step: hz / f64::from(rate),
        }
    }

    /// The sine at this phase plus `offset` cycles.
    fn at(&self, offset: f64) -> f64 {
        ((self.phase + offset) * TAU).sin()
    }

    fn advance(&mut self) {
        self.phase = (self.phase + self.step).fract();
    }
}

/// One channel's delay line, read between frames.
struct Line {
    ring: Vec<f32>,
    at: usize,
}

impl Line {
    fn new(rate: u32) -> Self {
        Self {
            ring: vec![0.0; (RING_SECONDS * f64::from(rate)) as usize + 4],
            at: 0,
        }
    }

    /// The sound `delay` frames ago (at least 1).
    fn read(&self, delay: f64) -> f32 {
        let n = self.ring.len();
        let delay = delay.clamp(1.0, (n - 2) as f64);
        let pos = self.at as f64 + n as f64 - delay;
        let whole = pos.floor();
        let frac = (pos - whole) as f32;
        let i0 = whole as usize % n;
        let i1 = (i0 + 1) % n;
        self.ring[i0] * (1.0 - frac) + self.ring[i1] * frac
    }

    fn push(&mut self, x: f32) {
        self.ring[self.at] = if x.abs() < 1e-20 { 0.0 } else { x };
        self.at = (self.at + 1) % self.ring.len();
    }

    fn reset(&mut self) {
        self.ring.fill(0.0);
        self.at = 0;
    }
}

// --- Chorus ---

pub struct Chorus {
    rate: f64,
    lines: [Line; 2],
    lfo: Lfo,
    voices: usize,
    base: f64,
    swing: f64,
    spread: f64,
    mix: f32,
}

impl Chorus {
    pub fn new(p: &Params, rate: u32) -> Self {
        let delay = p.get("delay");
        // The sweep can reach nearly the whole delay, never past it.
        let swing = p.get("depth") / 100.0 * delay * 0.9;
        Self {
            rate: f64::from(rate),
            lines: [Line::new(rate), Line::new(rate)],
            lfo: Lfo::new(p.get("rate"), rate),
            voices: p.get("voices").round().clamp(1.0, 6.0) as usize,
            base: delay,
            swing,
            spread: p.get("spread") / 100.0,
            mix: (p.get("mix") / 100.0) as f32,
        }
    }
}

impl Processor for Chorus {
    fn process(&mut self, buf: &mut [f32]) {
        let ms = self.rate / 1000.0;
        for f in buf.as_chunks_mut::<2>().0 {
            let mut wet = [0f32; 2];
            for (c, w) in wet.iter_mut().enumerate() {
                for v in 0..self.voices {
                    // Voices are spread around the cycle; the right side leads
                    // or lags the left by a quarter cycle at full spread.
                    let off = v as f64 / self.voices as f64 + c as f64 * 0.25 * self.spread;
                    let d = (self.base + self.swing * self.lfo.at(off)) * ms;
                    *w += self.lines[c].read(d);
                }
                *w /= self.voices as f32;
            }
            self.lines[0].push(f[0]);
            self.lines[1].push(f[1]);
            f[0] = f[0] * (1.0 - self.mix) + wet[0] * self.mix;
            f[1] = f[1] * (1.0 - self.mix) + wet[1] * self.mix;
            self.lfo.advance();
        }
    }

    fn reset(&mut self) {
        for l in &mut self.lines {
            l.reset();
        }
        self.lfo.phase = 0.0;
    }
}

// --- Flanger ---

pub struct Flanger {
    rate: f64,
    lines: [Line; 2],
    lfo: Lfo,
    base: f64,
    swing: f64,
    feedback: f32,
    offset: f64,
    mix: f32,
}

impl Flanger {
    pub fn new(p: &Params, rate: u32) -> Self {
        let delay = p.get("delay");
        Self {
            rate: f64::from(rate),
            lines: [Line::new(rate), Line::new(rate)],
            lfo: Lfo::new(p.get("rate"), rate),
            base: delay,
            swing: p.get("depth") / 100.0 * (delay - 0.05).max(0.0),
            feedback: (p.get("feedback") / 100.0) as f32,
            offset: p.get("phase") / 360.0,
            mix: (p.get("mix") / 100.0) as f32,
        }
    }
}

impl Processor for Flanger {
    fn process(&mut self, buf: &mut [f32]) {
        let ms = self.rate / 1000.0;
        for f in buf.as_chunks_mut::<2>().0 {
            let mut wet = [0f32; 2];
            for (c, w) in wet.iter_mut().enumerate() {
                let off = c as f64 * self.offset;
                // Sweep from the base delay down toward zero and back.
                let d = (self.base - self.swing * (0.5 + 0.5 * self.lfo.at(off))) * ms;
                *w = self.lines[c].read(d.max(1.0));
            }
            for (c, x) in f.iter_mut().enumerate() {
                self.lines[c].push(*x + self.feedback * wet[c]);
                *x = *x * (1.0 - self.mix) + wet[c] * self.mix;
            }
            self.lfo.advance();
        }
    }

    fn reset(&mut self) {
        for l in &mut self.lines {
            l.reset();
        }
        self.lfo.phase = 0.0;
    }
}

// --- Phaser ---

/// A first-order all-pass section.
#[derive(Clone, Copy, Default)]
struct Allpass {
    x1: f32,
    y1: f32,
}

impl Allpass {
    fn run(&mut self, x: f32, a: f32) -> f32 {
        let y = a * x + self.x1 - a * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }
}

pub struct Phaser {
    rate: f64,
    stages: usize,
    sections: [[Allpass; 12]; 2],
    lfo: Lfo,
    center: f64,
    depth: f64,
    feedback: f32,
    offset: f64,
    mix: f32,
    coef: [f32; 2],
    fb: [f32; 2],
    count: u32,
}

impl Phaser {
    pub fn new(p: &Params, rate: u32) -> Self {
        Self {
            rate: f64::from(rate),
            stages: (p.get("stages").round() as usize).clamp(2, 12),
            sections: [[Allpass::default(); 12]; 2],
            lfo: Lfo::new(p.get("rate"), rate),
            center: p.get("center"),
            depth: p.get("depth") / 100.0,
            feedback: (p.get("feedback") / 100.0) as f32,
            offset: p.get("phase") / 360.0,
            mix: (p.get("mix") / 100.0) as f32,
            coef: [0.0; 2],
            fb: [0.0; 2],
            count: 0,
        }
    }

    /// The all-pass coefficient for a corner frequency.
    fn coef_for(&self, hz: f64) -> f32 {
        let t = (std::f64::consts::PI * hz.min(self.rate * 0.45) / self.rate).tan();
        ((t - 1.0) / (t + 1.0)) as f32
    }
}

impl Processor for Phaser {
    fn process(&mut self, buf: &mut [f32]) {
        for f in buf.as_chunks_mut::<2>().0 {
            // The corner moves in octaves, so the sweep sounds even.
            if self.count == 0 {
                for c in 0..2 {
                    let l = self.lfo.at(c as f64 * self.offset);
                    let hz = self.center * (self.depth * l * 2.0).exp2();
                    self.coef[c] = self.coef_for(hz);
                }
            }
            self.count = (self.count + 1) % 8;
            for (c, out) in f.iter_mut().enumerate() {
                let dry = *out;
                let mut x = dry + self.feedback * self.fb[c];
                for s in &mut self.sections[c][..self.stages] {
                    x = s.run(x, self.coef[c]);
                }
                self.fb[c] = if x.abs() < 1e-20 { 0.0 } else { x };
                *out = dry * (1.0 - self.mix) + x * self.mix;
            }
            self.lfo.advance();
        }
    }

    fn reset(&mut self) {
        self.sections = [[Allpass::default(); 12]; 2];
        self.fb = [0.0; 2];
        self.lfo.phase = 0.0;
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::super::build;
    use super::super::tests::slot;
    use std::f32::consts::PI;

    fn sine(freq: f32, amp: f32, secs: f32) -> Vec<f32> {
        let n = (48_000.0 * secs) as usize;
        (0..n)
            .flat_map(|i| {
                let v = (2.0 * PI * freq * i as f32 / 48_000.0).sin() * amp;
                [v, v]
            })
            .collect()
    }

    fn peak(b: &[f32]) -> f32 {
        b.iter().fold(0f32, |m, s| m.max(s.abs()))
    }

    fn run(effect: &str, params: &[(&str, f64)], mut b: Vec<f32>) -> Vec<f32> {
        let mut p = build(&slot(effect, params), 48_000).unwrap();
        for chunk in b.chunks_mut(2 * 777) {
            p.process(chunk);
        }
        b
    }

    #[test]
    fn with_no_mix_each_one_passes_the_sound_untouched() {
        for fx in ["altaudio.chorus", "altaudio.flanger", "altaudio.phaser"] {
            let input = sine(440.0, 0.5, 0.3);
            let out = run(fx, &[("mix", 0.0)], input.clone());
            assert_eq!(out, input, "{fx}");
        }
    }

    #[test]
    fn flanger_with_a_fixed_delay_makes_a_comb() {
        // A 0.5 ms delay cancels 1 kHz (half a cycle) and doubles 2 kHz.
        let set = [
            ("delay", 0.5),
            ("depth", 0.0),
            ("feedback", 0.0),
            ("mix", 50.0),
        ];
        let notch = run("altaudio.flanger", &set, sine(1000.0, 0.5, 0.5));
        assert!(peak(&notch[notch.len() / 2..]) < 0.03, "1 kHz cancels");
        let boost = run("altaudio.flanger", &set, sine(2000.0, 0.5, 0.5));
        assert!(peak(&boost[boost.len() / 2..]) > 0.47, "2 kHz stays");
    }

    #[test]
    fn phaser_notches_its_center_when_it_is_not_moving() {
        // Two all-pass stages turn the center frequency half a cycle, so
        // an even mix with the dry sound cancels it.
        let set = [
            ("stages", 2.0),
            ("center", 1000.0),
            ("depth", 0.0),
            ("feedback", 0.0),
            ("mix", 50.0),
        ];
        let at = run("altaudio.phaser", &set, sine(1000.0, 0.5, 0.5));
        assert!(
            peak(&at[at.len() / 2..]) < 0.03,
            "{}",
            peak(&at[at.len() / 2..])
        );
        let far = run("altaudio.phaser", &set, sine(100.0, 0.5, 0.5));
        assert!(
            peak(&far[far.len() / 2..]) > 0.4,
            "far from the notch stays"
        );
    }

    #[test]
    fn chorus_changes_the_sound_but_keeps_its_level() {
        let input = sine(300.0, 0.5, 1.0);
        let out = run("altaudio.chorus", &[("mix", 50.0)], input.clone());
        let tail = &out[out.len() / 2..];
        let diff = tail
            .iter()
            .zip(&input[input.len() / 2..])
            .fold(0f32, |m, (a, b)| m.max((a - b).abs()));
        assert!(diff > 0.02, "the copies differ from the original");
        assert!(peak(tail) < 0.6, "and the level stays about the same");
    }

    #[test]
    fn silence_stays_silent_with_feedback() {
        for fx in ["altaudio.flanger", "altaudio.phaser"] {
            let out = run(fx, &[("feedback", 90.0)], vec![0.0; 2 * 48_000]);
            assert_eq!(peak(&out), 0.0, "{fx}");
        }
    }
}
