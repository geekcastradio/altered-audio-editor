// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Clip > Fit Music to Length: the edit and the words, in Rust.
//!
//! This mirrors `src/panels/editor/musicfit.ts` and is tested against
//! `shared-fixtures/band-phase-fit.json`, which the TypeScript test reads
//! too. The plan itself (which segments to play) comes from the core.

use crate::js_compat::{js_number_to_string, js_round, js_to_fixed, js_to_fixed1};
use serde_json::{json, Value};

/// Crossfade at each join, in seconds. Long enough to hide the cut, short
/// enough to stay on the beat.
pub const FIT_XFADE: f64 = 0.04;

#[derive(Clone, Debug)]
pub struct FitPlan {
    /// Seconds into the file: (start, end).
    pub segments: Vec<(f64, f64)>,
    pub length: f64,
    pub stretch: f64,
    pub joins: u32,
    pub quality: f64,
    pub tempo: f64,
}

/// True when `target` is the clip's own length (to a few milliseconds), so
/// fitting would change nothing.
pub fn same_length(target: f64, current: f64) -> bool {
    (target - current).abs() < 0.05
}

fn num(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64)
}

/// The edit that replaces `clip` with the plan's segments.
pub fn fit_ops(
    doc: &Value,
    track_id: &str,
    clip: &Value,
    media: &Value,
    plan: &FitPlan,
    new_id: &mut dyn FnMut() -> String,
) -> Value {
    let sess = num(doc, "sample_rate").unwrap_or(f64::NAN);
    let src = num(media, "sample_rate")
        .filter(|r| *r != 0.0)
        .unwrap_or(sess);
    let stretch = if (plan.stretch - 1.0).abs() < 0.0005 {
        1.0
    } else {
        plan.stretch
    };
    let xf = js_round(FIT_XFADE * sess * stretch);
    let many = plan.segments.len() > 1;
    let group = if many {
        json!(new_id())
    } else {
        clip.get("group").cloned().unwrap_or(Value::Null)
    };
    let mut start = num(clip, "start").unwrap_or(f64::NAN);
    let media_len = num(media, "length")
        .filter(|l| *l != 0.0)
        .unwrap_or(9_007_199_254_740_991.0);
    let mut clips: Vec<Value> = Vec::new();
    for (k, seg) in plan.segments.iter().enumerate() {
        let source_in = js_round(seg.0 * src).max(0.0);
        let source_out = media_len.min(js_round(seg.1 * src));
        let length = js_round((source_out - source_in) * sess / src * stretch);
        let first = k == 0;
        let last = k == plan.segments.len() - 1;
        let mut c = clip.as_object().cloned().unwrap_or_default();
        let id = if first && !many {
            clip.get("id").cloned().unwrap_or(Value::Null)
        } else {
            json!(new_id())
        };
        c.insert("id".into(), id);
        c.insert("start".into(), json!(start));
        c.insert("source_in".into(), json!(source_in));
        c.insert("source_out".into(), json!(source_out));
        let xfade = json!({ "length": xf, "shape": "cosine", "curve": 0 });
        if first {
            // Kept as the clip had it (absent stays absent).
        } else {
            c.insert("fade_in".into(), xfade.clone());
        }
        if !last {
            c.insert("fade_out".into(), xfade);
        }
        c.insert("looped".into(), Value::Null);
        c.insert(
            "stretch".into(),
            if stretch == 1.0 {
                Value::Null
            } else {
                json!({ "ratio": stretch, "keep_pitch": true })
            },
        );
        c.insert("group".into(), group.clone());
        // Envelopes were drawn for the old length; the new clips start clean.
        c.insert("envelopes".into(), json!([]));
        clips.push(json!([track_id, Value::Object(c)]));
        start += length - xf;
    }
    json!({ "op": "batch", "ops": [{ "op": "remove_clips", "ids": [clip.get("id").cloned().unwrap_or(Value::Null)] }, { "op": "restore_clips", "clips": clips }] })
}

/// One line for the dialog: how the plan sounds.
pub fn describe_plan(p: &FitPlan) -> String {
    let joins = match p.joins {
        0 => "No joins".to_string(),
        1 => "1 join".to_string(),
        n => format!("{n} joins"),
    };
    let matched = format!(
        "{}% match",
        js_number_to_string(js_round(p.quality * 100.0))
    );
    let st = if (p.stretch - 1.0).abs() < 0.0005 {
        "no stretch".to_string()
    } else {
        format!("stretched {}%", js_to_fixed1((p.stretch - 1.0) * 100.0))
    };
    format!(
        "{joins}, {matched}, {} BPM, {st}",
        js_number_to_string(js_round(p.tempo))
    )
}

fn clock(s: f64) -> String {
    let m = (s / 60.0).floor();
    let rest = js_to_fixed(s - m * 60.0, 1);
    if m > 0.0 {
        format!("{}:{rest:0>4}", js_number_to_string(m))
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
    // Each side runs half a crossfade past the beat it joins on.
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
