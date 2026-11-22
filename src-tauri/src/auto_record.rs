// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Recording automation: the maths of one take. The Rust copy of the pure
//! parts of `src/app/auto-record.ts` (the `Take` state machine, the lane
//! tolerance and the glide), tested against `shared-fixtures/auto-record.json`.
//!
//! While playback runs, the controls of tracks and buses in Write, Latch or
//! Touch mode feed moves into a take; when playback stops it hands back each
//! lane's new points. Positions are session samples at the playhead.

use crate::automation::{auto_at, thin};
use crate::envelope::Point;
use crate::js_compat::js_round;

/// How long the lane takes to come back after a release, in seconds.
pub const GLIDE_SECONDS: f64 = 0.02;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordMode {
    Write,
    Latch,
    Touch,
}

impl RecordMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "write" => Some(Self::Write),
            "latch" => Some(Self::Latch),
            "touch" => Some(Self::Touch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct Segment {
    from: f64,
    /// None while it is still writing.
    to: Option<f64>,
    points: Vec<Point>,
}

#[derive(Debug, Clone)]
struct Lane {
    mode: RecordMode,
    base: Vec<Point>,
    rest: f64,
    tolerance: f64,
    segments: Vec<Segment>,
}

fn pt(position: f64, value: f64) -> Point {
    Point {
        position: js_round(position).max(0.0),
        value,
        curve: "linear".to_string(),
    }
}

/// One take. Lanes keep the order they were armed in.
#[derive(Debug, Clone)]
pub struct Take {
    rate: f64,
    lanes: Vec<(String, Lane)>,
}

impl Take {
    /// `rate` is the session's sample rate, for the glide after a release.
    pub fn new(rate: f64) -> Self {
        Self {
            rate,
            lanes: Vec::new(),
        }
    }

    fn lane(&self, key: &str) -> Option<&Lane> {
        self.lanes.iter().find(|(k, _)| k == key).map(|(_, l)| l)
    }

    fn lane_mut(&mut self, key: &str) -> Option<&mut Lane> {
        self.lanes
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, l)| l)
    }

    fn begin(lane: &mut Lane, pos: f64, value: f64) {
        lane.segments.push(Segment {
            from: js_round(pos).max(0.0),
            to: None,
            points: vec![pt(pos, value)],
        });
    }

    /// Starts watching a lane. Write mode with a lane that has points begins
    /// writing at `pos` at once.
    #[allow(clippy::too_many_arguments)]
    pub fn arm(
        &mut self,
        key: &str,
        mode: RecordMode,
        base: &[Point],
        rest: f64,
        tolerance: f64,
        pos: f64,
        value: f64,
    ) {
        if self.has(key) {
            return;
        }
        let mut lane = Lane {
            mode,
            base: base.to_vec(),
            rest,
            tolerance,
            segments: Vec::new(),
        };
        if mode == RecordMode::Write && !base.is_empty() {
            Self::begin(&mut lane, pos, value);
        }
        self.lanes.push((key.to_string(), lane));
    }

    pub fn has(&self, key: &str) -> bool {
        self.lane(key).is_some()
    }

    /// True while the lane is being written right now.
    pub fn writing(&self, key: &str) -> bool {
        self.lane(key)
            .and_then(|l| l.segments.last())
            .is_some_and(|s| s.to.is_none())
    }

    /// A control was touched (Touch, Latch) or moved for the first time (Write
    /// with no lane yet).
    pub fn grab(&mut self, key: &str, pos: f64, value: f64) {
        if self.writing(key) {
            return;
        }
        let Some(lane) = self.lane_mut(key) else {
            return;
        };
        // Latch writes from the first touch on; a later touch only continues it.
        if lane.mode == RecordMode::Latch && !lane.segments.is_empty() {
            return;
        }
        Self::begin(lane, pos, value);
    }

    /// The control moved to `value` at `pos`. Moves outside a write are not
    /// recorded.
    pub fn set(&mut self, key: &str, pos: f64, value: f64) {
        let writing = self.writing(key);
        let Some(lane) = self.lane_mut(key) else {
            return;
        };
        if !writing {
            if lane.mode == RecordMode::Write {
                Self::begin(lane, pos, value);
            } else {
                return;
            }
        }
        let seg = lane.segments.last_mut().unwrap();
        let last_pos = seg.points.last().unwrap().position;
        let p = js_round(pos).max(0.0);
        if p == last_pos {
            *seg.points.last_mut().unwrap() = pt(p, value);
        } else if p > last_pos {
            seg.points.push(pt(p, value));
        }
    }

    /// The control was let go. Touch stops writing; Latch and Write keep going.
    pub fn release(&mut self, key: &str, pos: f64) {
        let writing = self.writing(key);
        let Some(lane) = self.lane_mut(key) else {
            return;
        };
        if lane.mode != RecordMode::Touch || !writing {
            return;
        }
        let seg = lane.segments.last_mut().unwrap();
        let last = seg.points.last().unwrap().clone();
        seg.points.push(pt(pos.max(last.position), last.value));
        seg.to = Some(pos.max(last.position));
    }

    /// Playback stopped at `pos`: every lane's points, merged with what was
    /// there. Lanes nothing was written to are left out.
    pub fn finish(&mut self, pos: f64) -> Vec<(String, Vec<Point>)> {
        let rate = self.rate;
        let mut out = Vec::new();
        for (key, lane) in self.lanes.iter_mut() {
            for seg in lane.segments.iter_mut() {
                if seg.to.is_none() {
                    let last = seg.points.last().unwrap().clone();
                    seg.points.push(pt(pos.max(last.position), last.value));
                    seg.to = Some(pos.max(last.position));
                }
            }
            if lane.segments.is_empty() {
                continue;
            }
            out.push((key.clone(), merge(rate, lane)));
        }
        out
    }
}

fn merge(rate: f64, lane: &Lane) -> Vec<Point> {
    let glide = js_round(rate * GLIDE_SECONDS).max(1.0);
    struct Span {
        from: f64,
        to: f64,
        points: Vec<Point>,
    }
    let spans: Vec<Span> = lane
        .segments
        .iter()
        .map(|s| Span {
            from: s.from,
            to: s.to.unwrap_or(s.from),
            points: thin(&s.points, lane.tolerance),
        })
        .collect();
    let inside = |p: f64| spans.iter().any(|s| p >= s.from && p <= s.to);
    // What was there, outside what was written, stays.
    let mut merged: Vec<Point> = lane
        .base
        .iter()
        .filter(|p| !inside(p.position))
        .cloned()
        .collect();
    for s in &spans {
        let last_base = lane.base.last();
        // The lane's value just before the write holds up to it, so the write
        // starts clean.
        if !lane.base.is_empty() && s.from > 0.0 {
            merged.push(pt(
                s.from - 1.0,
                auto_at(&lane.base, s.from - 1.0, lane.rest),
            ));
        }
        merged.extend(s.points.iter().cloned());
        // After it the lane comes back, unless there was nothing further on.
        if let Some(lb) = last_base {
            if lb.position > s.to {
                let back = s.to + glide;
                merged.push(pt(back, auto_at(&lane.base, back, lane.rest)));
            }
        }
    }
    merged.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
    // One point per position: a later one wins.
    let mut unique: Vec<Point> = Vec::new();
    for p in merged {
        match unique.last_mut() {
            Some(prev) if prev.position == p.position => *prev = p,
            _ => unique.push(p),
        }
    }
    unique
}

/// A lane's thinning tolerance: a share of its range, `share` 0 for none.
pub fn tolerance_for(top: f64, bottom: f64, share: f64) -> f64 {
    (top - bottom).abs() * share.max(0.0)
}
