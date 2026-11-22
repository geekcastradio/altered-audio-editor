// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Automation lane rules: lane height and value ranges, mapping between a
//! lane's pixels and its values, snapping, value text, the value at a
//! position, adding and moving points, and thinning recorded points. The Rust
//! copy of the pure parts of `src/panels/editor/automation.ts` and `thin` in
//! `src/app/auto-record.ts`; both are tested against
//! `shared-fixtures/automation.json`.

use crate::envelope::Point;
use crate::js_compat::{js_number_to_string, js_round, js_to_fixed};
use std::collections::HashSet;

pub const AUTO_H: f64 = 54.0;

/// A lane's height: its own, or the standard one.
pub fn lane_height(own: Option<f64>) -> f64 {
    own.unwrap_or(AUTO_H).clamp(24.0, 400.0)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Volume,
    Pan,
    EffectParam {
        slot_id: String,
        param: String,
    },
    /// A 3D Space spot setting: "x", "y", "z", "size", "width", "rotation",
    /// "lfe", "spread" or "spatialize".
    Space(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutoRange {
    /// Value drawn at the top of the lane.
    pub top: f64,
    pub bottom: f64,
    pub step: f64,
    /// Drawn on a log scale (frequencies).
    pub log: bool,
    pub unit: String,
}

fn range(top: f64, bottom: f64, step: f64, unit: &str) -> AutoRange {
    AutoRange {
        top,
        bottom,
        step,
        log: false,
        unit: unit.to_string(),
    }
}

pub fn volume_range() -> AutoRange {
    range(12.0, -60.0, 0.1, "dB")
}

pub fn pan_range() -> AutoRange {
    range(-100.0, 100.0, 1.0, "")
}

/// A spot setting's range: left at the top for left/right (as pan), front,
/// high or more at the top otherwise.
pub fn space_range(p: &str) -> AutoRange {
    match p {
        "x" => range(-1.0, 1.0, 0.01, ""),
        "y" | "z" => range(1.0, -1.0, 0.01, ""),
        "rotation" => range(180.0, -180.0, 1.0, "°"),
        _ => range(1.0, 0.0, 0.01, "%"),
    }
}

/// A stable key for a target, for pickers and lookups.
pub fn target_key(t: &Target) -> String {
    match t {
        Target::EffectParam { slot_id, param } => format!("fx:{slot_id}:{param}"),
        Target::Space(p) => format!("space:{p}"),
        Target::Volume => "volume".to_string(),
        Target::Pan => "pan".to_string(),
    }
}

/// 0 at the top of the lane to 1 at the bottom.
fn share(v: f64, r: &AutoRange) -> f64 {
    let (lo, hi) = (r.top.min(r.bottom), r.top.max(r.bottom));
    let c = v.min(hi).max(lo);
    if r.log {
        return (c / r.top).ln() / (r.bottom / r.top).ln();
    }
    let span = r.bottom - r.top;
    (c - r.top) / if span == 0.0 { 1.0 } else { span }
}

pub fn auto_y(v: f64, r: &AutoRange, top: f64, height: f64) -> f64 {
    top + share(v, r) * height
}

/// A y position to a value, rounded to the lane's step.
pub fn auto_value(y: f64, r: &AutoRange, top: f64, height: f64) -> f64 {
    let f = ((y - top) / height.max(1.0)).clamp(0.0, 1.0);
    let v = if r.log {
        r.top * (r.bottom / r.top).powf(f)
    } else {
        r.top + f * (r.bottom - r.top)
    };
    let s = if r.log {
        let own = v.abs() / 100.0;
        r.step.min(if own == 0.0 || own.is_nan() {
            r.step
        } else {
            own
        })
    } else {
        r.step
    };
    let (lo, hi) = (r.top.min(r.bottom), r.top.max(r.bottom));
    (js_round(v / s) * s).min(hi).max(lo)
}

/// Pulls a dragged value onto 0 dB or center when close (hold Alt to place it
/// freely).
pub fn auto_snap(t: &Target, v: f64) -> f64 {
    match t {
        Target::Volume if v.abs() <= 0.75 => 0.0,
        Target::Pan if v.abs() <= 4.0 => 0.0,
        _ => v,
    }
}

pub fn auto_format(t: &Target, v: f64, r: &AutoRange) -> String {
    let pct = |x: f64| js_number_to_string(js_round(x * 100.0));
    match t {
        Target::Space(p) => match p.as_str() {
            "x" if v.abs() < 0.005 => "Center".to_string(),
            "x" if v < 0.0 => format!("Left {}%", pct(-v)),
            "x" => format!("Right {}%", pct(v)),
            "y" if v.abs() < 0.005 => "Middle".to_string(),
            "y" if v > 0.0 => format!("Front {}%", pct(v)),
            "y" => format!("Back {}%", pct(-v)),
            "rotation" => format!("{}°", js_number_to_string(js_round(v))),
            _ => format!("{}%", pct(v)),
        },
        Target::Volume => {
            if v <= -59.9 {
                "Silent".to_string()
            } else {
                format!("{}{} dB", if v > 0.0 { "+" } else { "" }, js_to_fixed(v, 1))
            }
        }
        Target::Pan => {
            let p = js_round(v);
            if p == 0.0 {
                "C".to_string()
            } else if p < 0.0 {
                format!("L {}", js_number_to_string(-p))
            } else {
                format!("R {}", js_number_to_string(p))
            }
        }
        Target::EffectParam { .. } => {
            let digits = if v.abs() >= 100.0 {
                0
            } else if v.abs() >= 10.0 {
                1
            } else {
                2
            };
            let n = js_to_fixed(v, digits);
            if r.unit.is_empty() {
                n
            } else {
                format!("{n} {}", r.unit)
            }
        }
    }
}

/// The value at timeline sample `pos`, or `rest` with no points.
pub fn auto_at(points: &[Point], pos: f64, rest: f64) -> f64 {
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return rest;
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
            let f = (pos - a.position) / (b.position - a.position).max(1.0);
            return a.value + (b.value - a.value) * f;
        }
    }
    last.value
}

/// Adds a point at `pos`, or moves the one already there. Returns the list and
/// the point's index.
pub fn auto_add(points: &[Point], pos: f64, value: f64) -> (Vec<Point>, usize) {
    let p = js_round(pos).max(0.0);
    let mut next: Vec<Point> = points.iter().filter(|x| x.position != p).cloned().collect();
    next.push(Point {
        position: p,
        value,
        curve: "linear".to_string(),
    });
    next.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
    let index = next.iter().position(|x| x.position == p).unwrap();
    (next, index)
}

/// Moves point `i` to `pos` and `value`, kept between its neighbors.
pub fn auto_move(points: &[Point], i: usize, pos: f64, value: f64) -> Vec<Point> {
    let lo = if i > 0 {
        points[i - 1].position + 1.0
    } else {
        0.0
    };
    let hi = if i + 1 < points.len() {
        points[i + 1].position - 1.0
    } else {
        9_007_199_254_740_991.0
    };
    points
        .iter()
        .enumerate()
        .map(|(k, p)| {
            if k == i {
                Point {
                    position: js_round(pos.min(hi).max(lo)),
                    value,
                    curve: p.curve.clone(),
                }
            } else {
                p.clone()
            }
        })
        .collect()
}

/// Moves the points in `selected` together: `d_pos` samples along (held so the
/// group stays in order, between its unselected neighbors and not before 0),
/// each to the value `value_for` gives it.
pub fn auto_move_many(
    points: &[Point],
    selected: &HashSet<usize>,
    d_pos: f64,
    value_for: impl Fn(&Point) -> f64,
) -> Vec<Point> {
    let mut lo = f64::NEG_INFINITY;
    let mut hi = f64::INFINITY;
    for (k, p) in points.iter().enumerate() {
        if !selected.contains(&k) {
            continue;
        }
        let prev = k.checked_sub(1).and_then(|j| points.get(j));
        let next = points.get(k + 1);
        let floor = match prev {
            Some(pr) if !selected.contains(&(k - 1)) => pr.position + 1.0,
            _ => 0.0,
        };
        lo = lo.max(floor - p.position);
        if let Some(n) = next {
            if !selected.contains(&(k + 1)) {
                hi = hi.min(n.position - 1.0 - p.position);
            }
        }
    }
    let d = js_round(d_pos.min(hi).max(lo));
    points
        .iter()
        .enumerate()
        .map(|(k, p)| {
            if selected.contains(&k) {
                Point {
                    position: p.position + d,
                    value: value_for(p),
                    curve: p.curve.clone(),
                }
            } else {
                p.clone()
            }
        })
        .collect()
}

/// Drops points that lie within `tolerance` of the straight line between the
/// points kept either side of them (Douglas-Peucker on the value axis). The
/// first and last points always stay. Zero keeps everything.
pub fn thin(points: &[Point], tolerance: f64) -> Vec<Point> {
    if tolerance <= 0.0 || points.len() <= 2 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0, points.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let (pa, pb) = (&points[a], &points[b]);
        let mut worst = -1.0;
        let mut at: Option<usize> = None;
        for (i, p) in points.iter().enumerate().take(b).skip(a + 1) {
            let f = if pb.position == pa.position {
                0.0
            } else {
                (p.position - pa.position) / (pb.position - pa.position)
            };
            let d = (p.value - (pa.value + (pb.value - pa.value) * f)).abs();
            if d > worst {
                worst = d;
                at = Some(i);
            }
        }
        if let Some(i) = at.filter(|_| worst > tolerance) {
            keep[i] = true;
            stack.push((a, i));
            stack.push((i, b));
        }
    }
    points
        .iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, p)| p.clone())
        .collect()
}
