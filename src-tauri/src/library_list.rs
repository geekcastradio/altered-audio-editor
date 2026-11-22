// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The list of libraries the app knows about, and the small file that carries it between
//! computers. The Rust copy of `src/stores/library-list.ts`, tested against
//! `shared-fixtures/keys-tempo-midi.json`. The file holds names, colors and folder paths only.

use crate::js_compat::{js_slice_units, js_trim};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KnownLibrary {
    /// The library folder.
    pub dir: String,
    pub name: String,
    /// Palette token or #rrggbb, or None for the automatic color.
    pub color: Option<String>,
}

pub const LIST_FORMAT: &str = "altaudio.libraries";
pub const LIST_VERSION: f64 = 1.0;

/// The most libraries a list holds, so a damaged file cannot flood the app.
pub const MAX_KNOWN: usize = 500;

/// Last folder name in a path, used as a name when none is known.
pub fn last_segment(dir: &str) -> String {
    dir.split(['\\', '/'])
        .rfind(|p| !p.is_empty())
        .unwrap_or(dir)
        .to_string()
}

/// The same folder written two ways (slash style, letter case, trailing slash).
pub fn same_dir(a: &str, b: &str) -> bool {
    fn clean(p: &str) -> String {
        p.replace('\\', "/").trim_end_matches('/').to_lowercase()
    }
    clean(a) == clean(b)
}

/// Reads a stored or imported list: keeps good entries, drops duplicates and junk.
pub fn sanitize_known(raw: &Value) -> Vec<KnownLibrary> {
    let mut out: Vec<KnownLibrary> = Vec::new();
    let Some(list) = raw.as_array() else {
        return out;
    };
    for r in list {
        let Some(e) = r.as_object() else {
            continue;
        };
        let Some(dir) = e.get("dir").and_then(Value::as_str) else {
            continue;
        };
        if js_trim(dir).is_empty() {
            continue;
        }
        if out.iter().any(|k| same_dir(&k.dir, dir)) {
            continue;
        }
        let name = match e.get("name").and_then(Value::as_str) {
            Some(n) if !js_trim(n).is_empty() => js_slice_units(js_trim(n), 120),
            _ => last_segment(dir),
        };
        let color = match e.get("color").and_then(Value::as_str) {
            Some(c) if !c.is_empty() => Some(c.to_string()),
            _ => None,
        };
        out.push(KnownLibrary {
            dir: dir.to_string(),
            name,
            color,
        });
        if out.len() >= MAX_KNOWN {
            break;
        }
    }
    out
}

/// The list as text: pretty printed so it reads and diffs well.
pub fn pack_library_list(known: &[KnownLibrary]) -> String {
    // A struct, not a JSON value, so the keys keep this order.
    #[derive(Serialize)]
    struct File<'a> {
        format: &'a str,
        version: u32,
        libraries: &'a [KnownLibrary],
    }
    let mut text = serde_json::to_string_pretty(&File {
        format: LIST_FORMAT,
        version: 1,
        libraries: known,
    })
    .unwrap_or_default();
    text.push('\n');
    text
}

/// A list read from text, or the reason it cannot be used.
pub fn read_library_list(text: &str) -> Result<Vec<KnownLibrary>, String> {
    let Ok(raw) = serde_json::from_str::<Value>(text) else {
        return Err("That file is not a library list.".to_string());
    };
    if raw.get("format").and_then(Value::as_str) != Some(LIST_FORMAT) {
        return Err("That file is not an Altered Audio library list.".to_string());
    }
    match raw.get("version").and_then(Value::as_f64) {
        Some(v) if v <= LIST_VERSION => {}
        _ => return Err(
            "That list was made by a newer version of Altered Audio. Update the app to read it."
                .to_string(),
        ),
    }
    Ok(sanitize_known(raw.get("libraries").unwrap_or(&Value::Null)))
}
