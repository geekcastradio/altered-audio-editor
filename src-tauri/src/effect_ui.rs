// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Effect favorites and the effect window's controls. The Rust copy of the pure
//! parts of `src/panels/voice-chain/favorites.ts` and `effect-controls.ts`,
//! tested against `shared-fixtures/effect-ui.json`.

use crate::js_compat::{is_js_space, js_round, js_string_to_number, js_to_fixed};
use serde_json::Value;

pub const MAX_FAVORITES: usize = 40;

/// What a fresh install starts with: the effects a podcast edit reaches for most.
pub const STARTER_FAVORITES: [&str; 9] = [
    "altaudio.hum_cut",
    "altaudio.noise_reducer",
    "altaudio.dc_offset",
    "altaudio.deesser",
    "altaudio.dynamics",
    "altaudio.speech_leveler",
    "altaudio.parametric_eq",
    "altaudio.limiter",
    "altaudio.pitch_shift",
];

/// A saved list with anything unreadable dropped, each id once.
pub fn sanitize_favorites(raw: &Value) -> Vec<String> {
    let Some(list) = raw.as_array() else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for v in list {
        if let Some(s) = v.as_str() {
            // JS string length counts UTF-16 units.
            let n = s.encode_utf16().count();
            if n > 0 && n < 200 && !out.iter().any(|x| x == s) {
                out.push(s.to_string());
            }
        }
    }
    out.truncate(MAX_FAVORITES);
    out
}

/// The list with `id` added when it was missing and removed when it was there.
pub fn toggle_favorite(list: &[String], id: &str) -> Vec<String> {
    if list.iter().any(|x| x == id) {
        return list.iter().filter(|x| *x != id).cloned().collect();
    }
    let mut next = list.to_vec();
    next.push(id.to_string());
    let skip = next.len().saturating_sub(MAX_FAVORITES);
    next.split_off(skip)
}

/// What the interface knows about one control.
#[derive(Debug, Clone)]
pub struct Param {
    pub key: String,
    pub name: String,
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step: f64,
    pub toggle: bool,
    pub group: Option<String>,
    pub hidden: bool,
}

/// A group of controls drawn as one column, by control key.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    pub power: Option<String>,
    pub params: Vec<String>,
}

/// Splits controls into columns by group, keeping their order. Hidden ones are
/// left out.
pub fn control_columns(params: &[Param]) -> Vec<Column> {
    let mut out: Vec<Column> = Vec::new();
    for p in params {
        if p.hidden {
            continue;
        }
        let name = p.group.clone().unwrap_or_default();
        if out.last().is_none_or(|c| c.name != name) {
            out.push(Column {
                name: name.clone(),
                power: None,
                params: Vec::new(),
            });
        }
        let col = out.last_mut().unwrap();
        if !name.is_empty() && p.toggle && p.name == "On" && col.power.is_none() {
            col.power = Some(p.key.clone());
        } else {
            col.params.push(p.key.clone());
        }
    }
    out
}

/// A value held to the control's step and range.
pub fn clamp_param(p: &Param, value: f64) -> f64 {
    if !value.is_finite() {
        return p.default;
    }
    let snapped = if p.step > 0.0 {
        js_round(value / p.step) * p.step
    } else {
        value
    };
    // Trim float noise from the step maths.
    let trimmed = js_string_to_number(&js_to_fixed(snapped, 6)).unwrap_or(snapped);
    #[allow(clippy::manual_clamp)] // clamp panics when min > max; this follows the interface.
    let held = trimmed.max(p.min).min(p.max);
    held
}

/// A typed value, read in the control's unit: "1.5k" or "1.5 kHz" is 1500 Hz,
/// "-6" or "-6 dB" is -6 dB. Clamped to the control's range. None when the text
/// has no number in it.
pub fn parse_param(p: &Param, text: &str) -> Option<f64> {
    // JS replace with a string swaps only the first comma.
    let text = text.replacen(',', ".", 1);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() && is_js_space(chars[i]) {
        i += 1;
    }
    let start = i;
    if i < chars.len() && (chars[i] == '-' || chars[i] == '+') {
        i += 1;
    }
    let int_start = i;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    let int_len = i - int_start;
    if int_len > 0 {
        // Digits, then an optional dot and more digits.
        if i < chars.len() && chars[i] == '.' {
            i += 1;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
        }
    } else if i < chars.len() && chars[i] == '.' {
        i += 1;
        let frac_start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        if i == frac_start {
            return None;
        }
    } else {
        return None;
    }
    let number: String = chars[start..i].iter().collect();
    while i < chars.len() && is_js_space(chars[i]) {
        i += 1;
    }
    let rest: String = chars[i..].iter().collect::<String>().to_lowercase();
    let kilo = rest.starts_with('k');
    let mut v = js_string_to_number(&number)?;
    if !v.is_finite() {
        return None;
    }
    if kilo && p.unit == "Hz" {
        v *= 1000.0;
    }
    Some(clamp_param(p, v))
}
