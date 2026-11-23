// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Bars and beats in the ruler and grid: bar.beat.tick text, parsing it, ruler
//! ticks and grid lines through tempo and time signature changes. Mirrors the
//! bars part of `src/panels/editor/timeline.ts`, tested against
//! `shared-fixtures/bar-ruler.json`.

use crate::js_compat::{is_js_space, js_round};
use crate::tempo_map::{
    bar_at, bar_start, beats_to_seconds, seconds_to_beats, segment_at_beat, segments, unit_beats,
    Tempo,
};
use crate::timeline::{Tick, Viewport};

/// Ticks (fractions of a beat) shown in bar.beat.tick: 960 to a beat.
pub const TICKS_PER_BEAT: f64 = 960.0;

/// Bars and beats as bar.beat.tick, counted from 1 (1.1.000 is the start).
pub fn format_bars(samples: f64, rate: f64, t: &Tempo) -> String {
    let neg = samples < 0.0;
    let at = bar_at(t, seconds_to_beats(t, samples.abs() / rate));
    let mut bar = at.bar;
    let mut whole = (at.beat + 1e-9).floor();
    let mut tick = js_round((at.beat - whole) * TICKS_PER_BEAT);
    if tick >= TICKS_PER_BEAT {
        whole += 1.0;
        tick = 0.0;
    }
    if whole >= at.seg.beats_per_bar {
        bar += 1.0;
        whole = 0.0;
    }
    let body = format!("{}.{}.{:0>3}", bar + 1.0, whole + 1.0, tick);
    if neg {
        format!("-{body}")
    } else {
        body
    }
}

fn digits(s: &[char], i: &mut usize) -> Option<f64> {
    let start = *i;
    while *i < s.len() && s[*i].is_ascii_digit() {
        *i += 1;
    }
    if *i == start {
        return None;
    }
    s[start..*i].iter().collect::<String>().parse().ok()
}

fn sep(s: &[char], i: usize) -> bool {
    matches!(s.get(i), Some('.' | ':' | '|'))
}

/// Samples for a bar.beat.tick time (bar and beat from 1; the tick part may be
/// left off). None when it is not one.
pub fn parse_bars(text: &str, rate: f64, t: &Tempo) -> Option<f64> {
    let s: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < s.len() && is_js_space(s[i]) {
        i += 1;
    }
    let bar = digits(&s, &mut i)?;
    let (mut beat, mut tick) = (1.0, 0.0);
    if sep(&s, i) {
        let mut j = i + 1;
        if let Some(b) = digits(&s, &mut j) {
            beat = b;
            i = j;
            if sep(&s, i) {
                let mut k = i + 1;
                if let Some(tk) = digits(&s, &mut k) {
                    tick = tk;
                    i = k;
                }
            }
        }
    }
    while i < s.len() && is_js_space(s[i]) {
        i += 1;
    }
    if i != s.len() || bar < 1.0 || beat < 1.0 {
        return None;
    }
    let start = bar_start(t, bar - 1.0);
    let seg = segment_at_beat(t, start);
    let beats = start + (beat - 1.0 + tick / TICKS_PER_BEAT) * unit_beats(Some(seg.beat_unit));
    Some(js_round(beats_to_seconds(t, beats) * rate))
}

/// Ruler ticks on bars and beats, through tempo and time signature changes.
pub fn bar_ticks(v: Viewport, width: f64, rate: f64, t: &Tempo, min_gap: f64) -> Vec<Tick> {
    let t0 = v.scroll / rate;
    let t1 = (v.scroll + width * v.spp) / rate;
    let segs = segments(t);
    let mut ticks = Vec::new();
    for (i, seg) in segs.iter().enumerate() {
        let seg_end = segs.get(i + 1).map_or(f64::INFINITY, |s| s.seconds);
        if seg_end <= t0 {
            continue;
        }
        if seg.seconds > t1 {
            break;
        }
        let per = seg.beats_per_bar.max(1.0);
        let unit_sec = (unit_beats(Some(seg.beat_unit)) * 60.0) / seg.bpm;
        let px_per_unit = (unit_sec * rate) / v.spp;
        let choices = [
            0.25,
            0.5,
            1.0,
            per,
            per * 2.0,
            per * 4.0,
            per * 8.0,
            per * 16.0,
            per * 32.0,
            per * 64.0,
        ];
        let step = choices
            .iter()
            .copied()
            .find(|b| b * px_per_unit >= min_gap)
            .unwrap_or(per * 128.0);
        let minor = if step >= per {
            if step == per {
                1.0
            } else {
                step / 4.0
            }
        } else {
            step / 2.0
        };
        let bars_per_label = step / per;
        let mut k = ((t0 - seg.seconds) / unit_sec / minor).floor().max(0.0);
        loop {
            let u = k * minor;
            let time = seg.seconds + u * unit_sec;
            if time >= seg_end - 1e-9 || time > t1 + unit_sec * minor {
                break;
            }
            k += 1.0;
            let x = (time * rate - v.scroll) / v.spp;
            if x < -1.0 || x > width + 1.0 {
                continue;
            }
            let bars = (u / per + 1e-9).floor();
            let bar = seg.bar + bars;
            let in_bar = js_round((u - bars * per) * 1000.0) / 1000.0;
            let mut label = None;
            if step >= per {
                if in_bar == 0.0 && bar % bars_per_label == 0.0 {
                    label = Some(format!("{}", bar + 1.0));
                }
            } else if (k - 1.0) % 2.0 == 0.0 && in_bar.fract() == 0.0 {
                label = Some(format!("{}.{}", bar + 1.0, in_bar + 1.0));
            }
            ticks.push(Tick { x, label });
        }
    }
    ticks
}

/// Bar lines (and beat lines when they are at least `min_beat_px` apart)
/// across the view, as `(x, is_bar)`, for grids behind clips.
pub fn grid_lines(
    v: Viewport,
    width: f64,
    rate: f64,
    t: &Tempo,
    min_beat_px: f64,
) -> Vec<(f64, bool)> {
    let t0 = v.scroll / rate;
    let t1 = (v.scroll + width * v.spp) / rate;
    let segs = segments(t);
    let mut out = Vec::new();
    for (i, seg) in segs.iter().enumerate() {
        let seg_end = segs.get(i + 1).map_or(f64::INFINITY, |s| s.seconds);
        if seg_end <= t0 {
            continue;
        }
        if seg.seconds > t1 {
            break;
        }
        let per = seg.beats_per_bar.max(1.0);
        let unit_sec = (unit_beats(Some(seg.beat_unit)) * 60.0) / seg.bpm;
        let px_per_unit = (unit_sec * rate) / v.spp;
        let beats = px_per_unit >= min_beat_px;
        let every = if beats {
            1.0
        } else {
            per * 1.0_f64.max(
                2.0_f64.powf(
                    (1.0_f64.max(min_beat_px / (px_per_unit * per)))
                        .log2()
                        .ceil(),
                ),
            )
        };
        let mut k = ((t0 - seg.seconds) / unit_sec / every).floor().max(0.0);
        loop {
            let u = k * every;
            let time = seg.seconds + u * unit_sec;
            if time >= seg_end - 1e-9 || time > t1 {
                break;
            }
            k += 1.0;
            let x = (time * rate - v.scroll) / v.spp;
            if x < -1.0 {
                continue;
            }
            out.push((x, (u / per - js_round(u / per)).abs() < 1e-6));
        }
    }
    out
}
