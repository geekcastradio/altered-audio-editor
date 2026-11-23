// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! QuickPass scanning rules in Rust: the scan types, the settings and their
//! clean-up, and how measured levels become findings (silences, short noises,
//! the parts with sound, clipping and click spots) plus the starting levels
//! suggested from a clip. Mirrors the scanning half of
//! `src/panels/quickpass/quickpass.ts`, tested against
//! `shared-fixtures/quickpass-scan.json`. The edit-building half stays in
//! TypeScript until the clip-edit layer is ported. The scan types, defaults and
//! built-in presets are data in `data/quickpass.json`, kept equal to the
//! TypeScript lists by a test.
//!
//! The track tie-break in the findings sort uses `natural_cmp`, which follows
//! the browser order only for letters, digits and spaces; it only matters
//! when two findings start at the same sample.

use crate::js_compat::{js_number_to_string, js_round};
use crate::library_view::natural_cmp;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

pub fn data() -> &'static Value {
    static D: OnceLock<Value> = OnceLock::new();
    D.get_or_init(|| {
        serde_json::from_str(include_str!("../data/quickpass.json")).expect("quickpass.json")
    })
}

/// The actions a scan type offers, first one is the default.
pub fn scan_actions(scan: &str) -> Vec<String> {
    let types = data()["scanTypes"].as_array().cloned().unwrap_or_default();
    let t = types
        .iter()
        .find(|t| t["id"] == scan)
        .or(types.first())
        .cloned()
        .unwrap_or(Value::Null);
    t["actions"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub scan: String,
    pub max_burst_sec: f64,
    pub guard_sec: f64,
    pub clip_db: f64,
    pub clip_longest_ms: f64,
    pub click_sensitivity: f64,
    pub click_width_ms: f64,
    pub quiet_db: f64,
    pub min_quiet_sec: f64,
    pub content_db: f64,
    pub min_content_sec: f64,
    pub action: String,
    pub keep_ms: f64,
    pub ripple: String,
    pub duck_db: f64,
}

impl Settings {
    pub fn from_json(v: &Value) -> Settings {
        let f = |k: &str| v[k].as_f64().unwrap_or(0.0);
        let s = |k: &str| v[k].as_str().unwrap_or("").to_string();
        Settings {
            scan: s("scan"),
            max_burst_sec: f("maxBurstSec"),
            guard_sec: f("guardSec"),
            clip_db: f("clipDb"),
            clip_longest_ms: f("clipLongestMs"),
            click_sensitivity: f("clickSensitivity"),
            click_width_ms: f("clickWidthMs"),
            quiet_db: f("quietDb"),
            min_quiet_sec: f("minQuietSec"),
            content_db: f("contentDb"),
            min_content_sec: f("minContentSec"),
            action: s("action"),
            keep_ms: f("keepMs"),
            ripple: s("ripple"),
            duck_db: f("duckDb"),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "scan": self.scan, "maxBurstSec": self.max_burst_sec, "guardSec": self.guard_sec,
            "clipDb": self.clip_db, "clipLongestMs": self.clip_longest_ms,
            "clickSensitivity": self.click_sensitivity, "clickWidthMs": self.click_width_ms,
            "quietDb": self.quiet_db, "minQuietSec": self.min_quiet_sec, "contentDb": self.content_db,
            "minContentSec": self.min_content_sec, "action": self.action, "keepMs": self.keep_ms,
            "ripple": self.ripple, "duckDb": self.duck_db,
        })
    }
}

pub fn defaults() -> Settings {
    Settings::from_json(&data()["defaults"])
}

/// A number held to a range; anything that is not a finite number takes the fallback.
fn num(v: &Value, lo: f64, hi: f64, fallback: f64) -> f64 {
    match v.as_f64() {
        Some(x) if x.is_finite() => x.clamp(lo, hi),
        _ => fallback,
    }
}

/// Saved settings with anything missing or out of range put right.
pub fn sanitize_settings(raw: &Value) -> Settings {
    let d = defaults();
    let empty = Value::Null;
    let r = if raw.is_object() { raw } else { &empty };
    let quiet_db = num(&r["quietDb"], -100.0, -1.0, d.quiet_db);
    let scan = match r["scan"].as_str() {
        Some(s @ ("noise" | "clipping" | "clicks" | "audio")) => s.to_string(),
        _ => "silence".to_string(),
    };
    let actions = scan_actions(&scan);
    let action = match r["action"].as_str() {
        Some(a) if actions.iter().any(|x| x == a) => a.to_string(),
        _ => actions.first().cloned().unwrap_or_default(),
    };
    Settings {
        scan,
        max_burst_sec: num(&r["maxBurstSec"], 0.02, 2.0, d.max_burst_sec),
        guard_sec: num(&r["guardSec"], 0.05, 5.0, d.guard_sec),
        clip_db: num(&r["clipDb"], -12.0, 0.0, d.clip_db),
        clip_longest_ms: num(&r["clipLongestMs"], 0.1, 10.0, d.clip_longest_ms),
        click_sensitivity: num(&r["clickSensitivity"], 0.0, 100.0, d.click_sensitivity),
        click_width_ms: num(&r["clickWidthMs"], 0.1, 3.0, d.click_width_ms),
        quiet_db,
        min_quiet_sec: num(&r["minQuietSec"], 0.005, 60.0, d.min_quiet_sec),
        content_db: quiet_db.max(num(&r["contentDb"], -100.0, 0.0, d.content_db)),
        min_content_sec: num(&r["minContentSec"], 0.01, 10.0, d.min_content_sec),
        action,
        keep_ms: num(&r["keepMs"], 0.0, 5000.0, d.keep_ms),
        ripple: match r["ripple"].as_str() {
            Some("all") => "all".into(),
            Some("track") => "track".into(),
            _ => "none".into(),
        },
        duck_db: num(&r["duckDb"], -60.0, -1.0, d.duck_db),
    }
}

/// One clip's measured levels.
#[derive(Clone, Debug)]
pub struct ClipLevels {
    pub clip_id: String,
    /// Timeline position of the first window, in session samples.
    pub start: f64,
    /// Session samples in each window.
    pub window: f64,
    pub db: Vec<f64>,
    pub peak: Vec<f64>,
}

/// One clip's click or clipping spots, as sample spans.
#[derive(Clone, Debug)]
pub struct ClipSpots {
    pub clip_id: String,
    pub spans: Vec<(f64, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub id: String,
    pub kind: String,
    pub track_id: String,
    pub clip_id: String,
    pub clip_name: String,
    pub start: f64,
    pub end: f64,
}

impl Finding {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id, "kind": self.kind, "trackId": self.track_id, "clipId": self.clip_id,
            "clipName": self.clip_name, "start": self.start, "end": self.end,
        })
    }
}

/// Clip id to the track it is on and its name.
pub type ClipInfo = HashMap<String, (String, String)>;

type Runs = Vec<(usize, usize)>;

/// `Math.max(1, Math.ceil(sec / windowSec))`, kept as a number so huge values compare as in JS.
fn windows(sec: f64, window_sec: f64) -> f64 {
    (sec / window_sec).ceil().max(1.0)
}

/// Runs of windows [from, to) that are silence under the settings.
pub fn silent_runs(db: &[f64], window_sec: f64, s: &Settings) -> Runs {
    let min_quiet = windows(s.min_quiet_sec, window_sec);
    let min_content = windows(s.min_content_sec, window_sec);
    // Quiet runs, then join two that only a short burst of speech separates.
    let mut quiet: Runs = vec![];
    let mut from: Option<usize> = None;
    for (i, &v) in db.iter().enumerate() {
        if v < s.quiet_db {
            from.get_or_insert(i);
        } else if let Some(f) = from.take() {
            quiet.push((f, i));
        }
    }
    if let Some(f) = from {
        quiet.push((f, db.len()));
    }
    let mut joined: Runs = vec![];
    for run in quiet {
        if let Some(last) = joined.last_mut() {
            let between = &db[last.1.min(run.0)..run.0];
            let loud = between.iter().filter(|&&v| v >= s.content_db).count() as f64;
            if (between.len() as f64) < min_content || loud < min_content {
                last.1 = run.1;
                continue;
            }
        }
        joined.push(run);
    }
    joined
        .into_iter()
        .filter(|&(a, b)| (b - a) as f64 >= min_quiet)
        .collect()
}

fn sorted(mut out: Vec<Finding>) -> Vec<Finding> {
    out.sort_by(|x, y| {
        x.start
            .partial_cmp(&y.start)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| natural_cmp(&x.track_id, &y.track_id))
    });
    out
}

fn finding(
    l: &ClipLevels,
    info: &(String, String),
    kind: &str,
    id_prefix: String,
    run: (usize, usize),
) -> Finding {
    let clip_end = l.start + l.db.len() as f64 * l.window;
    let start = l.start + run.0 as f64 * l.window;
    let end = clip_end.min(l.start + run.1 as f64 * l.window);
    Finding {
        id: format!("{id_prefix}{}", js_number_to_string(start)),
        kind: kind.to_string(),
        track_id: info.0.clone(),
        clip_id: l.clip_id.clone(),
        clip_name: info.1.clone(),
        start,
        end,
    }
}

/// Findings for the measured clips, in timeline order per track.
pub fn find_silences(
    levels: &[ClipLevels],
    tracks: &ClipInfo,
    s: &Settings,
    rate: f64,
) -> Vec<Finding> {
    let mut out = vec![];
    for l in levels {
        let Some(info) = tracks.get(&l.clip_id) else {
            continue;
        };
        for run in silent_runs(&l.db, l.window / rate, s) {
            out.push(finding(l, info, "silence", format!("{}:", l.clip_id), run));
        }
    }
    sorted(out)
}

/// Short noises: sound at or above the quiet level that lasts no longer than
/// `max_burst_sec`, with at least `guard_sec` of quiet either side. Each run is
/// padded by one window so its edges go too.
pub fn noise_runs(db: &[f64], window_sec: f64, s: &Settings) -> Runs {
    let max_len = (s.max_burst_sec / window_sec).floor().max(1.0);
    let guard = windows(s.guard_sec, window_sec);
    let mut loud: Runs = vec![];
    let mut from: Option<usize> = None;
    for (i, &v) in db.iter().enumerate() {
        if v >= s.quiet_db {
            from.get_or_insert(i);
        } else if let Some(f) = from.take() {
            loud.push((f, i));
        }
    }
    if let Some(f) = from {
        loud.push((f, db.len()));
    }
    let mut out = vec![];
    for (i, &(a, b)) in loud.iter().enumerate() {
        if (b - a) as f64 > max_len {
            continue;
        }
        let before = if i == 0 { a } else { a - loud[i - 1].1 };
        let after = if i == loud.len() - 1 {
            db.len() - b
        } else {
            loud[i + 1].0 - b
        };
        if (i > 0 && (before as f64) < guard) || (i < loud.len() - 1 && (after as f64) < guard) {
            continue;
        }
        out.push((a.saturating_sub(1), db.len().min(b + 1)));
    }
    out
}

/// The parts with sound: everything between silences, in windows [from, to).
pub fn sound_runs(db: &[f64], window_sec: f64, s: &Settings) -> Runs {
    let mut out: Runs = vec![];
    let mut at = 0;
    for (a, b) in silent_runs(db, window_sec, s) {
        if a > at {
            out.push((at, a));
        }
        at = b;
    }
    if at < db.len() {
        out.push((at, db.len()));
    }
    // A lone window of noise is not a part with sound.
    let min_content = windows(s.min_content_sec, window_sec);
    out.into_iter()
        .filter(|&(a, b)| (b - a) as f64 >= min_content)
        .collect()
}

/// What the core looks for, from the settings.
pub fn spots_request(s: &Settings) -> Value {
    if s.scan == "clicks" {
        json!({ "kind": "clicks", "sensitivity": s.click_sensitivity, "widthMs": s.click_width_ms })
    } else {
        json!({ "kind": "clipping", "levelDb": s.clip_db, "longestMs": s.clip_longest_ms })
    }
}

/// Findings from the core's click or clipping spots, in timeline order.
pub fn spot_findings(spots: &[ClipSpots], tracks: &ClipInfo, kind: &str) -> Vec<Finding> {
    let mut out = vec![];
    for c in spots {
        let Some(info) = tracks.get(&c.clip_id) else {
            continue;
        };
        for &(a, b) in &c.spans {
            out.push(Finding {
                id: format!("{}:{}:{}", c.clip_id, kind, js_number_to_string(a)),
                kind: kind.to_string(),
                track_id: info.0.clone(),
                clip_id: c.clip_id.clone(),
                clip_name: info.1.clone(),
                start: a,
                end: b + 1.0,
            });
        }
    }
    sorted(out)
}

/// Clipping: windows whose peak reaches `clip_db`, joined when two or fewer windows apart.
pub fn clip_runs(peak: &[f64], s: &Settings) -> Runs {
    let mut out: Runs = vec![];
    for (i, &v) in peak.iter().enumerate() {
        if v < s.clip_db {
            continue;
        }
        match out.last_mut() {
            Some(last) if i - last.1 <= 2 => last.1 = i + 1,
            _ => out.push((i, i + 1)),
        }
    }
    out
}

/// Findings of the chosen scan type for the measured clips, in timeline order.
pub fn find_all(levels: &[ClipLevels], tracks: &ClipInfo, s: &Settings, rate: f64) -> Vec<Finding> {
    if s.scan == "silence" {
        return find_silences(levels, tracks, s, rate);
    }
    let mut out = vec![];
    for l in levels {
        let Some(info) = tracks.get(&l.clip_id) else {
            continue;
        };
        let runs = match s.scan.as_str() {
            "noise" => noise_runs(&l.db, l.window / rate, s),
            "audio" => sound_runs(&l.db, l.window / rate, s),
            _ => clip_runs(&l.peak, s),
        };
        for run in runs {
            out.push(finding(
                l,
                info,
                &s.scan,
                format!("{}:{}:", l.clip_id, s.scan),
                run,
            ));
        }
    }
    sorted(out)
}

/// Starting levels from what the clip sounds like: the quiet level sits a
/// little above its noise floor, the speech level halfway to its normal
/// speaking level. None when there is too little to judge.
pub fn suggest_levels(db: &[f64]) -> Option<(f64, f64)> {
    let mut v: Vec<f64> = db.iter().copied().filter(|&x| x > -120.0).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if v.len() < 20 {
        return None;
    }
    let at = |p: f64| v[((p * v.len() as f64).floor() as usize).min(v.len() - 1)];
    let floor = at(0.1);
    let speech = at(0.9);
    if speech - floor < 6.0 {
        return None;
    }
    let quiet = js_round(floor + (speech - floor) * 0.2);
    let content = quiet.max(js_round(floor + (speech - floor) * 0.4));
    Some((quiet, content))
}

/// What the Scan button says while a scan runs: the clip it has reached, and
/// the spots found so far when the scan counts them.
pub fn scan_label(progress: Option<(i64, i64, Option<i64>)>) -> String {
    match progress {
        Some((done, total, found)) if total > 0 => {
            let base = format!("Scanning clip {} of {}", done.min(total), total);
            match found {
                None => base,
                Some(n) => format!("{base}, {n} found"),
            }
        }
        _ => "Scanning".to_string(),
    }
}
