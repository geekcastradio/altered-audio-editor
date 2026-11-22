// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Track visibility rules: saved sets of hidden tracks, matched by name, and
//! the clip overlap test. The Rust copy of the pure parts of
//! `src/panels/editor/visibility.ts`; both are tested against
//! `shared-fixtures/visibility.json`. Names are compared lower-cased and
//! trimmed the way JavaScript does it (plain text only; exotic case rules
//! such as the Turkish dotted capital I are not matched).

use crate::js_compat::js_trim;
use serde_json::Value;

/// Most presets kept; the oldest is dropped when a new one is saved.
pub const MAX_VIS_PRESETS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisPreset {
    pub name: String,
    pub hidden: Vec<String>,
}

fn norm(s: &str) -> String {
    js_trim(s).to_lowercase()
}

/// For each track (id, hidden now), the new hidden flag when only the tracks
/// `keep` accepts should show. Only tracks that change are listed.
pub fn show_only(tracks: &[(String, bool)], keep: &[bool]) -> Vec<(String, bool)> {
    tracks
        .iter()
        .zip(keep)
        .filter_map(|((id, was), k)| {
            let hidden = !k;
            (hidden != *was).then(|| (id.clone(), hidden))
        })
        .collect()
}

/// Names of the tracks that are hidden now.
pub fn hidden_names(tracks: &[(String, bool)]) -> Vec<String> {
    tracks
        .iter()
        .filter(|(_, h)| *h)
        .map(|(n, _)| n.clone())
        .collect()
}

/// True when the track shows under a preset: its name is not in `hidden`.
pub fn kept_by_preset(hidden: &[String], name: &str) -> bool {
    let n = norm(name);
    !hidden.iter().any(|h| norm(h) == n)
}

/// Adds or replaces a preset by name (case ignored) and keeps the newest
/// `MAX_VIS_PRESETS`.
pub fn with_preset(list: &[VisPreset], preset: VisPreset) -> Vec<VisPreset> {
    let key = norm(&preset.name);
    let mut out: Vec<VisPreset> = list
        .iter()
        .filter(|p| norm(&p.name) != key)
        .cloned()
        .collect();
    out.push(preset);
    let drop = out.len().saturating_sub(MAX_VIS_PRESETS);
    out.split_off(drop)
}

/// Reads saved presets back, dropping anything that is not a name with a list
/// of names.
pub fn sanitize_presets(raw: &Value) -> Vec<VisPreset> {
    let Some(items) = raw.as_array() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for p in items {
        let (Some(name), Some(hidden)) = (
            p.get("name").and_then(Value::as_str),
            p.get("hidden").and_then(Value::as_array),
        ) else {
            continue;
        };
        if js_trim(name).is_empty() {
            continue;
        }
        out.push(VisPreset {
            name: js_trim(name).to_string(),
            hidden: hidden
                .iter()
                .filter_map(|h| h.as_str().map(str::to_string))
                .collect(),
        });
    }
    let drop = out.len().saturating_sub(MAX_VIS_PRESETS);
    out.split_off(drop)
}

/// True when any clip span (start, length) overlaps [start, end). A point
/// (start == end) uses start.
pub fn has_clip_in(spans: &[(f64, f64)], start: f64, end: f64) -> bool {
    let hi = end.max(start + 1.0);
    spans.iter().any(|&(s, l)| s < hi && s + l > start)
}
