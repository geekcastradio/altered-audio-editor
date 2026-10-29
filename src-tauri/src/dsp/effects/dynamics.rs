// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Dynamics effects: Dynamics (a compressor), Limiter, Breath Gate,
//! De-Esser and Speech Leveler. All original code built from textbook
//! techniques: envelope followers, a gain curve with a soft knee, and a
//! look-ahead window for the limiter.

use std::collections::VecDeque;

use super::{db_to_gain, num, toggle, EffectInfo, Params, Processor};
use crate::dsp::biquad::{Coeffs, Shape, StereoBiquad};

/// Smoothing coefficient for a time constant in milliseconds.
fn coef(ms: f64, rate: u32) -> f32 {
    let n = ms.max(0.01) * f64::from(rate) / 1000.0;
    (-1.0 / n).exp() as f32
}

fn to_db(x: f32) -> f32 {
    20.0 * x.max(1e-9).log10()
}

pub fn compressor_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.dynamics",
        name: "Dynamics",
        category: "Dynamics",
        description: "A compressor: evens out loud and quiet moments above the threshold.",
        params: vec![
            num("threshold", "Threshold", "dB", -60.0, 0.0, -20.0, 0.1),
            num("ratio", "Ratio", ":1", 1.0, 20.0, 3.0, 0.1).log(),
            num("attack", "Attack", "ms", 0.1, 200.0, 10.0, 0.1).log(),
            num("release", "Release", "ms", 5.0, 2000.0, 150.0, 1.0).log(),
            num("knee", "Knee", "dB", 0.0, 24.0, 6.0, 0.1),
            num("makeup", "Makeup gain", "dB", 0.0, 24.0, 0.0, 0.1),
            toggle("linked", "Link channels", true),
        ],
    }
}

pub fn multiband_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.multiband",
        name: "Multiband Compressor",
        category: "Dynamics",
        description:
            "Splits the sound into low, middle and high bands and compresses each on its own, to tame boom or harshness without squashing the whole voice.",
        params: vec![
            num("low_freq", "Low / mid split", "Hz", 40.0, 1000.0, 200.0, 1.0).log(),
            num("high_freq", "Mid / high split", "Hz", 1000.0, 12_000.0, 3500.0, 10.0).log(),
            num("low_threshold", "Low threshold", "dB", -60.0, 0.0, -24.0, 0.1),
            num("low_ratio", "Low ratio", ":1", 1.0, 20.0, 2.0, 0.1).log(),
            num("low_makeup", "Low gain", "dB", -12.0, 12.0, 0.0, 0.1),
            num("mid_threshold", "Mid threshold", "dB", -60.0, 0.0, -22.0, 0.1),
            num("mid_ratio", "Mid ratio", ":1", 1.0, 20.0, 2.5, 0.1).log(),
            num("mid_makeup", "Mid gain", "dB", -12.0, 12.0, 0.0, 0.1),
            num("high_threshold", "High threshold", "dB", -60.0, 0.0, -26.0, 0.1),
            num("high_ratio", "High ratio", ":1", 1.0, 20.0, 2.0, 0.1).log(),
            num("high_makeup", "High gain", "dB", -12.0, 12.0, 0.0, 0.1),
            num("attack", "Attack", "ms", 0.1, 200.0, 15.0, 0.1).log(),
            num("release", "Release", "ms", 5.0, 2000.0, 200.0, 1.0).log(),
            num("output", "Output gain", "dB", -12.0, 12.0, 0.0, 0.1),
        ],
    }
}

pub fn limiter_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.limiter",
        name: "Limiter",
        category: "Dynamics",
        description: "Keeps peaks from going over the ceiling, looking ahead so it never clips.",
        params: vec![
            num("ceiling", "Ceiling", "dB", -24.0, 0.0, -6.0, 0.1),
            num("input_boost", "Input boost", "dB", 0.0, 24.0, 0.0, 0.1),
            num("lookahead", "Look-ahead", "ms", 0.0, 20.0, 5.0, 0.1),
            num("release", "Release", "ms", 10.0, 1000.0, 100.0, 1.0).log(),
            toggle("linked", "Link channels", true),
        ],
    }
}

pub fn gate_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.breath_gate",
        name: "Breath Gate",
        category: "Dynamics",
        description: "Turns the track down between phrases, so breaths and room noise drop away.",
        params: vec![
            num("threshold", "Threshold", "dB", -80.0, 0.0, -40.0, 0.1),
            num("attack", "Attack", "ms", 0.1, 100.0, 2.0, 0.1).log(),
            num("hold", "Hold", "ms", 0.0, 1000.0, 50.0, 1.0),
            num("release", "Release", "ms", 5.0, 2000.0, 200.0, 1.0).log(),
            num("range", "Closed level", "dB", -100.0, 0.0, -60.0, 0.5),
        ],
    }
}

pub fn deesser_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.deesser",
        name: "De-Esser",
        category: "Dynamics",
        description:
            "Softens harsh S and SH sounds by turning down only the highs when they flare.",
        params: vec![
            num("freq", "Frequency", "Hz", 2000.0, 12_000.0, 6000.0, 10.0).log(),
            num("threshold", "Threshold", "dB", -60.0, 0.0, -30.0, 0.1),
            num("reduction", "Most reduction", "dB", 0.0, 24.0, 8.0, 0.1),
        ],
    }
}

pub fn leveler_info() -> EffectInfo {
    EffectInfo {
        id: "altaudio.speech_leveler",
        name: "Speech Leveler",
        category: "Dynamics",
        description: "Rides the level of speech toward a target, like a slow hand on the fader.",
        params: vec![
            num("target", "Target level", "dB", -40.0, -6.0, -20.0, 0.1),
            num("max_gain", "Most boost", "dB", 0.0, 30.0, 10.0, 0.1),
            num("max_cut", "Most cut", "dB", 0.0, 30.0, 10.0, 0.1),
            num("speed", "Response", "ms", 100.0, 10_000.0, 1500.0, 1.0).log(),
            num("floor", "Ignore below", "dB", -80.0, -20.0, -50.0, 0.1),
            num("amount", "Leveling amount", "%", 0.0, 100.0, 100.0, 1.0),
            num("range", "Target dynamic range", "dB", 0.0, 60.0, 45.0, 1.0),
            num("boost_low", "Boost low signals", "", 0.0, 1.0, 1.0, 1.0),
            num("comp_on", "Compressor", "", 0.0, 1.0, 0.0, 1.0),
            num(
                "comp_thr",
                "Compressor threshold",
                "dB",
                -9.0,
                -1.0,
                -1.0,
                0.1,
            ),
            num("gate_on", "Noise gate", "", 0.0, 1.0, 0.0, 1.0),
            num("gate_off", "Noise offset", "dB", 0.0, 25.0, 15.0, 0.1),
        ],
    }
}

// --- Dynamics (compressor) ---

pub struct Compressor {
    threshold: f32,
    slope: f32,
    knee: f32,
    makeup: f32,
    attack: f32,
    release: f32,
    linked: bool,
    /// Gain reduction in dB per channel (only [0] when linked).
    gr: [f32; 2],
}

impl Compressor {
    pub fn new(p: &Params, rate: u32) -> Self {
        Self::with(
            [
                p.get("threshold"),
                p.get("ratio"),
                p.get("knee"),
                p.get("makeup"),
            ],
            [p.get("attack"), p.get("release")],
            p.on("linked"),
            rate,
        )
    }

    /// From `[threshold dB, ratio, knee dB, makeup dB]` and `[attack ms, release ms]`.
    fn with(v: [f64; 4], ms: [f64; 2], linked: bool, rate: u32) -> Self {
        Self {
            threshold: v[0] as f32,
            slope: 1.0 - 1.0 / v[1].max(1.0) as f32,
            knee: v[2] as f32,
            makeup: v[3] as f32,
            attack: coef(ms[0], rate),
            release: coef(ms[1], rate),
            linked,
            gr: [0.0; 2],
        }
    }

    /// Wanted gain reduction in dB for a level in dB (soft knee).
    fn curve(&self, x: f32) -> f32 {
        let over = x - self.threshold;
        if self.knee > 0.0 && over.abs() <= self.knee / 2.0 {
            self.slope * (over + self.knee / 2.0).powi(2) / (2.0 * self.knee)
        } else if over > 0.0 {
            self.slope * over
        } else {
            0.0
        }
    }

    fn follow(&mut self, ch: usize, level: f32) -> f32 {
        let want = self.curve(to_db(level));
        let g = &mut self.gr[ch];
        let c = if want > *g { self.attack } else { self.release };
        *g = want + (*g - want) * c;
        db_to_gain(f64::from(self.makeup - *g))
    }
}

impl Compressor {
    /// Compresses `buf`, detecting the level of `key` (or of `buf` itself).
    fn run(&mut self, buf: &mut [f32], key: Option<&[f32]>) {
        for (i, f) in buf.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            let k = key.map_or([f[0], f[1]], |k| [k[i * 2], k[i * 2 + 1]]);
            if self.linked {
                let g = self.follow(0, k[0].abs().max(k[1].abs()));
                f[0] *= g;
                f[1] *= g;
            } else {
                let gl = self.follow(0, k[0].abs());
                let gr = self.follow(1, k[1].abs());
                f[0] *= gl;
                f[1] *= gr;
            }
        }
    }
}

impl Processor for Compressor {
    fn process(&mut self, buf: &mut [f32]) {
        self.run(buf, None);
    }
    fn process_keyed(&mut self, buf: &mut [f32], key: &[f32]) {
        self.run(buf, Some(key));
    }
    fn reset(&mut self) {
        self.gr = [0.0; 2];
    }
}

// --- Multiband Compressor ---

/// A Linkwitz-Riley split (two matched Butterworth sections per side), so
/// the low and high parts add back to a flat sound.
struct Split {
    lo: [StereoBiquad; 2],
    hi: [StereoBiquad; 2],
}

impl Split {
    fn new(rate: u32, freq: f64) -> Self {
        let q = std::f64::consts::FRAC_1_SQRT_2;
        let lp = Coeffs::new(Shape::LowPass, rate, freq, q, 0.0);
        let hp = Coeffs::new(Shape::HighPass, rate, freq, q, 0.0);
        Self {
            lo: [StereoBiquad::new(lp), StereoBiquad::new(lp)],
            hi: [StereoBiquad::new(hp), StereoBiquad::new(hp)],
        }
    }

    /// One frame in, (low, high) out.
    fn run(&mut self, f: [f32; 2]) -> ([f32; 2], [f32; 2]) {
        let mut l = f;
        let mut h = f;
        for b in &mut self.lo {
            l = [b.l.run(l[0]), b.r.run(l[1])];
        }
        for b in &mut self.hi {
            h = [b.l.run(h[0]), b.r.run(h[1])];
        }
        (l, h)
    }

    fn reset(&mut self) {
        for b in self.lo.iter_mut().chain(self.hi.iter_mut()) {
            b.reset();
        }
    }
}

pub struct Multiband {
    first: Split,
    /// The second split, also applied to the low band so all three bands share one phase.
    second: Split,
    second_low: Split,
    bands: [Compressor; 3],
    out: f32,
}

impl Multiband {
    pub fn new(p: &Params, rate: u32) -> Self {
        let ms = [p.get("attack"), p.get("release")];
        let band = |n: &str| {
            Compressor::with(
                [
                    p.get(&format!("{n}_threshold")),
                    p.get(&format!("{n}_ratio")),
                    6.0,
                    p.get(&format!("{n}_makeup")),
                ],
                ms,
                true,
                rate,
            )
        };
        let f1 = p.get("low_freq");
        let f2 = p.get("high_freq").max(f1 * 1.5);
        Self {
            first: Split::new(rate, f1),
            second: Split::new(rate, f2),
            second_low: Split::new(rate, f2),
            bands: [band("low"), band("mid"), band("high")],
            out: db_to_gain(p.get("output")),
        }
    }
}

impl Processor for Multiband {
    fn process(&mut self, buf: &mut [f32]) {
        for f in buf.as_chunks_mut::<2>().0 {
            let (low, rest) = self.first.run(*f);
            let (mid, high) = self.second.run(rest);
            let (la, lb) = self.second_low.run(low);
            let mut parts = [[la[0] + lb[0], la[1] + lb[1]], mid, high];
            let mut sum = [0f32; 2];
            for (part, comp) in parts.iter_mut().zip(self.bands.iter_mut()) {
                comp.run(part, None);
                sum[0] += part[0];
                sum[1] += part[1];
            }
            f[0] = sum[0] * self.out;
            f[1] = sum[1] * self.out;
        }
    }
    fn reset(&mut self) {
        self.first.reset();
        self.second.reset();
        self.second_low.reset();
        for b in &mut self.bands {
            b.reset();
        }
    }
}

// --- Limiter ---

/// One gain path of the limiter. With look-ahead L, the gain for a peak
/// is reached by the time that peak leaves the delay line:
/// 1. the gain each input sample needs,
/// 2. the smallest of those over the last L+1 samples,
/// 3. instant drop and smooth release,
/// 4. averaged over L+1 samples, so the drop is a ramp, not a step.
struct LimitPath {
    window: usize,
    /// (index, needed gain), increasing gains: front is the window minimum.
    mins: VecDeque<(u64, f32)>,
    held: f32,
    ring: Vec<f32>,
    at: usize,
    sum: f64,
    n: u64,
}

impl LimitPath {
    fn new(lookahead: usize) -> Self {
        let window = lookahead + 1;
        Self {
            window,
            mins: VecDeque::with_capacity(window + 1),
            held: 1.0,
            ring: vec![1.0; window],
            at: 0,
            sum: window as f64,
            n: 0,
        }
    }

    fn step(&mut self, need: f32, release: f32) -> f32 {
        while self.mins.back().is_some_and(|b| b.1 >= need) {
            self.mins.pop_back();
        }
        self.mins.push_back((self.n, need));
        while self
            .mins
            .front()
            .is_some_and(|f| f.0 + self.window as u64 <= self.n)
        {
            self.mins.pop_front();
        }
        self.n += 1;
        let min = self.mins.front().map_or(1.0, |f| f.1);
        self.held = if min < self.held {
            min
        } else {
            min + (self.held - min) * release
        };
        self.sum += f64::from(self.held) - f64::from(self.ring[self.at]);
        self.ring[self.at] = self.held;
        self.at = (self.at + 1) % self.window;
        (self.sum / self.window as f64) as f32
    }

    fn reset(&mut self) {
        self.mins.clear();
        self.held = 1.0;
        self.ring.fill(1.0);
        self.at = 0;
        self.sum = self.window as f64;
        self.n = 0;
    }
}

pub struct Limiter {
    ceiling: f32,
    boost: f32,
    release: f32,
    linked: bool,
    lookahead: usize,
    paths: [LimitPath; 2],
    delay: Vec<f32>,
    delay_at: usize,
}

impl Limiter {
    pub fn new(p: &Params, rate: u32) -> Self {
        let lookahead = (p.get("lookahead") * f64::from(rate) / 1000.0).round() as usize;
        Self {
            ceiling: db_to_gain(p.get("ceiling")),
            boost: db_to_gain(p.get("input_boost")),
            release: coef(p.get("release"), rate),
            linked: p.on("linked"),
            lookahead,
            paths: [LimitPath::new(lookahead), LimitPath::new(lookahead)],
            delay: vec![0.0; lookahead * 2],
            delay_at: 0,
        }
    }

    fn need(&self, x: f32) -> f32 {
        if x > self.ceiling {
            self.ceiling / x
        } else {
            1.0
        }
    }
}

impl Processor for Limiter {
    fn process(&mut self, buf: &mut [f32]) {
        let n = self.delay.len();
        for f in buf.as_chunks_mut::<2>().0 {
            let l = f[0] * self.boost;
            let r = f[1] * self.boost;
            let (gl, gr) = if self.linked {
                let g = self.paths[0].step(self.need(l.abs().max(r.abs())), self.release);
                (g, g)
            } else {
                let nl = self.need(l.abs());
                let nr = self.need(r.abs());
                (
                    self.paths[0].step(nl, self.release),
                    self.paths[1].step(nr, self.release),
                )
            };
            let (dl, dr) = if n == 0 {
                (l, r)
            } else {
                let a = self.delay_at;
                let out = (self.delay[a], self.delay[a + 1]);
                self.delay[a] = l;
                self.delay[a + 1] = r;
                self.delay_at = (a + 2) % n;
                out
            };
            // The clamp only catches float rounding; the gain already fits.
            let c = self.ceiling;
            f[0] = (dl * gl).clamp(-c, c);
            f[1] = (dr * gr).clamp(-c, c);
        }
    }
    fn latency(&self) -> usize {
        self.lookahead
    }
    fn reset(&mut self) {
        for p in &mut self.paths {
            p.reset();
        }
        self.delay.fill(0.0);
        self.delay_at = 0;
    }
}

// --- Breath Gate ---

pub struct Gate {
    open_at: f32,
    floor: f32,
    hold: u32,
    up: f32,
    down: f32,
    env_decay: f32,
    env: f32,
    held: u32,
    gain: f32,
}

impl Gate {
    pub fn new(p: &Params, rate: u32) -> Self {
        let per_ms = f64::from(rate) / 1000.0;
        let floor = if p.get("range") <= -99.0 {
            0.0
        } else {
            db_to_gain(p.get("range"))
        };
        Self {
            open_at: db_to_gain(p.get("threshold")),
            floor,
            hold: (p.get("hold") * per_ms) as u32,
            // Linear ramps across the whole attack or release time.
            up: (1.0 / (p.get("attack") * per_ms).max(1.0)) as f32,
            down: (1.0 / (p.get("release") * per_ms).max(1.0)) as f32,
            // The detector falls over 5 ms so it does not chatter on each cycle.
            env_decay: coef(5.0, rate),
            env: 0.0,
            held: 0,
            gain: floor,
        }
    }
}

impl Processor for Gate {
    fn process(&mut self, buf: &mut [f32]) {
        self.run(buf, None);
    }
    fn process_keyed(&mut self, buf: &mut [f32], key: &[f32]) {
        self.run(buf, Some(key));
    }
    fn reset(&mut self) {
        self.env = 0.0;
        self.held = 0;
        self.gain = self.floor;
    }
}

impl Gate {
    /// Gates `buf`, opening on the level of `key` (or of `buf` itself).
    fn run(&mut self, buf: &mut [f32], key: Option<&[f32]>) {
        let span = 1.0 - self.floor;
        for (i, f) in buf.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            let x = match key {
                Some(k) => k[i * 2].abs().max(k[i * 2 + 1].abs()),
                None => f[0].abs().max(f[1].abs()),
            };
            self.env = x.max(self.env * self.env_decay);
            let open = if self.env >= self.open_at {
                self.held = self.hold;
                true
            } else if self.held > 0 {
                self.held -= 1;
                true
            } else {
                false
            };
            if open {
                self.gain = (self.gain + self.up * span).min(1.0);
            } else {
                self.gain = (self.gain - self.down * span).max(self.floor);
            }
            f[0] *= self.gain;
            f[1] *= self.gain;
        }
    }
}

// --- De-Esser ---

/// A detector on the S band drives a high shelf that dips the highs by
/// as much as they flare. The shelf moves every 16 frames.
pub struct DeEsser {
    side: StereoBiquad,
    shelf: StereoBiquad,
    rate: u32,
    freq: f64,
    threshold: f32,
    most: f32,
    attack: f32,
    release: f32,
    env: f32,
    count: u32,
    cut: f32,
}

impl DeEsser {
    pub fn new(p: &Params, rate: u32) -> Self {
        Self {
            side: StereoBiquad::new(Coeffs::new(
                Shape::BandPass,
                rate,
                p.get("freq") * 1.2,
                1.2,
                0.0,
            )),
            shelf: StereoBiquad::new(Coeffs::FLAT),
            rate,
            freq: p.get("freq"),
            threshold: p.get("threshold") as f32,
            most: p.get("reduction") as f32,
            attack: coef(1.0, rate),
            release: coef(60.0, rate),
            env: 0.0,
            count: 0,
            cut: 0.0,
        }
    }
}

impl Processor for DeEsser {
    fn process(&mut self, buf: &mut [f32]) {
        for f in buf.as_chunks_mut::<2>().0 {
            let sl = self.side.l.run(f[0]);
            let sr = self.side.r.run(f[1]);
            let x = sl.abs().max(sr.abs());
            let c = if x > self.env {
                self.attack
            } else {
                self.release
            };
            self.env = x + (self.env - x) * c;
            if self.count == 0 {
                let cut = (to_db(self.env) - self.threshold).clamp(0.0, self.most);
                if (cut - self.cut).abs() > 0.05 {
                    self.cut = cut;
                    let co = if cut < 0.05 {
                        Coeffs::FLAT
                    } else {
                        Coeffs::new(Shape::HighShelf, self.rate, self.freq, 1.0, -f64::from(cut))
                    };
                    self.shelf.l.set(co);
                    self.shelf.r.set(co);
                }
            }
            self.count = (self.count + 1) % 16;
            f[0] = self.shelf.l.run(f[0]);
            f[1] = self.shelf.r.run(f[1]);
        }
    }
    fn reset(&mut self) {
        self.side.reset();
        self.shelf = StereoBiquad::new(Coeffs::FLAT);
        self.env = 0.0;
        self.count = 0;
        self.cut = 0.0;
    }
}

// --- Speech Leveler ---

pub struct SpeechLeveler {
    target: f32,
    max_gain: f32,
    max_cut: f32,
    floor: f32,
    /// How much of the correction is applied, 0 to 1.
    amount: f32,
    /// Half-width in dB around the target where the level is left alone.
    dead: f32,
    comp: Option<f32>,
    /// Noise gate threshold in dB: the floor plus the noise offset.
    gate: Option<f32>,
    gate_db: f32,
    gate_coef: f32,
    /// Mean square follower (fast, about a syllable).
    ms_coef: f32,
    ms: f32,
    /// Gain follower (slow, the Response setting).
    gain_coef: f32,
    gain_db: f32,
}

impl SpeechLeveler {
    pub fn new(p: &Params, rate: u32) -> Self {
        Self {
            target: p.get("target") as f32,
            max_gain: if p.get("boost_low") >= 0.5 {
                p.get("max_gain") as f32
            } else {
                0.0
            },
            max_cut: p.get("max_cut") as f32,
            floor: p.get("floor") as f32,
            amount: (p.get("amount") / 100.0).clamp(0.0, 1.0) as f32,
            dead: (p.get("range") / 20.0).max(0.0) as f32,
            comp: (p.get("comp_on") >= 0.5).then(|| db_to_gain(p.get("comp_thr"))),
            gate: (p.get("gate_on") >= 0.5).then(|| (p.get("floor") + p.get("gate_off")) as f32),
            gate_db: 0.0,
            gate_coef: coef(60.0, rate),
            ms_coef: coef(300.0, rate),
            ms: 0.0,
            gain_coef: coef(p.get("speed"), rate),
            gain_db: 0.0,
        }
    }
}

impl Processor for SpeechLeveler {
    fn process(&mut self, buf: &mut [f32]) {
        for f in buf.as_chunks_mut::<2>().0 {
            let x = (f[0] * f[0] + f[1] * f[1]) * 0.5;
            self.ms = x + (self.ms - x) * self.ms_coef;
            let level = 10.0 * self.ms.max(1e-18).log10();
            // Pauses and room noise leave the gain where it was, so the
            // noise floor is never pumped up between sentences.
            if level > self.floor {
                let dev = self.target - level;
                let dev = dev.signum() * (dev.abs() - self.dead).max(0.0);
                let want = (dev * self.amount).clamp(-self.max_cut, self.max_gain);
                self.gain_db = want + (self.gain_db - want) * self.gain_coef;
            }
            // The gate eases the sound down 20 dB while it sits below the threshold.
            if let Some(thr) = self.gate {
                let to = if level < thr { -20.0 } else { 0.0 };
                self.gate_db = to + (self.gate_db - to) * self.gate_coef;
            }
            let g = db_to_gain(f64::from(self.gain_db + self.gate_db));
            f[0] *= g;
            f[1] *= g;
            // The compressor only tames peaks left over after leveling: 4:1 above the threshold.
            if let Some(t) = self.comp {
                for v in f.iter_mut() {
                    let a = v.abs();
                    if a > t {
                        *v = v.signum() * (t + (a - t) / 4.0);
                    }
                }
            }
        }
    }
    fn reset(&mut self) {
        self.ms = 0.0;
        self.gain_db = 0.0;
        self.gate_db = 0.0;
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
        // Odd block sizes, as the renderer may send.
        for chunk in b.chunks_mut(2 * 777) {
            p.process(chunk);
        }
        b
    }

    #[test]
    fn limiter_never_passes_the_ceiling() {
        // A sudden full-scale burst after silence: the worst case.
        let mut b = vec![0f32; 2 * 4800];
        b.extend(sine(1000.0, 1.0, 0.2));
        b.extend(vec![0f32; 2 * 4800]);
        for la in [0.0, 1.0, 5.0, 20.0] {
            let out = run(
                "altaudio.limiter",
                &[("ceiling", -6.0), ("lookahead", la)],
                b.clone(),
            );
            assert!(
                peak(&out) <= 0.5012 + 1e-4,
                "look-ahead {la}: {}",
                peak(&out)
            );
        }
        // Quiet audio passes untouched apart from the delay.
        let quiet = sine(500.0, 0.1, 0.1);
        let out = run("altaudio.limiter", &[("lookahead", 0.0)], quiet.clone());
        assert_eq!(out, quiet);
    }

    #[test]
    fn compressor_follows_its_ratio() {
        let out = run(
            "altaudio.dynamics",
            &[("threshold", -20.0), ("ratio", 4.0), ("knee", 0.0)],
            sine(1000.0, 1.0, 1.0),
        );
        // 0 dB in, 20 dB over: 4:1 leaves 5 dB over, so about -15 dB.
        let tail = &out[out.len() / 2..];
        let db = 20.0 * peak(tail).log10();
        assert!((db + 15.0).abs() < 1.0, "{db}");
        let quiet = run(
            "altaudio.dynamics",
            &[("threshold", -20.0)],
            sine(1000.0, 0.01, 0.5),
        );
        assert!(
            (peak(&quiet[quiet.len() / 2..]) - 0.01).abs() < 1e-3,
            "below threshold is untouched"
        );
    }

    #[test]
    fn gate_closes_on_quiet_and_opens_on_speech() {
        let mut b = sine(300.0, 0.002, 1.0); // -54 dB room tone
        b.extend(sine(300.0, 0.5, 0.5));
        let out = run(
            "altaudio.breath_gate",
            &[("threshold", -30.0), ("range", -40.0)],
            b,
        );
        let half = 48_000;
        assert!(
            peak(&out[half..2 * half]) < 0.002 * 0.02,
            "closed part is down 40 dB"
        );
        assert!(
            peak(&out[out.len() - half / 2..]) > 0.49,
            "open part passes"
        );
    }

    #[test]
    fn deesser_cuts_highs_not_lows() {
        let hi = run(
            "altaudio.deesser",
            &[("threshold", -40.0), ("reduction", 12.0)],
            sine(7000.0, 0.5, 0.5),
        );
        assert!(peak(&hi[hi.len() / 2..]) < 0.25, "harsh highs come down");
        let lo = run(
            "altaudio.deesser",
            &[("threshold", -40.0)],
            sine(200.0, 0.5, 0.5),
        );
        assert!(peak(&lo[lo.len() / 2..]) > 0.47, "voice body stays");
    }

    #[test]
    fn leveler_brings_quiet_speech_up_and_ignores_silence() {
        // -36 dB RMS speech toward -20 dB, limited to +10 dB of boost.
        let out = run(
            "altaudio.speech_leveler",
            &[("speed", 200.0)],
            sine(200.0, 0.0224, 3.0),
        );
        let gain = peak(&out[out.len() - 9600..]) / 0.0224;
        assert!((20.0 * gain.log10() - 10.0).abs() < 0.5);
        let silent = run("altaudio.speech_leveler", &[], sine(200.0, 0.0001, 1.0));
        assert!(
            (peak(&silent) - 0.0001).abs() < 1e-6,
            "below the floor nothing moves"
        );
    }

    #[test]
    fn leveler_amount_boost_switch_compressor_and_gate() {
        let quiet = sine(200.0, 0.0224, 3.0);
        let tail = |o: &[f32]| peak(&o[o.len() - 9600..]) / 0.0224;
        // Amount 0: nothing is corrected.
        let none = run(
            "altaudio.speech_leveler",
            &[("speed", 200.0), ("amount", 0.0)],
            quiet.clone(),
        );
        assert!((tail(&none) - 1.0).abs() < 0.02);
        // Half the correction: about half the boost in dB.
        let half = run(
            "altaudio.speech_leveler",
            &[("speed", 200.0), ("amount", 50.0), ("range", 0.0)],
            quiet.clone(),
        );
        assert!(
            (20.0 * tail(&half).log10() - 8.0).abs() < 0.8,
            "{}",
            20.0 * tail(&half).log10()
        );
        // Boost low signals off: quiet speech is not raised.
        let off = run(
            "altaudio.speech_leveler",
            &[("speed", 200.0), ("boost_low", 0.0)],
            quiet,
        );
        assert!((tail(&off) - 1.0).abs() < 0.02);
        // Compressor: a hot peak is pulled down above the threshold.
        let hot = run(
            "altaudio.speech_leveler",
            &[("comp_on", 1.0), ("comp_thr", -6.0), ("target", -6.0)],
            sine(200.0, 0.9, 1.0),
        );
        assert!(peak(&hot) < 0.9);
        // Gate: speech below the floor plus offset is eased down.
        let gated = run(
            "altaudio.speech_leveler",
            &[
                ("gate_on", 1.0),
                ("floor", -60.0),
                ("gate_off", 20.0),
                ("amount", 0.0),
            ],
            sine(200.0, 0.005, 1.0),
        );
        assert!(peak(&gated[gated.len() / 2..]) < 0.0015);
    }

    #[test]
    fn multiband_at_ratio_one_passes_sound_through_at_the_same_level() {
        let flat = [("low_ratio", 1.0), ("mid_ratio", 1.0), ("high_ratio", 1.0)];
        for hz in [80.0, 200.0, 1000.0, 3500.0, 9000.0] {
            let out = run("altaudio.multiband", &flat, sine(hz, 0.3, 0.6));
            let p = peak(&out[out.len() / 2..]);
            assert!((p - 0.3).abs() < 0.015, "{hz} Hz came out at {p}");
        }
    }

    #[test]
    fn multiband_squeezes_only_the_band_that_is_loud() {
        let set = [
            ("low_threshold", -30.0),
            ("low_ratio", 8.0),
            ("mid_ratio", 1.0),
            ("high_ratio", 1.0),
        ];
        let boom = run("altaudio.multiband", &set, sine(100.0, 0.5, 0.6));
        assert!(peak(&boom[boom.len() / 2..]) < 0.3, "loud lows come down");
        let voice = run("altaudio.multiband", &set, sine(1000.0, 0.5, 0.6));
        assert!(
            peak(&voice[voice.len() / 2..]) > 0.47,
            "the middle is left alone"
        );
    }
}
