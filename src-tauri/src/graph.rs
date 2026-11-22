// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The response graph's maths: where a frequency and a level sit on the graph,
//! and back. The Rust copy of `src/panels/voice-chain/graph.ts`; both are
//! tested against `shared-fixtures/align-graph.json`.

pub const GRAPH_LOW: f64 = 20.0;
pub const GRAPH_HIGH: f64 = 20_000.0;

/// 0 to 1 across the graph.
pub fn x_of(freq: f64) -> f64 {
    (freq.clamp(GRAPH_LOW, GRAPH_HIGH) / GRAPH_LOW).ln() / (GRAPH_HIGH / GRAPH_LOW).ln()
}

pub fn freq_at(x: f64) -> f64 {
    GRAPH_LOW * (GRAPH_HIGH / GRAPH_LOW).powf(x.clamp(0.0, 1.0))
}

/// 0 (top) to 1 (bottom) for `db` in `top..bottom`.
pub fn y_of(db: f64, top: f64, bottom: f64) -> f64 {
    ((top - db) / (top - bottom)).clamp(0.0, 1.0)
}

pub fn db_at(y: f64, top: f64, bottom: f64) -> f64 {
    top - y.clamp(0.0, 1.0) * (top - bottom)
}

/// The level range shown: ±24 dB, opened out to fit deep cuts and big boosts.
/// `gains` is the (min, max) of each visible gain control; `point_dbs` the
/// level of each point on the graph.
pub fn range(point_dbs: &[f64], gains: &[(f64, f64)]) -> (f64, f64) {
    let mut top: f64 = 24.0;
    let mut bottom: f64 = -24.0;
    for &(min, max) in gains {
        top = top.max(max.min(40.0));
        bottom = bottom.min(min.max(-60.0));
    }
    for &db in point_dbs {
        bottom = bottom.min(db.max(-60.0));
    }
    (top, (bottom / 6.0).floor() * 6.0)
}
