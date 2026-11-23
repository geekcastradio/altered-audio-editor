// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Small rules from the interface's stores, in Rust: library paths and tags,
//! file names, the level meter's fall and peak hold, whether a session is
//! blank, and reading the saved editor settings. Mirrors parts of
//! `src/stores/libraries.ts`, `media-pool.ts`, `transport.ts`, `session.ts`
//! and `editor.ts`, tested against `shared-fixtures/store-rules.json`.

use crate::js_compat::js_round;
use serde_json::{json, Map, Value};

fn sep_of(dir: &str) -> char {
    if dir.contains('\\') {
        '\\'
    } else {
        '/'
    }
}

/// Whether a path starts at a root: `/`, a UNC `\\`, or a drive letter.
pub fn is_absolute_path(p: &str) -> bool {
    let b = p.as_bytes();
    p.starts_with('/')
        || p.starts_with("\\\\")
        || (b.len() >= 3
            && b[0].is_ascii_alphabetic()
            && b[1] == b':'
            && (b[2] == b'\\' || b[2] == b'/'))
}

/// Joins a folder and a relative path with the folder's own separator.
pub fn join_path(dir: &str, rel: &str) -> String {
    let sep = sep_of(dir);
    let clean: String = rel
        .chars()
        .map(|c| if c == '/' || c == '\\' { sep } else { c })
        .collect();
    if dir.ends_with(sep) {
        format!("{dir}{clean}")
    } else {
        format!("{dir}{sep}{clean}")
    }
}

/// Absolute path of an item's audio file.
pub fn item_path(dir: &str, file: &str) -> String {
    if is_absolute_path(file) {
        file.to_string()
    } else {
        join_path(dir, file)
    }
}

/// Tags from a comma separated box: trimmed, without blanks, and without
/// repeats (ignoring case).
pub fn parse_tags(raw: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for part in raw.split(',') {
        let t = part.trim();
        let low = t.to_lowercase();
        if t.is_empty() || seen.contains(&low) {
            continue;
        }
        seen.push(low);
        out.push(t.to_string());
    }
    out
}

/// `tags` with `add` put on the end (unless already there) and `remove`
/// taken off, matching ignoring case.
pub fn retag(tags: &[String], add: &[String], remove: &[String]) -> Vec<String> {
    let gone: Vec<String> = remove.iter().map(|t| t.to_lowercase()).collect();
    let mut out: Vec<String> = tags
        .iter()
        .filter(|t| !gone.contains(&t.to_lowercase()))
        .cloned()
        .collect();
    let mut have: Vec<String> = out.iter().map(|t| t.to_lowercase()).collect();
    for t in add {
        let low = t.to_lowercase();
        if gone.contains(&low) || have.contains(&low) {
            continue;
        }
        have.push(low);
        out.push(t.clone());
    }
    out
}

/// Folder-safe version of a library name.
pub fn folder_name(name: &str) -> String {
    let swapped: String = name
        .chars()
        .map(|c| if "<>:\"/\\|?*".contains(c) { ' ' } else { c })
        .collect();
    let collapsed = swapped
        .split(char::is_whitespace)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let cleaned = collapsed.trim_end_matches(['.', ' ']);
    if cleaned.is_empty() {
        "Library".to_string()
    } else {
        cleaned.to_string()
    }
}

/// File name from a Windows or POSIX path.
pub fn file_name(path: &str) -> String {
    let last = path.rsplit(['/', '\\']).next().unwrap_or("");
    if last.is_empty() {
        path.to_string()
    } else {
        last.to_string()
    }
}

/// One meter's level, peak and over flag, 0..1 (above 1 means over).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeterLevel {
    pub level: f64,
    pub peak: f64,
    pub over: bool,
}

/// How the meter falls and holds for a speed: (fall per tick, share of a rise taken, hold ticks).
fn ballistics(speed: &str) -> (f64, f64, f64) {
    match speed {
        "medium" => (0.93, 0.6, 45.0),
        "slow" => (0.965, 0.3, 60.0),
        _ => (0.86, 1.0, 30.0),
    }
}

/// One meter reading folded into the last: slow fall, held peak. `hold`
/// counts ticks left. Returns the new meter and hold.
pub fn fold_meter(speed: &str, prev: MeterLevel, reading: f64, hold: f64) -> (MeterLevel, f64) {
    let (fall, rise, hold_ticks) = ballistics(speed);
    let risen = if reading > prev.level {
        prev.level + (reading - prev.level) * rise
    } else {
        reading
    };
    let level = risen.max(prev.level * fall);
    let mut peak = prev.peak;
    let mut hold = hold;
    if reading >= peak {
        peak = reading;
        hold = hold_ticks;
    } else if hold > 0.0 {
        hold -= 1.0;
    } else {
        peak = level.max(peak * fall);
    }
    (
        MeterLevel {
            level,
            peak,
            over: prev.over || reading > 1.0,
        },
        hold,
    )
}

/// Meter position for a 0..1 peak on a scale from `floor_db` to 0 dB.
pub fn meter_fraction(peak: f64, floor_db: f64) -> f64 {
    if peak <= 0.0 {
        return 0.0;
    }
    let db = 20.0 * peak.log10();
    0.0_f64.max(1.0_f64.min((db - floor_db) / -floor_db))
}

fn has_items(v: Option<&Value>) -> bool {
    v.and_then(Value::as_array).is_some_and(|a| !a.is_empty())
}

/// True for a session with nothing in it worth keeping: no clips (audio or
/// MIDI), markers, buses or effects.
pub fn is_blank(doc: Option<&Value>) -> bool {
    let Some(doc) = doc.filter(|d| !d.is_null()) else {
        return true;
    };
    let fx = |c: Option<&Value>| has_items(c.and_then(|c| c.get("slots")));
    let tracks = doc
        .get("tracks")
        .and_then(Value::as_array)
        .map_or(&[][..], |a| a.as_slice());
    tracks.iter().all(|t| {
        !has_items(t.get("clips")) && !has_items(t.get("midi_clips")) && !fx(t.get("effects"))
    }) && !has_items(doc.get("markers"))
        && !has_items(doc.get("buses"))
        && !fx(doc.get("master").and_then(|m| m.get("effects")))
}

fn flag(v: Option<&Value>, d: &Value) -> Value {
    v.filter(|v| v.is_boolean())
        .cloned()
        .unwrap_or_else(|| d.clone())
}

/// Reads saved editor settings, keeping `base` for anything missing or the
/// wrong type.
pub fn parse_editor_settings(raw: &Value, base: &Value) -> Value {
    let o = raw.as_object().cloned().unwrap_or_default();
    let snap_raw: Map<String, Value> = o
        .get("snap")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let b = |k: &str| &base[k];
    let mut out = Map::new();
    let mut snap = Map::new();
    for k in ["on", "clips", "playhead", "markers", "ruler"] {
        snap.insert(k.into(), flag(snap_raw.get(k), &base["snap"][k]));
    }
    out.insert("snap".into(), Value::Object(snap));
    for k in [
        "autoCrossfade",
        "snapZero",
        "showVolume",
        "showPan",
        "followPlayhead",
        "showSpectrum",
        "showHud",
        "showPitch",
        "stretchHandles",
    ] {
        out.insert(k.into(), flag(o.get(k), b(k)));
    }
    let brush = o
        .get("brushSize")
        .and_then(Value::as_f64)
        .filter(|x| (2.0..=80.0).contains(x));
    out.insert(
        "brushSize".into(),
        brush.map_or_else(|| b("brushSize").clone(), |x| json!(js_round(x))),
    );
    let pick = |k: &str, allowed: &[&str]| match o.get(k).and_then(Value::as_str) {
        Some(s) if allowed.contains(&s) => json!(s),
        _ => b(k).clone(),
    };
    out.insert(
        "playMode".into(),
        pick("playMode", &["loop", "skip", "normal"]),
    );
    out.insert(
        "timeFormat".into(),
        pick(
            "timeFormat",
            &["samples", "time", "timecode", "seconds", "bars"],
        ),
    );
    Value::Object(out)
}
