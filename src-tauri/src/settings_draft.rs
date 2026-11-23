// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Edit > Settings draft as it is saved, and sizes as short text. The Rust copy of
//! `cleanDraft` and `sizeText` in `src/app/settings/pages.ts`, tested against
//! `shared-fixtures/settings-draft.json`. The dialog stays in TypeScript.

use crate::js_compat::{js_number_to_string, js_round, js_to_fixed};
use crate::settings_clean::sanitize_settings;
use serde_json::{json, Map, Value};

/// `clamp` from the TypeScript: anything that is not a finite number gives the default.
fn clamp(v: Option<&Value>, lo: f64, hi: f64, def: f64) -> f64 {
    match v.and_then(Value::as_f64) {
        Some(x) if x.is_finite() => x.clamp(lo, hi),
        _ => def,
    }
}

/// The draft as it should be saved: every value checked and in range.
pub fn clean_draft(d: &Value) -> Value {
    let editor = &d["editor"];
    let time_format = ["samples", "timecode", "seconds", "bars"]
        .into_iter()
        .find(|f| editor["timeFormat"].as_str() == Some(f))
        .unwrap_or("time");
    let mut record: Map<String, Value> = d["record"].as_object().cloned().unwrap_or_default();
    let offset = js_round(clamp(record.get("offsetMs"), 0.0, 1000.0, 0.0));
    let preroll = clamp(record.get("prerollSec"), 0.0, 10.0, 2.0);
    record.insert("offsetMs".into(), json!(offset));
    record.insert("prerollSec".into(), json!(preroll));
    json!({
        "app": sanitize_settings(&d["app"]),
        "editor": {
            "followPlayhead": editor["followPlayhead"] == json!(true),
            "autoCrossfade": editor["autoCrossfade"] == json!(true),
            "timeFormat": time_format,
        },
        "record": Value::Object(record),
        "loudness": d["loudness"].clone(),
    })
}

/// Bytes as a short size.
pub fn size_text(bytes: f64) -> String {
    if bytes < 1024.0 {
        return format!("{} B", js_number_to_string(bytes));
    }
    let units = ["KB", "MB", "GB"];
    let mut v = bytes / 1024.0;
    let mut i = 0;
    while v >= 1024.0 && i < units.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!(
        "{} {}",
        js_to_fixed(v, if v < 10.0 { 1 } else { 0 }),
        units[i]
    )
}
