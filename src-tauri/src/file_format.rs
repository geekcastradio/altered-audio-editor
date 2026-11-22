// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Files panel's number and text formats. The Rust copy of the pure parts
//! of `src/panels/files/format.ts`; both are tested against
//! `shared-fixtures/panel-rules.json`.

use crate::js_compat::{js_number_to_string, js_round, js_to_fixed};

/// Length as m:ss.mmm, or h:mm:ss.mmm past an hour.
pub fn format_duration(length: f64, sample_rate: f64) -> String {
    let total_ms = if sample_rate > 0.0 {
        js_round((length / sample_rate) * 1000.0)
    } else {
        0.0
    } as u64;
    let ms = total_ms % 1000;
    let total_sec = total_ms / 1000;
    let s = total_sec % 60;
    let total_min = total_sec / 60;
    let (m, h) = (total_min % 60, total_min / 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}.{ms:03}")
    } else {
        format!("{m}:{s:02}.{ms:03}")
    }
}

/// "44,100 Hz": whole rates get thousands commas, like JavaScript's
/// `toLocaleString("en-US")`; fractions keep up to three decimals.
pub fn format_rate(hz: f64) -> String {
    let fixed = js_to_fixed(hz.abs(), 3);
    let (int, frac) = fixed.split_once('.').unwrap_or((&fixed, ""));
    let mut grouped = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    let frac = frac.trim_end_matches('0');
    let sign = if hz < 0.0 { "-" } else { "" };
    if frac.is_empty() {
        format!("{sign}{grouped} Hz")
    } else {
        format!("{sign}{grouped}.{frac} Hz")
    }
}

pub fn format_channels(n: u32) -> String {
    match n {
        1 => "Mono".to_string(),
        2 => "Stereo".to_string(),
        _ => format!("{n} ch"),
    }
}

pub fn format_size(bytes: f64) -> String {
    if bytes < 1024.0 {
        return format!("{} B", js_number_to_string(bytes));
    }
    let units = ["KB", "MB", "GB"];
    let mut v = bytes / 1024.0;
    let mut u = 0;
    while v >= 1024.0 && u < units.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    format!(
        "{} {}",
        js_to_fixed(v, if v < 10.0 { 1 } else { 0 }),
        units[u]
    )
}

/// The status column text for a media item's state ("probing", "ready" or
/// "error").
pub fn status_text(status: &str) -> Option<&'static str> {
    match status {
        "probing" => Some("Reading"),
        "ready" => Some("Ready"),
        "error" => Some("Error"),
        _ => None,
    }
}
