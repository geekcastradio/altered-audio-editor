// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Dynamics, all in one window like Audition's: Gate, Compressor,
//! Expander and Limiter, each with its own On switch, run in that order.
//! Each stage is our own gate, compressor, expander or limiter, so the
//! sound matches those effects when set the same way.

use std::collections::BTreeMap;

use super::{
    dynamics::{compressor_info, gate_info, limiter_info, Compressor, Gate, Limiter},
    expander, num, toggle, EffectInfo, Params, Processor,
};

pub const SUITE: &str = "altaudio.dynamics_suite";

pub fn info() -> EffectInfo {
    EffectInfo {
        id: SUITE,
        name: "Dynamics (Gate, Compressor, Expander, Limiter)",
        category: "Dynamics",
        description: "A gate, a compressor, an expander and a limiter in one window, run in that order. Turn on the ones you want.",
        params: vec![
            toggle("g_on", "On", true).group("Gate"),
            num("g_threshold", "Threshold", "dB", -80.0, 0.0, -14.0, 0.1).group("Gate"),
            num("g_attack", "Attack", "ms", 0.1, 100.0, 2.0, 0.1).log().group("Gate"),
            num("g_release", "Release", "ms", 5.0, 2000.0, 200.0, 1.0).log().group("Gate"),
            num("g_hold", "Hold", "ms", 0.0, 1000.0, 50.0, 1.0).group("Gate"),
            toggle("c_on", "On", false).group("Compressor"),
            num("c_threshold", "Threshold", "dB", -60.0, 0.0, -20.0, 0.1).group("Compressor"),
            num("c_ratio", "Ratio", ":1", 1.0, 20.0, 1.0, 0.1).log().group("Compressor"),
            num("c_attack", "Attack", "ms", 0.1, 200.0, 1.0, 0.1).log().group("Compressor"),
            num("c_release", "Release", "ms", 5.0, 2000.0, 50.0, 1.0).log().group("Compressor"),
            num("c_makeup", "Makeup", "dB", 0.0, 24.0, 0.0, 0.1).group("Compressor"),
            toggle("e_on", "On", false).group("Expander"),
            num("e_threshold", "Threshold", "dB", -80.0, 0.0, -20.0, 0.1).group("Expander"),
            num("e_ratio", "Ratio", ":1", 1.0, 20.0, 1.0, 0.1).log().group("Expander"),
            toggle("l_on", "On", false).group("Limiter"),
            num("l_threshold", "Threshold", "dB", -24.0, 0.0, -1.0, 0.1).group("Limiter"),
            num("l_release", "Release", "ms", 10.0, 1000.0, 50.0, 1.0).log().group("Limiter"),
        ],
    }
}

/// The values for one stage's own effect: the suite's settings with the stage's prefix, plus fixed ones.
fn stage(sub: &EffectInfo, p: &Params, map: &[(&str, &str)], fixed: &[(&str, f64)]) -> Params {
    let mut saved: BTreeMap<String, f64> =
        fixed.iter().map(|(k, v)| ((*k).to_string(), *v)).collect();
    for (from, to) in map {
        saved.insert((*to).to_string(), p.get(from));
    }
    Params::resolve(sub, &saved)
}

pub struct Suite {
    gate: Option<Gate>,
    comp: Option<Compressor>,
    exp: Option<expander::Expander>,
    lim: Option<Limiter>,
}

impl Suite {
    pub fn new(p: &Params, rate: u32) -> Self {
        let on = |k: &str| p.get(k) >= 0.5;
        let gate = on("g_on").then(|| {
            let s = stage(
                &gate_info(),
                p,
                &[
                    ("g_threshold", "threshold"),
                    ("g_attack", "attack"),
                    ("g_release", "release"),
                    ("g_hold", "hold"),
                ],
                &[],
            );
            Gate::new(&s, rate)
        });
        let comp = on("c_on").then(|| {
            let s = stage(
                &compressor_info(),
                p,
                &[
                    ("c_threshold", "threshold"),
                    ("c_ratio", "ratio"),
                    ("c_attack", "attack"),
                    ("c_release", "release"),
                    ("c_makeup", "makeup"),
                ],
                &[],
            );
            Compressor::new(&s, rate)
        });
        let exp = on("e_on").then(|| {
            let s = stage(
                &expander::info(),
                p,
                &[("e_threshold", "threshold"), ("e_ratio", "ratio")],
                &[],
            );
            expander::Expander::new(&s, rate)
        });
        let lim = on("l_on").then(|| {
            let s = stage(
                &limiter_info(),
                p,
                &[("l_threshold", "ceiling"), ("l_release", "release")],
                &[],
            );
            Limiter::new(&s, rate)
        });
        Self {
            gate,
            comp,
            exp,
            lim,
        }
    }
}

impl Processor for Suite {
    fn process(&mut self, buf: &mut [f32]) {
        if let Some(g) = &mut self.gate {
            g.process(buf);
        }
        if let Some(c) = &mut self.comp {
            c.process(buf);
        }
        if let Some(e) = &mut self.exp {
            e.process(buf);
        }
        if let Some(l) = &mut self.lim {
            l.process(buf);
        }
    }
    fn latency(&self) -> usize {
        self.lim.as_ref().map_or(0, |l| l.latency())
    }
    fn reset(&mut self) {
        if let Some(g) = &mut self.gate {
            g.reset();
        }
        if let Some(c) = &mut self.comp {
            c.reset();
        }
        if let Some(e) = &mut self.exp {
            e.reset();
        }
        if let Some(l) = &mut self.lim {
            l.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::build;
    use super::super::tests::slot;
    use std::f32::consts::PI;

    fn sine(amp: f32, secs: f32) -> Vec<f32> {
        (0..(48_000.0 * secs) as usize)
            .flat_map(|i| {
                let v = (2.0 * PI * 500.0 * i as f32 / 48_000.0).sin() * amp;
                [v, v]
            })
            .collect()
    }
    fn peak(b: &[f32]) -> f32 {
        b.iter().fold(0f32, |m, s| m.max(s.abs()))
    }
    fn run(params: &[(&str, f64)], mut b: Vec<f32>) -> Vec<f32> {
        let mut p = build(&slot(super::SUITE, params), 48_000).unwrap();
        for chunk in b.chunks_mut(2 * 777) {
            p.process(chunk);
        }
        b
    }

    #[test]
    fn stages_switch_on_and_off() {
        // Everything off: the sound passes untouched.
        let loud = sine(0.9, 0.5);
        let off = run(&[("g_on", 0.0)], loud.clone());
        assert_eq!(off, loud);
        // Gate on at -14 dB: quiet sound (-40 dB) is shut away.
        let quiet = sine(0.01, 0.5);
        let gated = run(&[("g_on", 1.0), ("g_threshold", -14.0)], quiet.clone());
        assert!(peak(&gated[gated.len() / 2..]) < 0.002);
        // Limiter only: peaks stop at the threshold.
        let lim = run(
            &[("g_on", 0.0), ("l_on", 1.0), ("l_threshold", -6.0)],
            loud.clone(),
        );
        assert!(peak(&lim[lim.len() / 2..]) <= 0.5012 + 1e-3);
        // Compressor only, 4:1 above -20 dB: a full-scale tone comes out well below where it went in.
        let comp = run(
            &[
                ("g_on", 0.0),
                ("c_on", 1.0),
                ("c_threshold", -20.0),
                ("c_ratio", 4.0),
            ],
            loud.clone(),
        );
        assert!(peak(&comp[comp.len() / 2..]) < 0.5);
    }
}
