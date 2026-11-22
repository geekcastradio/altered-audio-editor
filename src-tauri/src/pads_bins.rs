// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Soundboard pads and library bins: the pure rules. The Rust copy of
//! `src/panels/soundboard/soundboard.ts` (pad labels, reading saved pads back,
//! putting a file on a pad) and the pure parts of
//! `src/panels/libraries/bins.ts` (starter bins, badge letters, the default
//! color). Tested against `shared-fixtures/panel-lists.json`.

use crate::js_compat::{is_js_space, js_slice_units, js_trim};
use serde_json::Value;

pub const PAD_COUNT: usize = 9;
const MAX_LABEL: usize = 40;
const MAX_PATH: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct Pad {
    pub path: Option<String>,
    pub label: String,
}

pub fn empty_pad() -> Pad {
    Pad {
        path: None,
        label: String::new(),
    }
}

/// The file name without its folder or extension.
pub fn pad_label(path: &str) -> String {
    let name = path.rsplit(['\\', '/']).next().unwrap_or(path);
    let stem = match name.rfind('.') {
        Some(dot) if dot > 0 => &name[..dot],
        _ => name,
    };
    js_slice_units(stem, MAX_LABEL)
}

/// Saved pads checked: always nine, each with a path or empty.
pub fn sanitize_pads(raw: &Value) -> Vec<Pad> {
    let list: &[Value] = raw.as_array().map(Vec::as_slice).unwrap_or(&[]);
    (0..PAD_COUNT)
        .map(|i| {
            let Some(p) = list.get(i) else {
                return empty_pad();
            };
            let path = match p.get("path").and_then(Value::as_str) {
                Some(s) if !s.is_empty() && s.encode_utf16().count() <= MAX_PATH => s,
                _ => return empty_pad(),
            };
            let label = match p.get("label").and_then(Value::as_str) {
                Some(l) if !js_trim(l).is_empty() => js_slice_units(js_trim(l), MAX_LABEL),
                _ => pad_label(path),
            };
            Pad {
                path: Some(path.to_string()),
                label,
            }
        })
        .collect()
}

/// The pads with slot `i` holding `path`.
pub fn with_pad(pads: &[Pad], i: usize, path: Option<&str>) -> Vec<Pad> {
    pads.iter()
        .enumerate()
        .map(|(k, p)| match (k == i, path) {
            (true, Some(path)) if !path.is_empty() => Pad {
                path: Some(path.to_string()),
                label: pad_label(path),
            },
            (true, _) => empty_pad(),
            _ => p.clone(),
        })
        .collect()
}

/// Palette tokens used for bins and show badges.
pub const PALETTE: [&str; 6] = [
    "--clip-1", "--clip-2", "--clip-3", "--clip-4", "--clip-5", "--clip-6",
];

/// The bins a new show library starts with.
pub const STARTER_BINS: [(&str, Option<&str>); 7] = [
    ("Intro / Outro", Some("--clip-2")),
    ("Ads", Some("--clip-4")),
    ("Promos", Some("--clip-5")),
    ("Clips", Some("--clip-1")),
    ("Music Beds", Some("--clip-3")),
    ("Sound Effects", Some("--clip-6")),
    ("Legal Reads", None),
];

/// Starter bins the library does not have yet, matched by name ignoring case.
pub fn missing_starter_bins(have: &[String]) -> Vec<(&'static str, Option<&'static str>)> {
    let have: std::collections::HashSet<String> =
        have.iter().map(|n| js_trim(n).to_lowercase()).collect();
    STARTER_BINS
        .iter()
        .filter(|(n, _)| !have.contains(&n.to_lowercase()))
        .copied()
        .collect()
}

/// Up to two letters for a show badge: "Morning Show" gives "MS", "SHOWCAST"
/// gives "SH".
pub fn initials(name: &str) -> String {
    let words: Vec<&str> = js_trim(name)
        .split(|c: char| is_js_space(c) || c == '_' || c == '-')
        .filter(|w| !w.is_empty())
        .collect();
    match words.as_slice() {
        [] => "?".to_string(),
        [one] => js_slice_units(one, 2).to_uppercase(),
        [a, b, ..] => {
            let first = |w: &str| w.chars().next().map(String::from).unwrap_or_default();
            (first(a) + &first(b)).to_uppercase()
        }
    }
}

/// A stable palette color for a library that has none set.
pub fn default_color(name: &str) -> &'static str {
    let mut h: u32 = 0;
    for c in name.chars() {
        let mut buf = [0u16; 2];
        let unit = c.encode_utf16(&mut buf)[0] as u32;
        h = h.wrapping_mul(31).wrapping_add(unit);
    }
    PALETTE[(h as usize) % PALETTE.len()]
}
