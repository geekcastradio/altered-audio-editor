// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Clip envelope rules: level maths, snapping, labels, point editing, the
//! Silence Time Selection dip, the stretch percent and time-entry rules, and
//! the session tab rules. The Rust copy of the pure parts of
//! `src/panels/editor/envelope.ts`, `silence-span.ts`, `stretch.ts` and
//! `session-tabs.ts`; all are tested against
//! `shared-fixtures/envelope-batch.json`. Functions that read the session or
//! the effect catalog stay in TypeScript.

use crate::js_compat::{js_number_to_string, js_round, js_to_fixed, js_trim};
use serde::{Deserialize, Serialize};

pub const DB_MIN: f64 = -60.0;
pub const DB_MAX: f64 = 12.0;
pub const SNAP_DB: f64 = 0.75;
pub const SNAP_PAN: f64 = 4.0;
/// Length of the ramp on each side of a silence, in seconds.
pub const SILENCE_RAMP_SECONDS: f64 = 0.005;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvKind {
    Volume,
    Pan,
    Fx,
}

impl EnvKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "volume" => Some(Self::Volume),
            "pan" => Some(Self::Pan),
            "fx" => Some(Self::Fx),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvRange {
    pub top: f64,
    pub bottom: f64,
    pub step: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub position: f64,
    pub value: f64,
    pub curve: String,
}

/// The range for a kind; an effect lane without a known parameter is 0 to 1.
pub fn range_of(kind: EnvKind) -> EnvRange {
    match kind {
        EnvKind::Volume => EnvRange {
            top: DB_MAX,
            bottom: DB_MIN,
            step: 0.1,
        },
        EnvKind::Pan => EnvRange {
            top: -100.0,
            bottom: 100.0,
            step: 1.0,
        },
        EnvKind::Fx => EnvRange {
            top: 1.0,
            bottom: 0.0,
            step: 0.01,
        },
    }
}

pub fn value_to_y(v: f64, top: f64, height: f64, r: EnvRange) -> f64 {
    let lo = r.top.min(r.bottom);
    let hi = r.top.max(r.bottom);
    let c = lo.max(hi.min(v));
    top + ((c - r.top) / (r.bottom - r.top)) * height
}

pub fn y_to_value(y: f64, top: f64, height: f64, r: EnvRange) -> f64 {
    let f = 0.0_f64.max(1.0_f64.min((y - top) / height.max(1.0)));
    let v = r.top + f * (r.bottom - r.top);
    js_round(v / r.step) * r.step
}

pub fn snap_value(kind: EnvKind, v: f64) -> f64 {
    if kind == EnvKind::Volume && v.abs() <= SNAP_DB {
        return 0.0;
    }
    if kind == EnvKind::Pan && v.abs() <= SNAP_PAN {
        return 0.0;
    }
    v
}

pub fn clamp_value(v: f64, r: EnvRange) -> f64 {
    r.top.min(r.bottom).max(r.top.max(r.bottom).min(v))
}

/// The text for a value: dB for volume, L/C/R for pan, a trimmed number for an
/// effect parameter. Finite values only.
pub fn format_value(kind: EnvKind, v: f64, unit: &str) -> String {
    match kind {
        EnvKind::Fx => {
            let t = if v.abs() >= 100.0 {
                js_to_fixed(v, 0)
            } else if v.abs() >= 10.0 {
                js_to_fixed(v, 1)
            } else {
                js_to_fixed(v, 2)
            };
            if unit.is_empty() {
                t
            } else {
                format!("{t} {unit}")
            }
        }
        EnvKind::Volume => format!("{}{} dB", if v > 0.0 { "+" } else { "" }, js_to_fixed(v, 1)),
        EnvKind::Pan => {
            let p = js_round(v);
            if p == 0.0 {
                "C".to_string()
            } else if p < 0.0 {
                format!("L {}", js_number_to_string(-p))
            } else {
                format!("R {}", js_number_to_string(p))
            }
        }
    }
}

/// The level of a line at `pos`: held before the first point and after the last.
pub fn level_at(points: &[Point], pos: f64) -> f64 {
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return 0.0;
    };
    if pos <= first.position {
        return first.value;
    }
    if pos >= last.position {
        return last.value;
    }
    for i in 1..points.len() {
        let b = &points[i];
        if pos <= b.position {
            let a = &points[i - 1];
            let t = (pos - a.position) / (b.position - a.position).max(1.0);
            return a.value + (b.value - a.value) * t;
        }
    }
    last.value
}

/// dB to a y position: +12 at the top, -60 at the bottom.
pub fn db_to_y(db: f64, top: f64, height: f64) -> f64 {
    let d = DB_MIN.max(DB_MAX.min(db));
    top + ((DB_MAX - d) / (DB_MAX - DB_MIN)) * height
}

pub fn y_to_db(y: f64, top: f64, height: f64) -> f64 {
    let f = 0.0_f64.max(1.0_f64.min((y - top) / height.max(1.0)));
    js_round((DB_MAX - f * (DB_MAX - DB_MIN)) * 10.0) / 10.0
}

/// The points for the part of a clip from `from` to `to`, moved so `from`
/// becomes 0, with the levels at both ends kept.
pub fn slice_volume(points: &[Point], from: f64, to: f64) -> Vec<Point> {
    let Some(first) = points.first() else {
        return Vec::new();
    };
    let curve = first.curve.clone();
    let mut out = vec![Point {
        position: 0.0,
        value: level_at(points, from),
        curve: curve.clone(),
    }];
    out.extend(
        points
            .iter()
            .filter(|p| p.position > from && p.position < to)
            .map(|p| Point {
                position: p.position - from,
                ..p.clone()
            }),
    );
    out.push(Point {
        position: (to - from).max(1.0),
        value: level_at(points, to),
        curve,
    });
    out
}

/// Adds a point at `pos`, or moves the one already there. Returns the new list
/// and the point's index.
pub fn add_point(points: &[Point], pos: f64, value: f64) -> (Vec<Point>, usize) {
    let p = js_round(pos).max(0.0);
    let mut next: Vec<Point> = points.iter().filter(|x| x.position != p).cloned().collect();
    next.push(Point {
        position: p,
        value,
        curve: "linear".to_string(),
    });
    next.sort_by(|a, b| {
        a.position
            .partial_cmp(&b.position)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let i = next.iter().position(|x| x.position == p).unwrap_or(0);
    (next, i)
}

/// Moves point `i` to `pos` and `value`, keeping it between its neighbors.
pub fn move_point(
    points: &[Point],
    i: usize,
    pos: f64,
    value: f64,
    len: f64,
    r: EnvRange,
) -> Vec<Point> {
    let lo = if i > 0 {
        points[i - 1].position + 1.0
    } else {
        0.0
    };
    let hi = if i + 1 < points.len() {
        points[i + 1].position - 1.0
    } else {
        len
    };
    let mut next = points.to_vec();
    next[i].position = js_round(lo.max(hi.min(pos)));
    next[i].value = clamp_value(value, r);
    next
}

/// Moves the points in `selected` together by `d_pos`, held so the group stays
/// in order, then gives each the value `value_for` returns.
pub fn move_points(
    points: &[Point],
    selected: &[usize],
    d_pos: f64,
    value_for: impl Fn(&Point) -> f64,
    len: f64,
    r: EnvRange,
) -> Vec<Point> {
    let mut lo = f64::NEG_INFINITY;
    let mut hi = f64::INFINITY;
    for (k, p) in points.iter().enumerate() {
        if !selected.contains(&k) {
            continue;
        }
        let prev = if k > 0 { points.get(k - 1) } else { None };
        let next = points.get(k + 1);
        let below = match prev {
            Some(q) if !selected.contains(&(k - 1)) => q.position + 1.0,
            _ => 0.0,
        };
        let above = match next {
            Some(q) if !selected.contains(&(k + 1)) => q.position - 1.0,
            _ => len,
        };
        lo = lo.max(below - p.position);
        hi = hi.min(above - p.position);
    }
    let d = js_round(lo.max(hi.min(d_pos)));
    points
        .iter()
        .enumerate()
        .map(|(k, p)| {
            if selected.contains(&k) {
                Point {
                    position: p.position + d,
                    value: clamp_value(value_for(p), r),
                    curve: p.curve.clone(),
                }
            } else {
                p.clone()
            }
        })
        .collect()
}

/// Label of the envelope chip in a clip's title bar.
pub fn chip_label(show_volume: bool, show_pan: bool, show_fx: bool) -> String {
    let mut on = Vec::new();
    if show_volume {
        on.push("Vol");
    }
    if show_pan {
        on.push("Pan");
    }
    if show_fx {
        on.push("FX");
    }
    if on.len() > 1 {
        return on.join(" + ");
    }
    if show_volume {
        "Volume".to_string()
    } else if show_pan {
        "Pan".to_string()
    } else if show_fx {
        "Effect".to_string()
    } else {
        "Envelopes".to_string()
    }
}

/// Where the chip sits in a clip's title bar: (x, width), or None when the
/// clip is too narrow.
pub fn chip_box(x0: f64, x1: f64, label: &str) -> Option<(f64, f64)> {
    let w = js_round(label.encode_utf16().count() as f64 * 5.6 + 22.0);
    if x1 - x0 < w + 70.0 {
        return None;
    }
    Some((x1 - w - 14.0, w))
}

/// The volume points of a clip `len` samples long with `from` to `to` silenced.
/// None when the span misses the clip.
pub fn silence_span(
    points: &[Point],
    len: f64,
    from: f64,
    to: f64,
    ramp: f64,
) -> Option<Vec<Point>> {
    let lo = js_round(from).max(0.0);
    let hi = js_round(to).min(len);
    if hi <= lo {
        return None;
    }
    let r = js_round(ramp).max(0.0);
    let in_l = r.min(lo);
    let in_r = r.min(len - hi);
    let pt = |position: f64, value: f64| Point {
        position,
        value,
        curve: "linear".to_string(),
    };
    let mut out: Vec<Point> = points
        .iter()
        .filter(|p| p.position < lo - in_l)
        .cloned()
        .collect();
    if in_l > 0.0 {
        out.push(pt(lo - in_l, level_at(points, lo - in_l)));
    }
    out.push(pt(lo, DB_MIN));
    if hi > lo {
        out.push(pt(hi, DB_MIN));
    }
    if in_r > 0.0 {
        out.push(pt(hi + in_r, level_at(points, hi + in_r)));
    }
    out.extend(points.iter().filter(|p| p.position > hi + in_r).cloned());
    Some(out)
}

/// A stretch ratio as a percent, like "125%" or "99.5%".
pub fn percent(ratio: f64) -> String {
    format!("{}%", js_number_to_string(js_round(ratio * 1000.0) / 10.0))
}

/// Reads a length typed as seconds or `m:ss` or `h:mm:ss`, with an optional
/// decimal part on the last number. None when it is not a length above zero.
pub fn parse_seconds(text: &str) -> Option<f64> {
    let t = js_trim(text);
    let (main, frac) = match t.split_once('.') {
        Some((m, f)) => (m, Some(f)),
        None => (t, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if let Some(f) = frac {
        if !digits(f) {
            return None;
        }
    }
    let parts: Vec<&str> = main.split(':').collect();
    if parts.len() > 3 || !digits(parts[0]) {
        return None;
    }
    if parts[1..].iter().any(|p| !digits(p) || p.len() > 2) {
        return None;
    }
    let mut secs = 0.0;
    for (i, p) in parts.iter().enumerate() {
        let text = match (i == parts.len() - 1, frac) {
            (true, Some(f)) => format!("{p}.{f}"),
            _ => (*p).to_string(),
        };
        secs = secs * 60.0 + text.parse::<f64>().ok()?;
    }
    (secs.is_finite() && secs > 0.0).then_some(secs)
}

/// Tabs show once a second session is open.
pub fn show_tabs(count: usize) -> bool {
    count >= 2
}

/// The tab to switch to when tab `id` closes: its right neighbor, else its
/// left, else none.
pub fn neighbor_of(ids: &[String], id: &str) -> Option<String> {
    let i = ids.iter().position(|x| x == id)?;
    if ids.len() < 2 {
        return None;
    }
    ids.get(i + 1)
        .or_else(|| i.checked_sub(1).and_then(|j| ids.get(j)))
        .cloned()
}

/// What a tab says: the session's name, with a star while it has unsaved changes.
pub fn tab_label(name: &str, dirty: bool) -> String {
    if dirty {
        format!("{name} *")
    } else {
        name.to_string()
    }
}
