// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Saved presets for one effect: making one from a slot, keeping the bank of
//! saved presets in order, applying one to a slot, and reading a bank back from
//! preferences. The Rust copy of the pure parts of
//! `src/panels/voice-chain/effect-presets.ts`, tested against
//! `shared-fixtures/effect-presets.json`. The built-in preset tables stay in
//! TypeScript.
//!
//! Preset names are cut to 60 UTF-16 units, as the interface does, but never in
//! the middle of a character. Sorting uses `natural_cmp`, which follows the
//! browser order only for letters, digits and spaces.

use crate::effect_ui::{clamp_param, Param};
use crate::js_compat::js_trim;
use crate::library_view::natural_cmp;
use serde_json::Value;
use std::collections::BTreeMap;

/// Larger own data (an Amp Model capture) stays out of a preset.
pub const MAX_PRESET_DATA: usize = 16 * 1024;
/// Presets kept per effect.
pub const MAX_PRESETS_PER_EFFECT: usize = 60;
const MAX_NAME: usize = 60;

#[derive(Debug, Clone, PartialEq)]
pub struct EffectPreset {
    pub name: String,
    pub params: BTreeMap<String, f64>,
    pub data: Option<String>,
}

/// Saved presets by effect id.
pub type Bank = BTreeMap<String, Vec<EffectPreset>>;

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// The first `MAX_NAME` UTF-16 units of a name, whole characters only.
fn cut_name(s: &str) -> String {
    let mut units = 0;
    let mut out = String::new();
    for c in s.chars() {
        units += c.len_utf16();
        if units > MAX_NAME {
            break;
        }
        out.push(c);
    }
    out
}

fn by_name(a: &EffectPreset, b: &EffectPreset) -> std::cmp::Ordering {
    natural_cmp(&a.name, &b.name)
}

/// The setting's value in a slot: what is saved, held to the control's range, or
/// the default when none is saved.
pub fn param_value(slot_params: &BTreeMap<String, f64>, p: &Param) -> f64 {
    match slot_params.get(&p.key) {
        Some(v) if v.is_finite() => clamp_param(p, *v),
        _ => p.default,
    }
}

/// A preset from a slot: all of the effect's settings as they are now. The flag
/// is true when its data was too big to keep.
pub fn effect_preset_from(
    name: &str,
    slot_params: &BTreeMap<String, f64>,
    slot_data: Option<&str>,
    info: &[Param],
) -> (EffectPreset, bool) {
    let params = info
        .iter()
        .map(|p| (p.key.clone(), param_value(slot_params, p)))
        .collect();
    let mut preset = EffectPreset {
        name: cut_name(js_trim(name)),
        params,
        data: None,
    };
    let mut left_out = false;
    if let Some(d) = slot_data.filter(|d| !d.is_empty()) {
        if utf16_len(d) <= MAX_PRESET_DATA {
            preset.data = Some(d.to_string());
        } else {
            left_out = true;
        }
    }
    (preset, left_out)
}

/// The bank with `preset` saved for `effect`, replacing one of the same name.
pub fn bank_with(bank: &Bank, effect: &str, preset: EffectPreset) -> Bank {
    let lower = preset.name.to_lowercase();
    let mut list: Vec<EffectPreset> = bank
        .get(effect)
        .map(|l| {
            l.iter()
                .filter(|p| p.name.to_lowercase() != lower)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    list.push(preset);
    list.sort_by(by_name);
    list.truncate(MAX_PRESETS_PER_EFFECT);
    let mut next = bank.clone();
    next.insert(effect.to_string(), list);
    next
}

/// The bank without the named preset.
pub fn bank_without(bank: &Bank, effect: &str, name: &str) -> Bank {
    let list: Vec<EffectPreset> = bank
        .get(effect)
        .map(|l| l.iter().filter(|p| p.name != name).cloned().collect())
        .unwrap_or_default();
    let mut next = bank.clone();
    if list.is_empty() {
        next.remove(effect);
    } else {
        next.insert(effect.to_string(), list);
    }
    next
}

/// What a preset does to one slot: the new settings, the preset's name, and the
/// slot's data (the preset's own when it kept some, else what the slot had).
/// Settings the preset lacks keep their value; the slot's on/off state and
/// sidechain are not touched.
pub fn apply_effect_preset(
    slot_params: &BTreeMap<String, f64>,
    slot_data: Option<&str>,
    preset: &EffectPreset,
    info: &[Param],
) -> (BTreeMap<String, f64>, String, Option<String>) {
    let mut params = slot_params.clone();
    for p in info {
        if let Some(v) = preset.params.get(&p.key).filter(|v| v.is_finite()) {
            params.insert(p.key.clone(), clamp_param(p, *v));
        }
    }
    let data = match &preset.data {
        Some(d) => Some(d.clone()),
        None => slot_data.map(str::to_string),
    };
    (params, preset.name.clone(), data)
}

/// Settings read from a JSON object (or an array, whose entries are numbered):
/// only the finite numbers.
fn numbers_of(v: &Value) -> Option<BTreeMap<String, f64>> {
    let mut out = BTreeMap::new();
    match v {
        Value::Object(m) => {
            for (k, n) in m {
                if let Some(x) = n.as_f64() {
                    out.insert(k.clone(), x);
                }
            }
        }
        Value::Array(a) => {
            for (i, n) in a.iter().enumerate() {
                if let Some(x) = n.as_f64() {
                    out.insert(i.to_string(), x);
                }
            }
        }
        _ => return None,
    }
    Some(out)
}

/// Saved presets read back from preferences, keeping only what is well formed.
pub fn sanitize_bank(raw: &Value) -> Bank {
    let mut out = Bank::new();
    let Some(map) = raw.as_object() else {
        return out;
    };
    for (effect, list) in map {
        let Some(list) = list.as_array() else {
            continue;
        };
        let mut kept: Vec<EffectPreset> = Vec::new();
        for r in list {
            let Some(name) = r.get("name").and_then(Value::as_str) else {
                continue;
            };
            let trimmed = js_trim(name);
            if trimmed.is_empty() {
                continue;
            }
            let Some(params) = r
                .get("params")
                .filter(|p| !p.is_null())
                .and_then(numbers_of)
            else {
                continue;
            };
            let mut preset = EffectPreset {
                name: cut_name(trimmed),
                params,
                data: None,
            };
            if let Some(d) = r.get("data").and_then(Value::as_str) {
                if utf16_len(d) <= MAX_PRESET_DATA {
                    preset.data = Some(d.to_string());
                }
            }
            let lower = preset.name.to_lowercase();
            if !kept.iter().any(|p| p.name.to_lowercase() == lower) {
                kept.push(preset);
            }
        }
        if !kept.is_empty() {
            kept.sort_by(by_name);
            kept.truncate(MAX_PRESETS_PER_EFFECT);
            out.insert(effect.clone(), kept);
        }
    }
    out
}
