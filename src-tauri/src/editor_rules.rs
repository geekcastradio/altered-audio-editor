// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Small Waveform and Multitrack editing rules: Follow Playhead, grabbing a
//! time selection's border, and silencing a span with a volume dip. The Rust
//! copy of `src/panels/editor/follow.ts`, `selection-edge.ts` and the
//! `silenceSpan` function of `silence-span.ts` (with `levelAt` from
//! `envelope.ts`); all are tested against `shared-fixtures/editor-rules.json`.

use crate::js_compat::js_round;

/// What a view change does to Follow Playhead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowEffect {
    Release,
    Hold,
    None,
}

/// The view as it was: samples per pixel and scroll position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub spp: f64,
    pub scroll: f64,
}

/// A view change made by the person. Fields that did not change are None.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewChange {
    pub spp: Option<f64>,
    pub scroll: Option<f64>,
    pub playhead: Option<f64>,
}

/// What a view change does to the following. A move of the playhead brings the
/// following back. A zoom changes nothing. A scroll holds it.
pub fn follow_effect(now: View, change: ViewChange) -> FollowEffect {
    if change.playhead.is_some() {
        return FollowEffect::Release;
    }
    if change.spp.is_some_and(|s| s != now.spp) {
        return FollowEffect::None;
    }
    if change.scroll.is_some_and(|s| s != now.scroll) {
        return FollowEffect::Hold;
    }
    FollowEffect::None
}

/// The farthest the Waveform editor's view may scroll. While recording, the
/// end it may reach is the end of the new audio. Following a recording pages
/// the view so the end sits `lead` pixels from the left edge.
pub fn max_scroll(
    total: f64,
    width: f64,
    spp: f64,
    live_end: Option<f64>,
    following: bool,
    lead: Option<f64>,
) -> f64 {
    let lead = lead.unwrap_or(20.0);
    let end = total.max(live_end.unwrap_or(0.0));
    let shown = if following && live_end.is_some() {
        lead
    } else {
        width
    };
    (end - shown * spp).max(0.0)
}

/// How close, in pixels, the pointer must be to a selection border to grab it.
pub const EDGE_GRAB_PX: f64 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Start,
    End,
}

/// Which border of a time selection the pointer is over. When both borders are
/// in reach, the closer one wins, and a tie goes to the side the pointer is
/// outside of.
pub fn edge_at(sel: Option<(f64, f64)>, x: f64, view: View, grab_px: f64) -> Option<Edge> {
    let (start, end) = sel?;
    if end <= start || view.spp <= 0.0 {
        return None;
    }
    let xs = (start - view.scroll) / view.spp;
    let xe = (end - view.scroll) / view.spp;
    let ds = (x - xs).abs();
    let de = (x - xe).abs();
    let in_s = ds <= grab_px;
    let in_e = de <= grab_px;
    if in_s && in_e {
        if ds < de {
            return Some(Edge::Start);
        }
        if de < ds {
            return Some(Edge::End);
        }
        return Some(if x < xs { Edge::Start } else { Edge::End });
    }
    if in_s {
        Some(Edge::Start)
    } else if in_e {
        Some(Edge::End)
    } else {
        None
    }
}

/// The anchor for dragging this border: the border that stays where it is.
pub fn anchor_for(sel: (f64, f64), edge: Edge) -> f64 {
    match edge {
        Edge::Start => sel.1,
        Edge::End => sel.0,
    }
}

/// The lowest volume level, in dB.
pub const DB_MIN: f64 = -60.0;

/// One point of a clip's volume envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvelopePoint {
    /// Samples from the clip start.
    pub position: f64,
    pub value: f64,
    pub curve: String,
}

/// Level in dB at `pos` (samples from the clip start). 0 dB when there are no points.
pub fn level_at(points: &[EnvelopePoint], pos: f64) -> f64 {
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

/// The volume points of a clip `len` samples long with `from` to `to` (samples
/// from its start) silenced, with `ramp` samples of fade on each side. None
/// when the span misses the clip.
pub fn silence_span(
    points: &[EnvelopePoint],
    len: f64,
    from: f64,
    to: f64,
    ramp: f64,
) -> Option<Vec<EnvelopePoint>> {
    let lo = js_round(from).max(0.0);
    let hi = js_round(to).min(len);
    if hi <= lo {
        return None;
    }
    let r = js_round(ramp).max(0.0);
    let in_l = r.min(lo);
    let in_r = r.min(len - hi);
    let pt = |position: f64, value: f64| EnvelopePoint {
        position,
        value,
        curve: "linear".into(),
    };
    let mut out: Vec<EnvelopePoint> = points
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
