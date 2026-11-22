// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Preset packs in Rust: saved Voice Chains and single-effect presets in one
//! small JSON file, read back and merged without replacing anything. Mirrors
//! `src/panels/voice-chain/preset-pack.ts`; tested against
//! `shared-fixtures/pack-recipes.json`.
//!
//! Two small differences, both harmless: objects compare with their keys in any
//! order (the TypeScript compares the written text), and sorting uses
//! `natural_cmp`, which follows the browser order only for letters, digits and
//! spaces.

use crate::effect_presets::{sanitize_bank, Bank, EffectPreset, MAX_PRESETS_PER_EFFECT};
use crate::js_compat::js_slice_units;
use crate::library_view::natural_cmp;
use crate::voice_chain::sanitize_presets;
use serde_json::{json, Map, Value};
use std::collections::HashSet;

pub const PACK_FORMAT: &str = "altaudio.presets";
pub const PACK_VERSION: f64 = 1.0;

pub fn bank_json(bank: &Bank) -> Value {
    let mut m = Map::new();
    for (effect, list) in bank {
        let items: Vec<Value> = list.iter().map(preset_json).collect();
        m.insert(effect.clone(), Value::Array(items));
    }
    Value::Object(m)
}

fn preset_json(p: &EffectPreset) -> Value {
    let mut m = Map::new();
    m.insert("name".into(), json!(p.name));
    m.insert("params".into(), json!(p.params));
    if let Some(d) = &p.data {
        m.insert("data".into(), json!(d));
    }
    Value::Object(m)
}

/// The pack as text, pretty printed so it diffs and reads well.
pub fn pack_presets(chains: &[Value], bank: &Bank) -> String {
    let v = json!({ "format": PACK_FORMAT, "version": PACK_VERSION as i64, "chains": chains, "effects": bank_json(bank) });
    serde_json::to_string_pretty(&v).unwrap_or_default() + "\n"
}

#[derive(Debug, Clone)]
pub struct ReadPack {
    pub chains: Vec<Value>,
    pub bank: Bank,
}

/// A pack read from text, or the reason it cannot be used.
pub fn read_pack(text: &str) -> Result<ReadPack, String> {
    let Ok(raw) = serde_json::from_str::<Value>(text) else {
        return Err("That file is not a preset pack.".into());
    };
    if !raw.is_object() && !raw.is_array() {
        return Err("That file is not an Altered Audio preset pack.".into());
    }
    if raw.get("format").and_then(Value::as_str) != Some(PACK_FORMAT) {
        return Err("That file is not an Altered Audio preset pack.".into());
    }
    match raw.get("version").and_then(Value::as_f64) {
        Some(v) if v <= PACK_VERSION => {}
        _ => return Err(
            "That pack was made by a newer version of Altered Audio. Update the app to read it."
                .into(),
        ),
    }
    let null = Value::Null;
    Ok(ReadPack {
        chains: sanitize_presets(raw.get("chains").unwrap_or(&null)),
        bank: sanitize_bank(raw.get("effects").unwrap_or(&null)),
    })
}

/// `name` made unique among the names `taken` accepts, by adding " (2)", " (3)"
/// and so on, within 60 characters.
pub fn unique_name(name: &str, taken: &dyn Fn(&str) -> bool) -> String {
    if !taken(name) {
        return name.to_string();
    }
    let mut i = 2;
    loop {
        let suffix = format!(" ({i})");
        let next = js_slice_units(name, 60 - suffix.len()) + &suffix;
        if !taken(&next) {
            return next;
        }
        i += 1;
    }
}

#[derive(Debug, Clone)]
pub struct Merged {
    pub chains: Vec<Value>,
    pub bank: Bank,
    pub added_chains: usize,
    pub added_effects: usize,
    /// Presets that kept their settings but got a new name because theirs was taken.
    pub renamed: usize,
    /// Single-effect presets left out because that effect already holds the most it can.
    pub full: usize,
}

/// Equal as JavaScript would see them after reading JSON: numbers by value
/// (1 and 1.0 match), objects with their keys in any order.
fn same_json(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same_json(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| same_json(v, w)))
        }
        _ => a == b,
    }
}

fn name_of(v: &Value) -> &str {
    v["name"].as_str().unwrap_or("")
}

/// Your presets plus the pack's. Nothing you have is replaced. A preset that
/// matches one you have in name and settings is skipped; one that only shares
/// the name comes in under a new name. `reserved` are names that cannot be used.
pub fn merge_pack(chains: &[Value], bank: &Bank, pack: &ReadPack, reserved: &[String]) -> Merged {
    let lower = |s: &str| s.to_lowercase();
    let mut out_chains: Vec<Value> = chains.to_vec();
    let mut used: HashSet<String> = reserved
        .iter()
        .map(String::as_str)
        .chain(chains.iter().map(name_of))
        .map(lower)
        .collect();
    let (mut added_chains, mut added_effects, mut renamed, mut full) = (0, 0, 0, 0);
    for c in &pack.chains {
        let mine = out_chains
            .iter()
            .find(|x| lower(name_of(x)) == lower(name_of(c)));
        if mine.is_some_and(|m| same_json(&m["chain"], &c["chain"])) {
            continue;
        }
        let name = unique_name(name_of(c), &|n| used.contains(&lower(n)));
        if name != name_of(c) {
            renamed += 1;
        }
        used.insert(lower(&name));
        let mut added = c.clone();
        added["name"] = json!(name);
        out_chains.push(added);
        added_chains += 1;
    }
    out_chains.sort_by(|a, b| natural_cmp(name_of(a), name_of(b)));
    let mut out_bank = bank.clone();
    for (effect, list) in &pack.bank {
        let mut have: Vec<EffectPreset> = out_bank.get(effect).cloned().unwrap_or_default();
        for p in list {
            let mine = have.iter().find(|x| lower(&x.name) == lower(&p.name));
            if mine.is_some_and(|m| m.params == p.params && m.data == p.data) {
                continue;
            }
            if have.len() >= MAX_PRESETS_PER_EFFECT {
                full += 1;
                continue;
            }
            let name = unique_name(&p.name, &|n| {
                have.iter().any(|x| lower(&x.name) == lower(n))
            });
            if name != p.name {
                renamed += 1;
            }
            have.push(EffectPreset { name, ..p.clone() });
            added_effects += 1;
        }
        if !have.is_empty() {
            have.sort_by(|a, b| natural_cmp(&a.name, &b.name));
            out_bank.insert(effect.clone(), have);
        }
    }
    Merged {
        chains: out_chains,
        bank: out_bank,
        added_chains,
        added_effects,
        renamed,
        full,
    }
}
