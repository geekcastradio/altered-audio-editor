// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Reads saved settings the way the interface does, in Rust: anything
//! missing, out of range or the wrong type falls back to its default, and
//! damaged controller bindings are dropped. Mirrors `sanitizeSettings` in
//! `src/stores/settings.ts` and `cleanBindings` in
//! `src/app/controller/map.ts`, tested against
//! `shared-fixtures/settings-clean.json`.

use crate::js_compat::js_round;
use serde_json::{json, Map, Value};

/// The settings a fresh install starts with.
pub fn default_settings() -> Value {
    json!({
        "zoomStep": 25, "wheelAnchor": "between", "wheelZooms": true, "fadeShape": "cosine",
        "dropMany": "tracks",
        "colors": {
            "wave": "", "playhead": "", "accent": "", "laneSelected": "", "spectrumLow": "",
            "spectrumHigh": "", "spectrumScheme": "", "phaseLow": "", "phaseHigh": ""
        },
        "surface": "standard", "theme": "dark", "output": null, "outputPair": [0, 1],
        "speakerOutputs": {}, "waveInput": null, "waveMonitor": false, "driver": "system",
        "bufferFrames": 0, "meterSpeed": "fast", "meterRangeDb": 60, "customLufs": -18, "customPeak": -1,
        "autosave": true, "autosaveMinutes": 2, "openChainOnAdd": true,
        "fillClickTexture": "natural", "realtimePlugins": false, "soloExclusive": false,
        "autoThinPercent": 0.3, "meterYellowDb": -12, "meterRedDb": -3, "backups": false,
        "backupCount": 5, "copyMedia": "always", "mediaFolder": "Audio", "collectOnSave": true,
        "leaveLibraryFiles": true, "countdownSec": 0, "markerName": "Marker", "timecodeFps": 30,
        "phaseWheel": { "size": 4096, "fps": 60, "a": "left", "b": "right" },
        "musicMode": false, "quickActionsBar": false, "favorites": [], "returnOnStop": true, "takeLanes": true, "playbackSpeed": false,
        "revealAfterExport": true, "scrubAudio": false, "ampModelAlways": false,
        "stopAtSelectionEnd": true, "updateCheck": false, "updateEvery": "week",
        "updateAutoInstall": false, "lastUpdateCheck": 0, "controllerPorts": [],
        "controllerBindings": [], "controllerFeedback": false, "welcomeRev": 0, "spaceSnap": true,
        "metronome": {
            "on": false, "muted": false, "when": "always", "levelDb": -6, "sound": "tick",
            "accent": true, "subdivide": "none"
        }
    })
}

/// Commands pinned to Favorites: text ids with no repeats, none of the Favorites or Open Recent
/// rows themselves, at most 24. Mirrors `sanitizeFavorites` in the interface.
fn favorites(v: Option<&Value>) -> Value {
    let mut out: Vec<Value> = Vec::new();
    if let Some(Value::Array(items)) = v {
        for it in items {
            let Some(id) = it.as_str() else { continue };
            let pinnable =
                !id.is_empty() && !id.starts_with("favorites.") && !id.starts_with("file.recent.");
            if pinnable && !out.iter().any(|x| x.as_str() == Some(id)) {
                out.push(Value::String(id.to_string()));
            }
            if out.len() >= 24 {
                break;
            }
        }
    }
    Value::Array(out)
}

fn num(v: Option<&Value>, lo: f64, hi: f64, def: f64) -> f64 {
    match v.and_then(Value::as_f64) {
        Some(x) if x.is_finite() => hi.min(lo.max(x)),
        _ => def,
    }
}

fn boolean(v: Option<&Value>, def: bool) -> bool {
    v.and_then(Value::as_bool).unwrap_or(def)
}

fn is_int_in(v: &Value, lo: f64, hi: f64) -> bool {
    v.as_f64()
        .is_some_and(|x| x.fract() == 0.0 && x >= lo && x <= hi)
}

fn int_in(v: Option<&Value>, lo: f64, hi: f64) -> Option<i64> {
    v.filter(|v| is_int_in(v, lo, hi))
        .and_then(Value::as_f64)
        .map(|x| x as i64)
}

fn pick<'a>(v: Option<&Value>, allowed: &[&'a str], def: &'a str) -> Value {
    json!(v
        .and_then(Value::as_str)
        .and_then(|s| allowed.iter().find(|a| **a == s))
        .copied()
        .unwrap_or(def))
}

fn color(v: Option<&Value>) -> String {
    match v.and_then(Value::as_str) {
        Some(s)
            if s.len() == 7
                && s.starts_with('#')
                && s[1..].chars().all(|c| c.is_ascii_hexdigit()) =>
        {
            s.to_lowercase()
        }
        _ => String::new(),
    }
}

fn obj(v: Option<&Value>) -> Map<String, Value> {
    v.and_then(Value::as_object).cloned().unwrap_or_default()
}

/// JS `string.slice(0, n)` for text without characters outside the basic plane.
fn slice(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn round_to(x: f64, scale: f64) -> f64 {
    js_round(x * scale) / scale
}

fn metronome(raw: Option<&Value>) -> Value {
    let m = obj(raw);
    let lvl = num(m.get("levelDb"), -60.0, 6.0, -6.0);
    json!({
        "on": boolean(m.get("on"), false),
        "muted": boolean(m.get("muted"), false),
        "when": if m.get("when").and_then(Value::as_str) == Some("recording") { "recording" } else { "always" },
        "levelDb": round_to(lvl, 10.0),
        "sound": pick(m.get("sound"), &["tick", "wood", "beep", "shaker"], "tick"),
        "accent": boolean(m.get("accent"), true),
        "subdivide": pick(m.get("subdivide"), &["none", "halves", "triplets", "quarters"], "none"),
    })
}

fn phase_prefs(raw: Option<&Value>) -> Value {
    let p = obj(raw);
    let one_of = |key: &str, list: &[f64], def: f64| {
        p.get(key)
            .and_then(Value::as_f64)
            .filter(|x| list.contains(x))
            .unwrap_or(def)
    };
    let signals = ["left", "right", "mid", "side"];
    json!({
        "size": one_of("size", &[1024.0, 2048.0, 4096.0, 8192.0, 16384.0], 4096.0),
        "fps": one_of("fps", &[15.0, 30.0, 60.0, 120.0], 60.0),
        "a": pick(p.get("a"), &signals, "left"),
        "b": pick(p.get("b"), &signals, "right"),
    })
}

fn clean_target(t: Option<&Value>) -> Option<Value> {
    let o = t?.as_object()?;
    match o.get("type")?.as_str()? {
        "master" => Some(json!({ "type": "master" })),
        "bank" => {
            let by = int_in(o.get("by"), -64.0, 64.0)?;
            (by != 0).then(|| json!({ "type": "bank", "by": by }))
        }
        "action" => {
            let a = o.get("action")?.as_str()?;
            let len = a.chars().count();
            (len > 0 && len < 80).then(|| json!({ "type": "action", "action": a }))
        }
        ty @ ("volume" | "pan" | "mute" | "solo" | "arm" | "touch") => {
            let track = int_in(o.get("track"), 0.0, 255.0)?;
            Some(json!({ "type": ty, "track": track }))
        }
        _ => None,
    }
}

/// Bindings read from saved settings: anything damaged is dropped, a source
/// bound twice keeps the last (in the place of the first).
pub fn clean_bindings(raw: Option<&Value>) -> Vec<Value> {
    let Some(list) = raw.and_then(Value::as_array) else {
        return vec![];
    };
    let mut seen: Vec<(String, Value)> = Vec::new();
    for r in list.iter().take(1024) {
        let Some(o) = r.as_object() else { continue };
        let s = o.get("source").and_then(Value::as_object);
        let kind = s
            .and_then(|s| s.get("kind"))
            .and_then(Value::as_str)
            .filter(|k| matches!(*k, "cc" | "note" | "bend"));
        let channel = int_in(s.and_then(|s| s.get("channel")), 0.0, 15.0);
        let number = int_in(s.and_then(|s| s.get("number")), 0.0, 127.0);
        let target = clean_target(o.get("target"));
        let (Some(kind), Some(channel), Some(number), Some(target)) =
            (kind, channel, number, target)
        else {
            continue;
        };
        let mut b = json!({ "source": { "kind": kind, "channel": channel, "number": number }, "target": target });
        if o.get("relative") == Some(&json!(true)) {
            b["relative"] = json!(true);
        }
        let key = format!("{kind}:{channel}:{number}");
        match seen.iter_mut().find(|x| x.0 == key) {
            Some(x) => x.1 = b,
            None => seen.push((key, b)),
        }
    }
    seen.into_iter().map(|x| x.1).collect()
}

/// Saved settings with defaults for anything missing, out of range or the wrong type.
pub fn sanitize_settings(raw: &Value) -> Value {
    let o = obj(Some(raw));
    let g = |k: &str| o.get(k);
    let d = default_settings();
    let dv = |k: &str| d[k].clone();
    let c = obj(g("colors"));
    let yellow = num(g("meterYellowDb"), -40.0, -1.0, -12.0);
    let valid_pair = |v: &Value| {
        v.as_array()
            .is_some_and(|a| a.len() == 2 && a.iter().all(|x| is_int_in(x, 0.0, 63.0)))
    };
    let output_pair = g("outputPair")
        .filter(|v| valid_pair(v))
        .cloned()
        .unwrap_or_else(|| dv("outputPair"));
    let mut speakers = Map::new();
    for (k, v) in obj(g("speakerOutputs")) {
        let key_ok = (1..=4).contains(&k.len()) && k.chars().all(|c| c.is_ascii_alphabetic());
        if key_ok && is_int_in(&v, 0.0, 63.0) {
            speakers.insert(k, v);
        }
    }
    let media = g("mediaFolder").and_then(Value::as_str).filter(|s| {
        !s.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|', '.'], "")
            .trim()
            .is_empty()
    });
    let media_folder = match media {
        Some(s) => slice(
            s.replace(['\\', '/', ':', '*', '?', '"', '<', '>', '|'], "")
                .trim(),
            60,
        ),
        None => "Audio".to_string(),
    };
    let marker = g("markerName")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map_or("Marker".to_string(), |s| slice(s.trim(), 40));
    let welcome = match g("welcomeRev")
        .and_then(Value::as_f64)
        .filter(|x| x.is_finite())
    {
        Some(x) => 0.0_f64.max(x.floor()),
        None if g("welcomeSeen") == Some(&json!(true)) => 1.0,
        None => 0.0,
    };
    let last_check = g("lastUpdateCheck")
        .and_then(Value::as_f64)
        .filter(|x| x.is_finite() && *x > 0.0)
        .map_or(0.0, f64::floor);
    let ports: Vec<Value> = match g("controllerPorts").and_then(Value::as_array) {
        Some(a) => a
            .iter()
            .filter(|p| {
                p.as_str()
                    .is_some_and(|s| !s.is_empty() && s.chars().count() < 200)
            })
            .take(16)
            .cloned()
            .collect(),
        None => vec![],
    };
    json!({
        "zoomStep": js_round(num(g("zoomStep"), 5.0, 100.0, 25.0)),
        "wheelAnchor": pick(g("wheelAnchor"), &["playhead", "between", "mouse"], "between"),
        "wheelZooms": boolean(g("wheelZooms"), true),
        "dropMany": if g("dropMany").and_then(Value::as_str) == Some("sequence") { "sequence" } else { "tracks" },
        "fadeShape": pick(g("fadeShape"), &["linear", "logarithmic", "exponential", "equal_power"], "cosine"),
        "surface": if g("surface").and_then(Value::as_str) == Some("classic") { "classic" } else { "standard" },
        "theme": match g("theme").and_then(Value::as_str) {
            Some("light") => "light",
            Some("system") => "system",
            _ => "dark",
        },
        "colors": {
            "wave": color(c.get("wave")),
            "playhead": color(c.get("playhead")),
            "accent": color(c.get("accent")),
            "laneSelected": color(c.get("laneSelected")),
            "spectrumLow": color(c.get("spectrumLow")),
            "spectrumHigh": color(c.get("spectrumHigh")),
            "spectrumScheme": pick(c.get("spectrumScheme"), &["aae", "classic"], ""),
            "phaseLow": color(c.get("phaseLow")),
            "phaseHigh": color(c.get("phaseHigh")),
        },
        "output": match g("output").and_then(Value::as_str) { Some(s) if !s.trim().is_empty() => json!(s), _ => Value::Null },
        "outputPair": output_pair,
        "speakerOutputs": Value::Object(speakers),
        "driver": if g("driver").and_then(Value::as_str) == Some("asio") { "asio" } else { "system" },
        "bufferFrames": int_in(g("bufferFrames"), 0.0, 8192.0).map_or(dv("bufferFrames"), |n| json!(n)),
        "meterSpeed": pick(g("meterSpeed"), &["medium", "slow"], "fast"),
        "meterRangeDb": match g("meterRangeDb").and_then(Value::as_f64) { Some(x) if [24.0, 48.0, 60.0, 72.0, 96.0, 120.0].contains(&x) => json!(x), _ => dv("meterRangeDb") },
        "customLufs": round_to(num(g("customLufs"), -40.0, -5.0, -18.0), 10.0),
        "customPeak": round_to(num(g("customPeak"), -10.0, 0.0, -1.0), 10.0),
        "waveInput": match g("waveInput").and_then(Value::as_str) { Some(s) if s.starts_with("Mono: ") || s.starts_with("Stereo: ") => json!(s), _ => Value::Null },
        "waveMonitor": boolean(g("waveMonitor"), false),
        "autosave": boolean(g("autosave"), true),
        "autosaveMinutes": js_round(num(g("autosaveMinutes"), 1.0, 60.0, 2.0)),
        "openChainOnAdd": boolean(g("openChainOnAdd"), true),
        "fillClickTexture": if g("fillClickTexture").and_then(Value::as_str) == Some("smooth") { "smooth" } else { "natural" },
        "realtimePlugins": boolean(g("realtimePlugins"), false),
        "soloExclusive": boolean(g("soloExclusive"), false),
        "autoThinPercent": round_to(num(g("autoThinPercent"), 0.0, 2.0, 0.3), 100.0),
        "meterYellowDb": yellow,
        "meterRedDb": num(g("meterRedDb"), yellow + 1.0, 0.0, (-3.0_f64).max(yellow + 1.0)),
        "backups": boolean(g("backups"), false),
        "backupCount": js_round(num(g("backupCount"), 1.0, 50.0, 5.0)),
        "copyMedia": pick(g("copyMedia"), &["ask", "never"], "always"),
        "mediaFolder": media_folder,
        "collectOnSave": boolean(g("collectOnSave"), true),
        "leaveLibraryFiles": boolean(g("leaveLibraryFiles"), true),
        "countdownSec": js_round(num(g("countdownSec"), 0.0, 10.0, 0.0)),
        "markerName": marker,
        "timecodeFps": g("timecodeFps").and_then(Value::as_f64).filter(|x| [24.0, 25.0, 30.0, 50.0, 60.0].contains(x)).unwrap_or(30.0),
        "phaseWheel": phase_prefs(g("phaseWheel")),
        "musicMode": boolean(g("musicMode"), false),
        "askBeforeSave": boolean(g("askBeforeSave"), false),
        "quickActionsBar": boolean(g("quickActionsBar"), false),
        "favorites": favorites(g("favorites")),
        "returnOnStop": boolean(g("returnOnStop"), true),
        "takeLanes": boolean(g("takeLanes"), true),
        "playbackSpeed": boolean(g("playbackSpeed"), false),
        "revealAfterExport": boolean(g("revealAfterExport"), true),
        "scrubAudio": boolean(g("scrubAudio"), false),
        "ampModelAlways": boolean(g("ampModelAlways"), false),
        "stopAtSelectionEnd": boolean(g("stopAtSelectionEnd"), true),
        "updateCheck": boolean(g("updateCheck"), false),
        "updateEvery": if g("updateEvery").and_then(Value::as_str) == Some("launch") { "launch" } else { "week" },
        "updateAutoInstall": boolean(g("updateAutoInstall"), false),
        "lastUpdateCheck": last_check,
        "metronome": metronome(g("metronome")),
        "controllerPorts": ports,
        "controllerBindings": clean_bindings(g("controllerBindings")),
        "controllerFeedback": g("controllerFeedback") == Some(&json!(true)),
        "welcomeRev": welcome,
        "spaceSnap": boolean(g("spaceSnap"), true),
    })
}

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn theme_accepts_light_system_and_dark() {
        for (given, want) in [
            ("light", "light"),
            ("system", "system"),
            ("dark", "dark"),
            ("purple", "dark"),
        ] {
            let s = sanitize_settings(&json!({ "theme": given }));
            assert_eq!(s["theme"], want, "given {given}");
        }
        assert_eq!(sanitize_settings(&json!({}))["theme"], "dark");
    }
}

#[cfg(test)]
mod scheme_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keeps_the_two_named_spectrogram_schemes() {
        for scheme in ["aae", "classic"] {
            let out = sanitize_settings(&json!({ "colors": { "spectrumScheme": scheme } }));
            assert_eq!(out["colors"]["spectrumScheme"], scheme);
        }
        let out = sanitize_settings(&json!({ "colors": { "spectrumScheme": "neon" } }));
        assert_eq!(out["colors"]["spectrumScheme"], "");
    }
}
