// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Time stretch edits and the Auto Mic Mix result, in Rust. Mirrors
//! `src/panels/editor/stretch.ts` (fit a clip to a length, remove a stretch,
//! percent labels, reading a typed time) and `src/app/multitrack.ts` (mic
//! tracks, the talk-time lines, the volume lanes an analysis writes). Tested
//! against `shared-fixtures/stretch-mix.json`.

use crate::clip_edit::stretch_of;
use crate::clip_math::{clip_length, ClipSpan, STRETCH_MAX, STRETCH_MIN};
use crate::js_compat::{js_number_to_string, js_round};
use serde_json::{json, Map, Value};

fn num(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(0.0)
}

fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k)
        .and_then(Value::as_array)
        .map_or(&[], |a| a.as_slice())
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0 && !x.is_nan()),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

/// Length the clip would play at with no stretch, in session samples.
pub fn natural_length(clip: &Value, media: Option<&Value>, rate: f64) -> f64 {
    let span = ClipSpan {
        start: num(clip, "start"),
        source_in: num(clip, "source_in"),
        source_out: num(clip, "source_out"),
        looped: clip
            .get("looped")
            .filter(|l| !l.is_null())
            .map(|l| num(l, "length")),
        stretch: None,
    };
    clip_length(
        &span,
        media
            .and_then(|m| m.get("sample_rate"))
            .and_then(Value::as_f64),
        rate,
    )
}

/// Envelope points for the volume and pan lanes, moved by `by`.
fn scale_envelopes(clip: &Value, by: f64) -> Map<String, Value> {
    let mut out = Map::new();
    for e in arr(clip, "envelopes") {
        if !truthy(e.get("enabled")) {
            continue;
        }
        let pts: Vec<Value> = arr(e, "points")
            .iter()
            .map(|p| {
                let mut p = p.clone();
                p["position"] = json!(js_round(num(&p, "position") * by));
                p
            })
            .collect();
        match e["target"]["type"].as_str() {
            Some("volume") => {
                out.insert("volume".into(), json!(pts));
            }
            Some("pan") => {
                out.insert("pan".into(), json!(pts));
            }
            _ => {}
        }
    }
    out
}

/// The result of a stretch edit.
#[derive(Debug, Clone, PartialEq)]
pub struct StretchResult {
    pub change: Value,
    pub ratio: f64,
    /// The ratio had to be limited to what a clip can take.
    pub limited: bool,
}

/// The change that makes `clip` play for `length` session samples. None when
/// the clip loops or nothing changes.
pub fn stretch_to(
    clip: &Value,
    media: Option<&Value>,
    rate: f64,
    length: f64,
    keep_pitch: bool,
) -> Option<StretchResult> {
    if clip.get("looped").is_some_and(|l| !l.is_null()) || length <= 0.0 {
        return None;
    }
    let natural = natural_length(clip, media, rate);
    if natural <= 0.0 {
        return None;
    }
    let want = length / natural;
    let ratio = STRETCH_MAX.min(STRETCH_MIN.max(want));
    let before = stretch_of(clip);
    let new_len = js_round(natural * ratio);
    let current_keep = clip
        .get("stretch")
        .and_then(|s| s.get("keep_pitch"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if (ratio - before).abs() < 1e-6 && current_keep == keep_pitch {
        return None;
    }
    let by = ratio / before;
    let mut kept = clip
        .get("stretch")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    kept.insert("ratio".into(), json!(ratio));
    kept.insert("keep_pitch".into(), json!(keep_pitch));
    let mut change = if (ratio - 1.0).abs() < 1e-6 && !(keep_pitch && truthy(kept.get("pitch"))) {
        json!({ "id": clip["id"], "unstretch": true })
    } else {
        json!({ "id": clip["id"], "stretch": kept })
    };
    for (k, v) in scale_envelopes(clip, by) {
        change[k] = v;
    }
    let fade_len = |k: &str| clip.get(k).map_or(0.0, |f| num(f, "length"));
    let fi = js_round(fade_len("fade_in") * by).min(new_len);
    let fo = js_round(fade_len("fade_out") * by).min(new_len - fi);
    if fi != fade_len("fade_in") {
        let mut f = clip["fade_in"].clone();
        f["length"] = json!(fi);
        change["fade_in"] = f;
    }
    if fo != fade_len("fade_out") {
        let mut f = clip["fade_out"].clone();
        f["length"] = json!(fo);
        change["fade_out"] = f;
    }
    Some(StretchResult {
        change,
        ratio,
        limited: (ratio - want).abs() > 1e-6,
    })
}

/// Back to the natural length and pitch. None when the clip is not stretched.
pub fn unstretch(clip: &Value, media: Option<&Value>, rate: f64) -> Option<Value> {
    let stretch = clip.get("stretch").filter(|s| !s.is_null())?;
    let keep = stretch
        .get("keep_pitch")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let mut change = stretch_to(clip, media, rate, natural_length(clip, media, rate), keep)
        .map_or_else(|| json!({ "id": clip["id"] }), |r| r.change);
    if let Some(o) = change.as_object_mut() {
        o.remove("stretch");
        o.insert("unstretch".into(), json!(true));
    }
    Some(change)
}

/// "112.5%" style label for a ratio.
pub fn percent(ratio: f64) -> String {
    format!("{}%", js_number_to_string(js_round(ratio * 1000.0) / 10.0))
}

/// Parses "30", "30.5", "1:05", "1:05.250" or "0:01:05" into seconds. None
/// when it is not a time.
pub fn parse_seconds(text: &str) -> Option<f64> {
    let t = text.trim();
    let (whole, frac) = match t.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (t, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let parts: Vec<&str> = whole.split(':').collect();
    let shape_ok = parts.len() <= 3
        && parts
            .iter()
            .enumerate()
            .all(|(i, p)| digits(p) && (i == 0 || p.len() <= 2))
        && frac.is_none_or(digits);
    if !shape_ok {
        return None;
    }
    let mut secs = 0.0;
    for (i, p) in parts.iter().enumerate() {
        let text = if i == parts.len() - 1 && frac.is_some() {
            format!("{p}.{}", frac.unwrap_or(""))
        } else {
            (*p).to_string()
        };
        secs = secs * 60.0 + text.parse::<f64>().ok()?;
    }
    (secs.is_finite() && secs > 0.0).then_some(secs)
}

/// What the dialog offers for how far a quiet mic is turned down: (id, label, dB).
pub const REDUCE_CHOICES: [(&str, &str, f64); 3] = [
    ("gentle", "Gentle (8 dB)", 8.0),
    ("standard", "Standard (12 dB)", 12.0),
    ("firm", "Firm (18 dB)", 18.0),
];

/// Audio tracks that have something on them: the candidates for a mic.
pub fn mic_tracks(doc: &Value) -> Vec<Value> {
    arr(doc, "tracks")
        .iter()
        .filter(|t| t["kind"] != "midi" && !arr(t, "clips").is_empty())
        .cloned()
        .collect()
}

/// Minutes and seconds, or hours when long.
pub fn format_duration(seconds: f64) -> String {
    let s = 0.0_f64.max(js_round(seconds));
    let h = (s / 3600.0).floor();
    let m = ((s % 3600.0) / 60.0).floor();
    let r = s % 60.0;
    let two = |n: f64| format!("{:02}", n as i64);
    if h > 0.0 {
        format!("{}:{}:{}", h as i64, two(m), two(r))
    } else {
        format!("{}:{}", m as i64, two(r))
    }
}

/// One line per voice for the result, most talkative first.
pub fn describe_talk(analysis: &Value) -> Vec<String> {
    let mut tracks: Vec<&Value> = arr(analysis, "tracks").iter().collect();
    tracks.sort_by(|a, b| {
        num(b, "talkSeconds")
            .partial_cmp(&num(a, "talkSeconds"))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    tracks
        .iter()
        .map(|t| {
            format!(
                "{}: {} ({}%)",
                t["name"].as_str().unwrap_or(""),
                format_duration(num(t, "talkSeconds")),
                js_number_to_string(js_round(num(t, "share") * 100.0))
            )
        })
        .collect()
}

fn target_key(t: &Value) -> String {
    match t["type"].as_str().unwrap_or("") {
        "effect_param" => format!(
            "fx:{}:{}",
            t["slot_id"].as_str().unwrap_or(""),
            t["param"].as_str().unwrap_or("")
        ),
        "space" => format!("space:{}", t["param"].as_str().unwrap_or("")),
        other => other.to_string(),
    }
}

/// The volume lane for a track: points from the analysis, enabled,
/// replacing any volume lane there. `new_id` makes the id of a new lane.
fn with_volume(track: &Value, points: &Value, new_id: &mut dyn FnMut() -> String) -> Value {
    let envs = arr(track, "envelopes");
    let volume = target_key(&json!({ "type": "volume" }));
    let existing = envs
        .iter()
        .find(|e| target_key(&e["target"]) == volume)
        .cloned();
    let mut lane = existing.unwrap_or_else(
        || json!({ "id": new_id(), "target": { "type": "volume" }, "enabled": true, "points": [] }),
    );
    lane["enabled"] = json!(true);
    lane["points"] = points.clone();
    let id = lane["id"].as_str().unwrap_or("").to_string();
    let id = if id.is_empty() {
        let n = new_id();
        lane["id"] = json!(n);
        n
    } else {
        id
    };
    let mut out: Vec<Value> = envs.to_vec();
    match out.iter().position(|e| e["id"] == json!(id)) {
        Some(i) => out[i] = lane,
        None => out.push(lane),
    }
    json!(out)
}

/// Writes each analysed track's gain curve to its volume lane, leaving other
/// lanes alone. One edit, so a single undo puts every fader lane back.
pub fn mix_ops(doc: &Value, analysis: &Value, new_id: &mut dyn FnMut() -> String) -> Option<Value> {
    let mut ops = Vec::new();
    for m in arr(analysis, "tracks") {
        let Some(track) = arr(doc, "tracks").iter().find(|t| t["id"] == m["track"]) else {
            continue;
        };
        if arr(m, "points").is_empty() {
            continue;
        }
        let envelopes = with_volume(track, &m["points"], new_id);
        ops.push(json!({ "op": "set_envelopes", "owner": track["id"], "envelopes": envelopes }));
    }
    match ops.len() {
        0 => None,
        1 => ops.pop(),
        _ => Some(json!({ "op": "batch", "ops": ops })),
    }
}

/// One line for the notice after the mix is written.
pub fn describe_mix(analysis: &Value) -> String {
    format!(
        "Wrote volume lanes for {} tracks. Quiet mics drop while someone else talks; undo takes it all back. Adjust the lanes by hand where a word sounds off.",
        arr(analysis, "tracks").len()
    )
}
