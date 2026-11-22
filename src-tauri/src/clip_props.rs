// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Clip property rules: the channel mix of a multichannel file, and the lines
//! that describe a Music Fit plan. The Rust copy of the pure parts of
//! `src/panels/properties/channel-mix.ts` and `src/panels/editor/musicfit.ts`;
//! both are tested against `shared-fixtures/panel-rules.json`.

use crate::js_compat::{js_round, js_to_fixed1};

/// One input channel's level into the left and right outputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Send {
    pub left: f64,
    pub right: f64,
}

fn s(left: f64, right: f64) -> Send {
    Send { left, right }
}

/// Speaker names for a file with `n` channels, in file order; numbers when the
/// order is unknown.
pub fn channel_labels(n: usize) -> Vec<String> {
    let named: Option<&[&str]> = match n {
        1 => Some(&["M"]),
        2 => Some(&["L", "R"]),
        3 => Some(&["L", "R", "C"]),
        4 => Some(&["L", "R", "Ls", "Rs"]),
        5 => Some(&["L", "R", "C", "Ls", "Rs"]),
        6 => Some(&["L", "R", "C", "LFE", "Ls", "Rs"]),
        7 => Some(&["L", "R", "C", "LFE", "Cs", "Ls", "Rs"]),
        8 => Some(&["L", "R", "C", "LFE", "Lrs", "Rrs", "Lss", "Rss"]),
        10 => Some(&["L", "R", "C", "LFE", "Ls", "Rs", "Ltf", "Rtf", "Ltr", "Rtr"]),
        12 => Some(&[
            "L", "R", "C", "LFE", "Lrs", "Rrs", "Lss", "Rss", "Ltf", "Rtf", "Ltr", "Rtr",
        ]),
        _ => None,
    };
    match named {
        Some(v) => v.iter().map(|x| x.to_string()).collect(),
        None => (1..=n).map(|k| k.to_string()).collect(),
    }
}

const M3: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// The standard fold-down: fronts straight, center and surrounds 3 dB down,
/// LFE left out.
pub fn default_mix(n: usize) -> Vec<Send> {
    channel_labels(n)
        .iter()
        .enumerate()
        .map(|(k, name)| match name.as_str() {
            "M" => s(1.0, 1.0),
            "L" => s(1.0, 0.0),
            "R" => s(0.0, 1.0),
            "C" => s(M3, M3),
            "Cs" => s(0.5, 0.5),
            "LFE" => s(0.0, 0.0),
            x if x.starts_with('L') => s(M3, 0.0),
            x if x.starts_with('R') => s(0.0, M3),
            _ => match k {
                0 => s(1.0, 0.0),
                1 => s(0.0, 1.0),
                _ => s(0.0, 0.0),
            },
        })
        .collect()
}

/// The mix a clip plays with now: its own, filled out to `n` channels, or the
/// default.
pub fn mix_of(own: Option<&[Send]>, n: usize) -> Vec<Send> {
    match own {
        None => default_mix(n),
        Some(o) => (0..n)
            .map(|k| o.get(k).copied().unwrap_or(s(0.0, 0.0)))
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixPreset {
    pub id: String,
    pub label: String,
    pub mix: Vec<Send>,
}

fn only(n: usize, pick: &[(usize, (f64, f64))]) -> Vec<Send> {
    (0..n)
        .map(|k| {
            pick.iter()
                .rev()
                .find(|(i, _)| *i == k)
                .map(|(_, (l, r))| s(*l, *r))
                .unwrap_or(s(0.0, 0.0))
        })
        .collect()
}

/// Quick settings for a file of `n` channels.
pub fn presets(n: usize) -> Vec<MixPreset> {
    let names = channel_labels(n);
    let mut out = vec![MixPreset {
        id: "default".into(),
        label: if n > 2 {
            "Standard fold-down"
        } else {
            "As recorded"
        }
        .into(),
        mix: default_mix(n),
    }];
    if let Some(lfe) = names.iter().position(|x| x == "LFE") {
        let mut m = default_mix(n);
        m[lfe] = s(0.5, 0.5);
        out.push(MixPreset {
            id: "fold-lfe".into(),
            label: "Fold-down with LFE".into(),
            mix: m,
        });
    }
    if n == 2 {
        out.push(MixPreset {
            id: "swap".into(),
            label: "Swap left and right".into(),
            mix: only(2, &[(0, (0.0, 1.0)), (1, (1.0, 0.0))]),
        });
    }
    if n >= 2 {
        let g = 1.0 / n as f64;
        out.push(MixPreset {
            id: "mono".into(),
            label: "Mono: all channels even".into(),
            mix: names.iter().map(|_| s(g, g)).collect(),
        });
    }
    // Each left/right pair on its own: L and R, Ls and Rs, Ltf and Rtf...
    for (k, a) in names.iter().enumerate() {
        if !a.starts_with('L') || a == "LFE" {
            continue;
        }
        let Some(r) = names.iter().position(|x| *x == format!("R{}", &a[1..])) else {
            continue;
        };
        out.push(MixPreset {
            id: format!("pair-{k}"),
            label: format!("{a} and {} only", names[r]),
            mix: only(n, &[(k, (1.0, 0.0)), (r, (0.0, 1.0))]),
        });
    }
    if n > 2 && !names.iter().any(|x| x == "L") {
        out.push(MixPreset {
            id: "pair-0".into(),
            label: "Channels 1 and 2 only".into(),
            mix: only(n, &[(0, (1.0, 0.0)), (1, (0.0, 1.0))]),
        });
    }
    if n > 1 {
        for (k, name) in names.iter().enumerate() {
            out.push(MixPreset {
                id: format!("one-{k}"),
                label: format!("{name} alone, to both sides"),
                mix: only(n, &[(k, (1.0, 1.0))]),
            });
        }
    }
    out
}

/// The preset that matches `mix`, if one does.
pub fn preset_for(mix: &[Send], n: usize) -> Option<String> {
    let near = |a: f64, b: f64| (a - b).abs() < 1e-4;
    presets(n)
        .into_iter()
        .find(|p| {
            p.mix.iter().enumerate().all(|(k, x)| {
                near(x.left, mix.get(k).map_or(0.0, |m| m.left))
                    && near(x.right, mix.get(k).map_or(0.0, |m| m.right))
            })
        })
        .map(|p| p.id)
}

// ---- Music Fit ----

/// Crossfade at each join, in seconds.
pub const FIT_XFADE: f64 = 0.04;

#[derive(Debug, Clone, PartialEq)]
pub struct FitPlan {
    /// (start, end) in seconds into the file.
    pub segments: Vec<(f64, f64)>,
    pub joins: u32,
    pub quality: f64,
    pub stretch: f64,
    pub tempo: f64,
}

/// True when `target` is the clip's own length (to a few milliseconds), so
/// fitting would change nothing.
pub fn same_length(target: f64, current: f64) -> bool {
    (target - current).abs() < 0.05
}

/// One line for the dialog: how the plan sounds.
pub fn describe_plan(p: &FitPlan) -> String {
    let joins = match p.joins {
        0 => "No joins".to_string(),
        1 => "1 join".to_string(),
        n => format!("{n} joins"),
    };
    let matched = format!("{}% match", js_round(p.quality * 100.0));
    let st = if (p.stretch - 1.0).abs() < 0.0005 {
        "no stretch".to_string()
    } else {
        format!("stretched {}%", js_to_fixed1((p.stretch - 1.0) * 100.0))
    };
    format!("{joins}, {matched}, {} BPM, {st}", js_round(p.tempo))
}

fn clock(secs: f64) -> String {
    let m = (secs / 60.0).floor();
    let rest = js_to_fixed1(secs - m * 60.0);
    if m > 0.0 {
        format!("{}:{rest:0>4}", m)
    } else {
        format!("{rest} s")
    }
}

/// Where the plan joins, in seconds into the clip (`origin` is where the clip
/// starts in its file): "Cuts 0:42.0 to 1:10.0" for a skip, "Repeats 0:30.0 to
/// 0:46.0, 2 more times" for a loop.
pub fn describe_joins(p: &FitPlan, origin: f64) -> String {
    let (Some(a), Some(b)) = (p.segments.first(), p.segments.get(1)) else {
        return "Plays the whole clip".to_string();
    };
    let out = a.1 - FIT_XFADE / 2.0 - origin;
    let back = b.0 + FIT_XFADE / 2.0 - origin;
    if back > out {
        return format!("Cuts {} to {}", clock(out), clock(back));
    }
    let more = p.segments.len() - 1;
    format!(
        "Repeats {} to {}, {}",
        clock(back),
        clock(out),
        if more == 1 {
            "1 more time".to_string()
        } else {
            format!("{more} more times")
        }
    )
}
