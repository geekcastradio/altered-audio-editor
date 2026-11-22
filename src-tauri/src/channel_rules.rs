// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Source Channels quick settings and the Waveform editor's channel
//! switches. Matches `src/panels/properties/channel-mix.ts` and
//! `src/panels/editor/wave-channels.ts`; both are checked against
//! `shared-fixtures/channel-rules.json`.

use std::f64::consts::FRAC_1_SQRT_2;

/// Speaker names for a file with `n` channels, in file order; numbers when the order is unknown.
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
        Some(list) => list.iter().map(|s| s.to_string()).collect(),
        None => (0..n).map(|k| (k + 1).to_string()).collect(),
    }
}

/// How much of one channel goes to the left and the right.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Send {
    pub left: f64,
    pub right: f64,
}

const fn send(left: f64, right: f64) -> Send {
    Send { left, right }
}

/// The standard fold-down: fronts straight, center and surrounds 3 dB down, LFE left out.
pub fn default_mix(n: usize) -> Vec<Send> {
    channel_labels(n)
        .iter()
        .enumerate()
        .map(|(k, name)| match name.as_str() {
            "M" => send(1.0, 1.0),
            "L" => send(1.0, 0.0),
            "R" => send(0.0, 1.0),
            "C" => send(FRAC_1_SQRT_2, FRAC_1_SQRT_2),
            "Cs" => send(0.5, 0.5),
            "LFE" => send(0.0, 0.0),
            s if s.starts_with('L') => send(FRAC_1_SQRT_2, 0.0),
            s if s.starts_with('R') => send(0.0, FRAC_1_SQRT_2),
            _ => match k {
                0 => send(1.0, 0.0),
                1 => send(0.0, 1.0),
                _ => send(0.0, 0.0),
            },
        })
        .collect()
}

/// The mix a clip plays with now: its own, filled out to `n` channels, or the default.
pub fn mix_of(own: Option<&[Send]>, n: usize) -> Vec<Send> {
    match own {
        None => default_mix(n),
        Some(s) => (0..n)
            .map(|k| s.get(k).copied().unwrap_or(send(0.0, 0.0)))
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixPreset {
    pub id: String,
    pub label: String,
    pub mix: Vec<Send>,
}

fn only(n: usize, pick: &[(usize, f64, f64)]) -> Vec<Send> {
    (0..n)
        .map(|k| {
            pick.iter()
                .find(|p| p.0 == k)
                .map(|p| send(p.1, p.2))
                .unwrap_or(send(0.0, 0.0))
        })
        .collect()
}

fn preset(id: &str, label: String, mix: Vec<Send>) -> MixPreset {
    MixPreset {
        id: id.to_string(),
        label,
        mix,
    }
}

/// Quick settings for a file of `n` channels.
pub fn presets(n: usize) -> Vec<MixPreset> {
    let names = channel_labels(n);
    let mut out = vec![preset(
        "default",
        if n > 2 {
            "Standard fold-down".into()
        } else {
            "As recorded".into()
        },
        default_mix(n),
    )];
    if let Some(lfe) = names.iter().position(|s| s == "LFE") {
        let mut m = default_mix(n);
        m[lfe] = send(0.5, 0.5);
        out.push(preset("fold-lfe", "Fold-down with LFE".into(), m));
    }
    if n == 2 {
        out.push(preset(
            "swap",
            "Swap left and right".into(),
            only(2, &[(0, 0.0, 1.0), (1, 1.0, 0.0)]),
        ));
    }
    if n >= 2 {
        let g = 1.0 / n as f64;
        out.push(preset(
            "mono",
            "Mono: all channels even".into(),
            names.iter().map(|_| send(g, g)).collect(),
        ));
    }
    // Each left/right pair on its own: L and R, Ls and Rs, Ltf and Rtf...
    for k in 0..n {
        let a = &names[k];
        if !a.starts_with('L') || a == "LFE" {
            continue;
        }
        let want = format!("R{}", &a[1..]);
        let Some(r) = names.iter().position(|s| *s == want) else {
            continue;
        };
        out.push(preset(
            &format!("pair-{k}"),
            format!("{a} and {} only", names[r]),
            only(n, &[(k, 1.0, 0.0), (r, 0.0, 1.0)]),
        ));
    }
    if n > 2 && !names.iter().any(|s| s == "L") {
        out.push(preset(
            "pair-0",
            "Channels 1 and 2 only".into(),
            only(n, &[(0, 1.0, 0.0), (1, 0.0, 1.0)]),
        ));
    }
    if n > 1 {
        for (k, name) in names.iter().enumerate() {
            out.push(preset(
                &format!("one-{k}"),
                format!("{name} alone, to both sides"),
                only(n, &[(k, 1.0, 1.0)]),
            ));
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
            p.mix.iter().enumerate().all(|(k, s)| {
                let (l, r) = mix.get(k).map(|m| (m.left, m.right)).unwrap_or((0.0, 0.0));
                near(s.left, l) && near(s.right, r)
            })
        })
        .map(|p| p.id)
}

/// Which channels edits reach, one flag per channel. Never all off.
pub type Active = Vec<bool>;

pub fn all_on(n: usize) -> Active {
    vec![true; n.max(1)]
}

fn is_all(c: &[bool]) -> bool {
    c.iter().all(|&v| v)
}

/// Switches channel `k` on or off. Switching off the only one left switches the others on.
pub fn toggle_channel(c: &[bool], k: usize) -> Active {
    let next: Active = c
        .iter()
        .enumerate()
        .map(|(i, &on)| if i == k { !on } else { on })
        .collect();
    if next.iter().any(|&v| v) {
        next
    } else {
        c.iter().enumerate().map(|(i, _)| i != k).collect()
    }
}

/// Channel `k` alone, or every channel again when it already is.
pub fn solo_channel(c: &[bool], k: usize) -> Active {
    let only: Active = (0..c.len()).map(|i| i == k).collect();
    if c == only.as_slice() {
        all_on(c.len())
    } else {
        only
    }
}

/// The one channel on, or None when more (or all) are.
pub fn one_channel(c: &[bool]) -> Option<usize> {
    let on: Vec<usize> = c
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(i, _)| i)
        .collect();
    if on.len() == 1 && c.len() > 1 {
        Some(on[0])
    } else {
        None
    }
}

/// True when some channels are off, so edits reach only part of the file.
pub fn some_off(c: &[bool]) -> bool {
    c.len() > 1 && !is_all(c)
}

/// Where one output channel comes from: the file before or after the edit, and its channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Source {
    pub after: bool,
    pub channel: usize,
}

/// For an edit in place: channels that are on take the edited audio. None with all on.
pub fn edit_take(c: &[bool]) -> Option<Vec<Source>> {
    if is_all(c) {
        return None;
    }
    Some(
        c.iter()
            .enumerate()
            .map(|(k, &on)| Source {
                after: on,
                channel: k,
            })
            .collect(),
    )
}

/// For a paste over the audio: each channel that is on takes the copy's `from` channel (its own when None).
pub fn paste_take(c: &[bool], from: Option<usize>) -> Option<Vec<Source>> {
    if is_all(c) && from.is_none() {
        return None;
    }
    Some(
        c.iter()
            .enumerate()
            .map(|(k, &on)| {
                if on {
                    Source {
                        after: true,
                        channel: from.unwrap_or(k),
                    }
                } else {
                    Source {
                        after: false,
                        channel: k,
                    }
                }
            })
            .collect(),
    )
}

/// " (Left Channel)", " (C, LFE)" and so on for edit labels, or "" with all on.
pub fn channel_label(c: &[bool]) -> String {
    if is_all(c) {
        return String::new();
    }
    if c.len() == 2 {
        return if c[0] {
            " (Left Channel)".into()
        } else {
            " (Right Channel)".into()
        };
    }
    let names = channel_labels(c.len());
    let on: Vec<&str> = c
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(k, _)| names[k].as_str())
        .collect();
    format!(" ({})", on.join(", "))
}
