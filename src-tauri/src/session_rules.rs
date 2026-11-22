// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Rules that read or change a session document as JSON: which files to
//! copy into a session's folder and how the session then points at the
//! copies, the Session Properties choices and edit, the edit that swaps
//! clips for their bleed-cleaned copies, and the host profile list.
//! Matches `src/app/session-media.ts`, `src/app/session-properties.ts`,
//! `src/app/bleed.ts` (`replaceOps`) and the list in `src/app/host-profiles.ts`;
//! all are checked against `shared-fixtures/guide-media.json`.

use crate::js_compat::js_trim;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};

/// Comparable form of a folder or file path.
fn norm(p: &str, ignore_case: bool) -> String {
    let s = p.replace('\\', "/");
    let s = s.trim_end_matches('/');
    if ignore_case {
        s.to_lowercase()
    } else {
        s.to_string()
    }
}

/// Whether `path` is inside any of `dirs`.
pub fn inside_any(path: &str, dirs: &[String], ignore_case: bool) -> bool {
    let p = norm(path, ignore_case);
    dirs.iter().any(|d| {
        let n = norm(d, ignore_case);
        !n.is_empty() && p.starts_with(&format!("{n}/"))
    })
}

/// The folder holding a session file.
pub fn folder_of(session_file: &str) -> String {
    let units: Vec<char> = session_file.chars().collect();
    let i = units.iter().rposition(|&c| c == '/' || c == '\\');
    match i {
        Some(i) if i > 0 => units[..i].iter().collect(),
        _ => String::new(),
    }
}

/// The files to copy into a session saved at `session_file`: everything outside its
/// folder, and nothing inside the library folders in `leave`. Each path once, in order.
pub fn to_copy(
    paths: &[String],
    session_file: &str,
    leave: &[String],
    ignore_case: bool,
) -> Vec<String> {
    let home = vec![folder_of(session_file)];
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in paths {
        if !seen.insert(norm(p, ignore_case)) {
            continue;
        }
        if inside_any(p, &home, ignore_case) || inside_any(p, leave, ignore_case) {
            continue;
        }
        out.push(p.clone());
    }
    out
}

/// After copying: the session edits that point its media at the copies, and the Files
/// panel's changes as (id, new path). `moved` maps an old path to its copy.
pub fn repoint_plan(
    media: &[Value],
    pool: &[(String, String)],
    moved: &[(String, String)],
    ignore_case: bool,
) -> (Vec<Value>, Vec<(String, String)>) {
    let mut to: HashMap<String, &str> = HashMap::new();
    for (from, copy) in moved {
        to.insert(norm(from, ignore_case), copy.as_str());
    }
    let find = |p: &str| -> Option<&str> {
        let c = *to.get(&norm(p, ignore_case))?;
        (c != p).then_some(c)
    };
    let mut ops = Vec::new();
    for m in media {
        let path = m["path"].as_str().unwrap_or("");
        if let Some(c) = find(path) {
            let mut next = m.as_object().cloned().unwrap_or_default();
            next.insert("path".into(), json!(c));
            next.insert("relative_path".into(), Value::Null);
            ops.push(json!({ "op": "set_media", "media": Value::Object(next) }));
        }
    }
    let files = pool
        .iter()
        .filter_map(|(id, path)| find(path).map(|c| (id.clone(), c.to_string())))
        .collect();
    (ops, files)
}

pub const SAMPLE_RATES: [u32; 6] = [22_050, 32_000, 44_100, 48_000, 88_200, 96_000];
pub const BIT_DEPTHS: [(u32, &str); 3] = [(16, "16-bit"), (24, "24-bit"), (32, "32-bit float")];
pub const PAN_LAWS: [(&str, &str, &str); 2] = [
    (
        "constant_power",
        "-3 dB at center",
        "A sound keeps the same loudness as it moves across the mix.",
    ),
    (
        "balance",
        "Balance (full level at center)",
        "Centered plays both sides at full level; panning turns the far side down.",
    ),
];
pub const LOUDNESS_CHOICES: [(&str, &str); 6] = [
    ("", "Use my Settings choice"),
    ("podcast-stereo", "Podcast, stereo (-16 LUFS)"),
    ("podcast-mono", "Podcast, mono (-19 LUFS)"),
    ("spotify", "Louder platforms (-14 LUFS)"),
    ("broadcast", "Broadcast (your region)"),
    ("custom", "Custom (from Settings)"),
];

/// The values the Session Properties dialog starts from.
pub fn current_properties(doc: &Value) -> Value {
    json!({
        "name": doc["name"],
        "sampleRate": doc["sample_rate"],
        "bitDepth": doc["bit_depth"],
        "panLaw": doc.get("pan_law").filter(|v| !v.is_null()).cloned().unwrap_or(json!("balance")),
        "loudnessTarget": doc.get("loudness_target").filter(|v| !v.is_null()).cloned().unwrap_or(json!("")),
    })
}

/// The edit for the changed values, or None when nothing changed. `next` leaves
/// out `panLaw` and `loudnessTarget` when the caller does not change them.
pub fn properties_op(doc: &Value, next: &Value) -> Option<Value> {
    let mut ops: Vec<Value> = Vec::new();
    let name = js_trim(next["name"].as_str().unwrap_or(""));
    let mut info = Map::new();
    info.insert("op".into(), json!("set_session_info"));
    let mut changed = false;
    if !name.is_empty() && Some(name) != doc["name"].as_str() {
        info.insert("name".into(), json!(name));
        changed = true;
    }
    if next["bitDepth"] != doc["bit_depth"] {
        info.insert("bit_depth".into(), next["bitDepth"].clone());
        changed = true;
    }
    if changed {
        ops.push(Value::Object(info));
    }
    if next["sampleRate"] != doc["sample_rate"] {
        ops.push(json!({ "op": "set_sample_rate", "sample_rate": next["sampleRate"] }));
    }
    if let Some(law) = next.get("panLaw") {
        let cur = doc
            .get("pan_law")
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or(json!("balance"));
        if *law != cur {
            ops.push(json!({ "op": "set_pan_law", "pan_law": law }));
        }
    }
    if let Some(target) = next.get("loudnessTarget") {
        let cur = doc
            .get("loudness_target")
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or(json!(""));
        if *target != cur {
            let t = if target.as_str() == Some("") {
                Value::Null
            } else {
                target.clone()
            };
            ops.push(json!({ "op": "set_loudness_target", "target": t }));
        }
    }
    match ops.len() {
        0 => None,
        1 => Some(ops.remove(0)),
        _ => Some(json!({ "op": "batch", "ops": ops })),
    }
}

/// One edit for bleed-cleaned clips: add any media the session does not know yet, then
/// swap each clip for one on its copy. Each entry is (clip, media as read back).
pub fn bleed_replace_ops(
    track_id: &str,
    files: &[(Value, Value)],
    known: &HashSet<String>,
) -> Vec<Value> {
    let mut have = known.clone();
    let mut ops = Vec::new();
    let mut restored = Vec::new();
    for (clip, media) in files {
        let id = media["id"].as_str().unwrap_or("").to_string();
        if have.insert(id) {
            ops.push(json!({
                "op": "add_media",
                "media": {
                    "id": media["id"],
                    "path": media["path"],
                    "relative_path": Value::Null,
                    "sample_rate": media["sampleRate"],
                    "channels": media["channels"],
                    "length": media["length"],
                },
            }));
        }
        let mut c = clip.as_object().cloned().unwrap_or_default();
        c.insert("media_id".into(), media["id"].clone());
        c.insert("source_in".into(), json!(0));
        c.insert("source_out".into(), media["length"].clone());
        restored.push(json!([track_id, Value::Object(c)]));
    }
    let ids: Vec<Value> = files.iter().map(|(c, _)| c["id"].clone()).collect();
    ops.push(json!({ "op": "remove_clips", "ids": ids }));
    ops.push(json!({ "op": "restore_clips", "clips": restored }));
    ops
}

/// A built-in host profile.
#[derive(Debug, Clone, PartialEq)]
pub struct HostProfile {
    pub id: &'static str,
    pub name: &'static str,
    pub note: &'static str,
    pub gate_db: f64,
    pub guest: bool,
}

pub const HOST_PROFILES: [HostProfile; 3] = [
    HostProfile {
        id: "close-mic",
        name: "Close Mic",
        note: "A strong voice right on the mic.",
        gate_db: -14.0,
        guest: false,
    },
    HostProfile {
        id: "standard-mic",
        name: "Standard Mic",
        note: "A typical speaker at normal mic distance.",
        gate_db: -20.0,
        guest: false,
    },
    HostProfile {
        id: "remote-guest",
        name: "Remote Guest",
        note: "Someone calling in over the internet or phone.",
        gate_db: -30.0,
        guest: true,
    },
];

pub fn profile_by_id(id: Option<&str>) -> Option<&'static HostProfile> {
    HOST_PROFILES.iter().find(|p| Some(p.id) == id)
}
