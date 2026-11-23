// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Small rules for opening files and reading the screen, in Rust: what File >
//! Open does with each picked file, the one-line summaries of an import, the
//! display name of a session, where a dropped file lands, wheel zoom anchors
//! and ratios, theme color overrides and meter position. Mirrors parts of
//! `src/app/session-files.ts`, `src/app/filedrop.ts` and
//! `src/stores/settings.ts`, tested against `shared-fixtures/open-rules.json`.

use serde_json::{json, Map, Value};

const AUDIO_EXTENSIONS: [&str; 19] = [
    "wav", "wave", "aif", "aiff", "aifc", "mp3", "flac", "m4a", "mp4", "aac", "ogg", "oga", "opus",
    "mkv", "mka", "webm", "w64", "rf64", "bwf",
];
/// Files that may carry a picture. An .mp4 with no picture is still opened as audio.
const VIDEO_EXTENSIONS: [&str; 6] = ["mp4", "m4v", "mov", "wmv", "avi", "mkv"];

/// The text after the last dot, lowercased (the whole name when there is no dot).
fn ext_of(p: &str) -> String {
    let i = p.rfind('.').map_or(0, |i| i + 1);
    p[i..].to_lowercase()
}

/// Whether the importer reads this file as audio.
pub fn is_audio_file(p: &str) -> bool {
    AUDIO_EXTENSIONS.contains(&ext_of(p).as_str())
}

/// Whether the file may carry a picture.
pub fn is_video_file(p: &str) -> bool {
    VIDEO_EXTENSIONS.contains(&ext_of(p).as_str())
}

/// A session file the app opens: our own or a .sesx.
pub fn is_session_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".altaudio") || lower.ends_with(".sesx")
}

/// Sorts files picked in File > Open by what happens to them.
pub fn plan_open(paths: &[String]) -> Value {
    let (mut session, mut sesx): (Option<&String>, Option<&String>) = (None, None);
    let (mut audio, mut video, mut skipped): (Vec<&String>, Vec<&String>, Vec<&String>) =
        (vec![], vec![], vec![]);
    for p in paths {
        let e = ext_of(p);
        if e == "altaudio" && session.is_none() {
            session = Some(p);
        } else if e == "sesx" && sesx.is_none() {
            sesx = Some(p);
        } else if VIDEO_EXTENSIONS.contains(&e.as_str()) {
            video.push(p);
        } else if AUDIO_EXTENSIONS.contains(&e.as_str()) {
            audio.push(p);
        } else {
            skipped.push(p);
        }
    }
    if session.is_some() && sesx.is_some() {
        skipped.extend(sesx.take());
    }
    json!({ "session": session, "sesx": sesx, "audio": audio, "video": video, "skipped": skipped })
}

/// Display name: the file name without extension, or the session name.
pub fn display_name(path: Option<&str>, doc_name: Option<&str>) -> String {
    if let Some(p) = path.filter(|p| !p.is_empty()) {
        let base = p.rsplit(['/', '\\']).next().unwrap_or(p);
        let lower = base.to_lowercase();
        return if lower.ends_with(".altaudio") {
            base[..base.len() - ".altaudio".len()].to_string()
        } else {
            base.to_string()
        };
    }
    doc_name.unwrap_or("Untitled Session").to_string()
}

fn count(n: f64, one: &str, many: &str) -> String {
    let n = crate::js_compat::js_number_to_string(n);
    format!("{n} {}", if n == "1" { one } else { many })
}

fn n(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn names(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn missing_files(r: &Value, parts: &mut Vec<String>) {
    let k = names(r, "missingFiles").len();
    if k > 0 {
        parts.push(format!(
            "{k} media {} not found.",
            if k == 1 { "file was" } else { "files were" }
        ));
    }
}

/// One line describing what a .sesx import brought across.
pub fn sesx_summary(r: &Value) -> String {
    let buses = if n(r, "buses") != 0.0 {
        format!(", {}", count(n(r, "buses"), "bus", "buses"))
    } else {
        String::new()
    };
    let mut parts = vec![format!(
        "Imported {}{buses} and {}.",
        count(n(r, "tracks"), "track", "tracks"),
        count(n(r, "clips"), "clip", "clips")
    )];
    if n(r, "markers") != 0.0 {
        parts.push(format!(
            "{} came across.",
            count(n(r, "markers"), "marker", "markers")
        ));
    }
    if n(r, "plugins") != 0.0 {
        let p = n(r, "plugins");
        parts.push(format!(
            "{} came across with {} settings.",
            count(p, "plug-in", "plug-ins"),
            if p == 1.0 { "its" } else { "their" }
        ));
    }
    let missing = names(r, "missingPlugins");
    if !missing.is_empty() {
        parts.push(format!(
            "Not found on this computer (install or scan them in the Plug-in Manager): {}.",
            missing.join(", ")
        ));
    }
    if n(r, "converted") != 0.0 {
        parts.push(format!(
            "{} across as ours (Phase Curve, Parametric EQ, Vox Strip or Limiter).",
            count(
                n(r, "converted"),
                "built-in effect came",
                "built-in effects came"
            )
        ));
    }
    let approx = names(r, "approximate");
    if !approx.is_empty() {
        parts.push(format!(
            "Check by ear, some settings are estimates: {}.",
            approx.join(", ")
        ));
    }
    if n(r, "stretched") != 0.0 {
        parts.push(format!(
            "{} stretch and pitch.",
            count(n(r, "stretched"), "clip keeps its", "clips keep their")
        ));
    }
    if n(r, "skippedEffects") > 0.0 {
        let list = names(r, "skippedNames");
        let tail = if list.is_empty() {
            String::new()
        } else {
            format!(" ({})", list.join(", "))
        };
        parts.push(format!(
            "{} left out{tail}; rebuild them in Voice Chain.",
            count(
                n(r, "skippedEffects"),
                "built-in effect was",
                "built-in effects were"
            )
        ));
    }
    missing_files(r, &mut parts);
    parts.join(" ")
}

fn project_summary(r: &Value, stretch_note: bool) -> String {
    let mut parts = vec![format!(
        "Imported {} and {}.",
        count(n(r, "tracks"), "track", "tracks"),
        count(n(r, "clips"), "clip", "clips")
    )];
    if n(r, "markers") != 0.0 {
        parts.push(format!(
            "{} came across.",
            count(n(r, "markers"), "marker", "markers")
        ));
    }
    if stretch_note && n(r, "stretched") != 0.0 {
        parts.push(format!(
            "{} speed change as a stretch.",
            count(n(r, "stretched"), "clip keeps its", "clips keep their")
        ));
    }
    let left = names(r, "leftOut");
    if !left.is_empty() {
        parts.push(format!("Left out: {}.", left.join(", ")));
    }
    missing_files(r, &mut parts);
    parts.join(" ")
}

/// One line describing what a .rpp import brought across.
pub fn rpp_summary(r: &Value) -> String {
    project_summary(r, true)
}

/// One line describing what an .otio import brought across.
pub fn otio_summary(r: &Value) -> String {
    project_summary(r, false)
}

/// Pixels per CSS pixel for drop positions. The core reports drops in device
/// pixels; measuring the webview's real size against its CSS size gives the
/// true ratio. Falls back to the device pixel ratio when the size is unknown.
pub fn drop_scale(physical_width: Option<f64>, css_width: f64, dpr: f64) -> f64 {
    if let Some(w) = physical_width.filter(|w| *w != 0.0) {
        if css_width > 0.0 {
            let r = w / css_width;
            if r > 0.25 && r < 8.0 {
                return r;
            }
        }
    }
    if dpr > 0.0 {
        dpr
    } else {
        1.0
    }
}

/// Where a wheel zoom centers, in pixels across a view `width` wide: the
/// pointer, the playhead (when in view), or between the pointer and the
/// middle of the view. Over a ruler, `exact` keeps it on the pointer.
pub fn wheel_anchor_x(anchor: &str, mouse: f64, width: f64, playhead_x: f64, exact: bool) -> f64 {
    if exact || anchor == "mouse" {
        return mouse;
    }
    if anchor == "playhead" {
        return 0.0_f64.max(width.min(playhead_x));
    }
    mouse + (width / 2.0 - mouse) * 0.4
}

/// The Ctrl+wheel zoom ratios: (zoom in, zoom out).
pub fn zoom_ratios(step: f64) -> (f64, f64) {
    let out = 1.0 + step / 100.0;
    (1.0 / out, out)
}

/// The theme color overrides as CSS variables; empty ones fall back to the theme.
pub fn color_vars(wave: &str, playhead: &str, accent: &str, lane_selected: &str) -> Value {
    let mut v = Map::new();
    for (name, value) in [
        ("--wave", wave),
        ("--playhead", playhead),
        ("--accent", accent),
        ("--lane-selected", lane_selected),
    ] {
        if !value.is_empty() {
            v.insert(name.to_string(), json!(value));
        }
    }
    Value::Object(v)
}

/// Where a level in dBFS sits on a meter from `floor_db` to 0 dB, as a percentage.
pub fn meter_percent(db: f64, floor_db: f64) -> f64 {
    0.0_f64.max(100.0_f64.min(((db - floor_db) / -floor_db) * 100.0))
}
