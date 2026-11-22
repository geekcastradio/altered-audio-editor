// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Timeline view rules: zoom and scroll, the ruler, time text, peak level
//! picking, the overview strip and track heights. The Rust copy of the pure
//! parts of `src/panels/editor/timeline.ts`; both are tested against
//! `shared-fixtures/timeline.json`. Bars and beats, the display format switch
//! and anything that reads settings stay in TypeScript.

use crate::js_compat::{js_number_to_string, js_round, js_to_fixed};

pub const MIN_SPP: f64 = 1.0 / 32.0;
pub const MAX_SPP: f64 = 1_048_576.0;
pub const DEFAULT_MAX_SECONDS: f64 = 4.0 * 3600.0;
pub const TIMECODE_FPS: f64 = 30.0;
pub const COLLAPSED_H: f64 = 30.0;
pub const MIN_TRACK_H: f64 = 48.0;

/// Samples per pixel and the sample at the left edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub spp: f64,
    pub scroll: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    pub x: f64,
    pub label: Option<String>,
}

pub fn x_of(sample: f64, v: Viewport) -> f64 {
    (sample - v.scroll) / v.spp
}

pub fn sample_at(x: f64, v: Viewport) -> f64 {
    0.0_f64.max(js_round(v.scroll + x * v.spp))
}

/// Pixels per frame to scroll while a drag sits near an edge.
pub fn edge_scroll_px(x: f64, left: f64, right: f64, band: f64) -> f64 {
    if x > right - band {
        return 40.0_f64.min(4.0 + (x - (right - band)) / 3.0);
    }
    if x < left + band {
        return -(40.0_f64.min(4.0 + (left + band - x) / 3.0));
    }
    0.0
}

/// The most samples per pixel worth allowing for this width and content.
pub fn max_spp_for(width: f64, content_end: f64, rate: f64) -> f64 {
    let span = (DEFAULT_MAX_SECONDS * rate).max(content_end * 1.1);
    (MAX_SPP * 8.0).min(MIN_SPP.max(span / width.max(1.0)))
}

/// Zooms by `factor` (above 1 zooms out) keeping the sample under `anchor_x`.
pub fn zoom_around(
    v: Viewport,
    factor: f64,
    anchor_x: f64,
    max_spp: f64,
    min_spp: f64,
) -> Viewport {
    let spp = max_spp.min(min_spp.max(v.spp * factor));
    let anchor = v.scroll + anchor_x * v.spp;
    Viewport {
        spp,
        scroll: 0.0_f64.max(anchor - anchor_x * spp),
    }
}

/// Fits `length` samples into `width` pixels with `room` to spare.
pub fn fit(length: f64, width: f64, room: f64) -> Viewport {
    if length <= 0.0 || width <= 0.0 {
        return Viewport {
            spp: 1024.0,
            scroll: 0.0,
        };
    }
    Viewport {
        spp: MAX_SPP.min(MIN_SPP.max((length * room) / width)),
        scroll: 0.0,
    }
}

/// Seconds between labeled ruler ticks, from fine to coarse.
const STEPS: [f64; 29] = [
    0.0001, 0.0002, 0.0005, 0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0,
    10.0, 15.0, 20.0, 30.0, 60.0, 120.0, 180.0, 300.0, 600.0, 900.0, 1200.0, 1800.0, 3600.0,
    7200.0,
];

/// Unlabeled ticks between labels, where 5 would land on odd times.
const MINOR_DIVISIONS: [(f64, f64); 12] = [
    (2.0, 4.0),
    (15.0, 3.0),
    (20.0, 4.0),
    (30.0, 3.0),
    (60.0, 4.0),
    (120.0, 4.0),
    (180.0, 3.0),
    (900.0, 3.0),
    (1200.0, 4.0),
    (1800.0, 3.0),
    (3600.0, 4.0),
    (7200.0, 4.0),
];

/// Ruler ticks for the visible range. Labeled ticks are at least `min_gap`
/// pixels apart.
pub fn ruler_ticks(v: Viewport, width: f64, rate: f64, min_gap: f64) -> Vec<Tick> {
    let px_per_sec = rate / v.spp;
    let step = STEPS
        .iter()
        .copied()
        .find(|s| s * px_per_sec >= min_gap)
        .unwrap_or(14400.0);
    let div = MINOR_DIVISIONS
        .iter()
        .find(|(s, _)| *s == step)
        .map_or(5.0, |(_, d)| *d);
    let minor = step / div;
    let start_sec = v.scroll / rate;
    let end_sec = start_sec + (width * v.spp) / rate;
    let first = (start_sec / minor).floor() * minor;
    let n = ((end_sec - first) / minor).ceil() + 1.0;
    let per_major = js_round(step / minor);
    let first_index = js_round(first / minor);
    let mut ticks = Vec::new();
    let mut i = 0.0;
    while i <= n {
        let k = first_index + i;
        let t = k * minor;
        let x = (t * rate - v.scroll) / v.spp;
        i += 1.0;
        if x < -1.0 || x > width + 1.0 {
            continue;
        }
        ticks.push(Tick {
            x,
            label: (k % per_major == 0.0).then(|| format_time(t, step)),
        });
    }
    ticks
}

fn pad(s: String, width: usize) -> String {
    format!("{s:0>width$}")
}

/// Time for display. Shows fractions of a second only when the step needs them.
pub fn format_time(seconds: f64, step: f64) -> String {
    let neg = seconds < 0.0;
    let mut s = seconds.abs();
    let decimals: usize = if step < 0.001 {
        4
    } else if step < 0.01 {
        3
    } else if step < 0.1 {
        2
    } else if step < 1.0 {
        1
    } else {
        0
    };
    let scale = 10f64.powi(decimals as i32);
    s = js_round(s * scale) / scale;
    let h = (s / 3600.0).floor();
    let m = ((s % 3600.0) / 60.0).floor();
    let sec = s % 60.0;
    let sec_text = if decimals > 0 {
        pad(js_to_fixed(sec, decimals), decimals + 3)
    } else {
        pad(js_number_to_string(sec.floor()), 2)
    };
    let body = if h > 0.0 {
        format!(
            "{}:{}:{sec_text}",
            js_number_to_string(h),
            pad(js_number_to_string(m), 2)
        )
    } else {
        format!("{}:{sec_text}", js_number_to_string(m))
    };
    if neg {
        format!("-{body}")
    } else {
        body
    }
}

/// Full-precision time for readouts: 1:15:05.399 or 0:42.120.
pub fn format_clock(samples: f64, rate: f64) -> String {
    format_time(samples / rate, 0.001)
}

/// Timecode as hh:mm:ss:ff (non-drop frame).
pub fn format_timecode(samples: f64, rate: f64, fps: f64) -> String {
    let neg = samples < 0.0;
    let total_frames = ((samples.abs() * fps) / rate).floor();
    let f = total_frames % fps;
    let total_sec = (total_frames / fps).floor();
    let two = |n: f64| pad(js_number_to_string(n), 2);
    let body = format!(
        "{}:{}:{}:{}",
        two((total_sec / 3600.0).floor()),
        two(((total_sec % 3600.0) / 60.0).floor()),
        two(total_sec % 60.0),
        two(f)
    );
    if neg {
        format!("-{body}")
    } else {
        body
    }
}

/// The index of the coarsest level whose blocks are no wider than one pixel's
/// worth of source frames, given each level's frames per block.
pub fn pick_level(blocks: &[f64], frames_per_pixel: f64) -> Option<usize> {
    let mut order: Vec<usize> = (0..blocks.len()).collect();
    order.sort_by(|&a, &b| {
        blocks[a]
            .partial_cmp(&blocks[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut best = order.first().copied();
    for &i in &order {
        if blocks[i] <= frames_per_pixel {
            best = Some(i);
        }
    }
    best
}

/// Min and max (as -1..1) of channel `ch` over source frames [from, to).
pub fn peak_range(
    data: &[i16],
    block: f64,
    channels: usize,
    ch: usize,
    from: f64,
    to: f64,
) -> (f64, f64) {
    let stride = channels * 2;
    let blocks = (data.len() / stride) as f64;
    let mut b0 = 0.0_f64.max((from / block).floor());
    let b1 = blocks.min((b0 + 1.0).max((to / block).ceil()));
    if b0 >= blocks {
        return (0.0, 0.0);
    }
    let mut lo = 32767_i32;
    let mut hi = -32768_i32;
    while b0 < b1 {
        let i = b0 as usize * stride + ch * 2;
        let mn = i32::from(data[i]);
        let mx = i32::from(data[i + 1]);
        lo = lo.min(mn);
        hi = hi.max(mx);
        b0 += 1.0;
    }
    if lo > hi {
        return (0.0, 0.0);
    }
    (f64::from(lo) / 32767.0, f64::from(hi) / 32767.0)
}

/// The overview strip's window (x, width) for what the lanes show.
pub fn overview_window(v: Viewport, lanes_w: f64, total: f64, strip_w: f64) -> (f64, f64) {
    let k = strip_w / total.max(1.0);
    (v.scroll * k, 6.0_f64.max(lanes_w * v.spp * k))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
}

/// The view after dragging an edge of the overview window to `x`; the
/// opposite edge stays put, which zooms the lanes.
pub fn drag_overview_edge(
    v: Viewport,
    edge: Edge,
    x: f64,
    lanes_w: f64,
    total: f64,
    strip_w: f64,
) -> Viewport {
    let per_px = total / strip_w.max(1.0);
    let start = v.scroll;
    let end = v.scroll + lanes_w * v.spp;
    let at = 0.0_f64.max(x * per_px);
    let min_span = MIN_SPP * lanes_w;
    let (mut a, mut b) = (start, end);
    match edge {
        Edge::Left => a = at.min(end - min_span),
        Edge::Right => b = at.max(start + min_span),
    }
    let spp = MAX_SPP.min(MIN_SPP.max((b - a) / lanes_w.max(1.0)));
    let scroll = 0.0_f64.max(if edge == Edge::Left {
        b - spp * lanes_w
    } else {
        a
    });
    Viewport { spp, scroll }
}

/// Color for a track that has none of its own: the clip palette, by row.
pub fn default_track_color(row: i64) -> String {
    format!("--clip-{}", row.rem_euclid(6) + 1)
}

/// A track's height on screen: folded tracks show only their name row; open
/// ones are never shorter than `MIN_TRACK_H`.
pub fn track_height(height: f64, collapsed: bool) -> f64 {
    if collapsed {
        COLLAPSED_H
    } else {
        MIN_TRACK_H.max(height)
    }
}
