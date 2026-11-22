// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Voice Chain edits as plain functions, in Rust.
//!
//! This mirrors `src/panels/voice-chain/chain.ts` (slot edits, slider maths and
//! the preset rules) and is tested against `shared-fixtures/voice-chain.json`,
//! which the TypeScript test reads too. A chain is the effect chain as JSON,
//! as the interface sends it to the core; every function returns a new chain.
//! The built-in presets are data in `data/voice-presets.json`, written from the
//! TypeScript list; a TypeScript test keeps the two equal.

use crate::js_compat::{
    is_js_space, js_number_to_string, js_round, js_string_to_number, js_to_fixed, js_trim,
};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::sync::OnceLock;

pub const MAX_SLOTS: usize = 16;

/// What the interface knows about one control (`EffectParamInfo`).
#[derive(Clone, Debug)]
pub struct ParamInfo {
    pub key: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step: f64,
    pub log: bool,
    pub toggle: bool,
    pub unit: Option<String>,
    pub choices: Vec<String>,
}

impl ParamInfo {
    pub fn from_json(v: &Value) -> Self {
        let n = |k: &str| v.get(k).and_then(Value::as_f64).unwrap_or(f64::NAN);
        let truthy = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(false);
        Self {
            key: v
                .get("key")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            min: n("min"),
            max: n("max"),
            default: n("default"),
            step: n("step"),
            log: truthy("log"),
            toggle: truthy("toggle"),
            unit: v.get("unit").and_then(Value::as_str).map(str::to_string),
            choices: v
                .get("choices")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|c| c.as_str().unwrap_or("").to_string())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

fn empty_chain() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("enabled".into(), json!(true));
    m.insert("input_gain_db".into(), json!(0));
    m.insert("output_gain_db".into(), json!(0));
    m.insert("mix".into(), json!(1));
    m.insert("slots".into(), json!([]));
    m
}

fn slots_of(chain: &Value) -> Vec<Value> {
    chain
        .get("slots")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn with_slots(chain: &Value, slots: Vec<Value>) -> Value {
    let mut m = chain.as_object().cloned().unwrap_or_default();
    m.insert("slots".into(), Value::Array(slots));
    Value::Object(m)
}

fn id_of(s: &Value) -> &str {
    s.get("id").and_then(Value::as_str).unwrap_or("")
}

/// The chain with every missing field filled in (`chainOf`).
pub fn chain_of(c: Option<&Value>) -> Value {
    let mut m = empty_chain();
    if let Some(obj) = c.and_then(Value::as_object) {
        for (k, v) in obj {
            m.insert(k.clone(), v.clone());
        }
        let slots = match obj.get("slots") {
            Some(v) if !v.is_null() => v.clone(),
            _ => json!([]),
        };
        m.insert("slots".into(), slots);
    }
    Value::Object(m)
}

/// A new slot with every parameter at its default. `info` is `{ id, params: [{ key, default }] }`.
pub fn new_slot(info: &Value, id: &str) -> Value {
    let mut params = Map::new();
    for p in info
        .get("params")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
    {
        if let (Some(k), Some(d)) = (p.get("key").and_then(Value::as_str), p.get("default")) {
            params.insert(k.to_string(), d.clone());
        }
    }
    json!({ "id": id, "effect": info.get("id").cloned().unwrap_or(Value::Null), "enabled": true, "params": params, "preset": null })
}

/// Adds an effect at `index` (the end when None). None when the chain is full.
pub fn add_slot(chain: &Value, slot: &Value, index: Option<f64>) -> Option<Value> {
    let mut slots = slots_of(chain);
    if slots.len() >= MAX_SLOTS {
        return None;
    }
    let at = match index {
        None => slots.len(),
        Some(i) => i.min(slots.len() as f64).max(0.0) as usize,
    };
    slots.insert(at, slot.clone());
    Some(with_slots(chain, slots))
}

fn position_of(s: &Value) -> Option<f64> {
    s.get("position").and_then(Value::as_f64)
}

/// The chain as its numbered slots: each effect in the slot it asked for (its
/// `position`), or the one after the effect before it.
pub fn layout_slots(chain: &Value) -> Vec<Option<Value>> {
    let mut out: Vec<Option<Value>> = vec![None; MAX_SLOTS];
    let slots = slots_of(chain);
    let n = slots.len() as f64;
    let mut next = 0.0_f64;
    for (k, s) in slots.iter().enumerate() {
        // Leave room for the effects still to come.
        let latest = MAX_SLOTS as f64 - (n - k as f64);
        let at = latest.min(next.max(position_of(s).unwrap_or(next)));
        if at >= 0.0 && at < MAX_SLOTS as f64 && at.fract() == 0.0 {
            out[at as usize] = Some(s.clone());
        }
        next = at + 1.0;
    }
    out
}

fn count_before(chain: &Value, at: usize, skip: Option<&str>) -> usize {
    layout_slots(chain)
        .into_iter()
        .take(at)
        .filter(|s| s.as_ref().is_some_and(|s| Some(id_of(s)) != skip))
        .count()
}

fn with_position(s: &Value, position: Value) -> Value {
    let mut m = s.as_object().cloned().unwrap_or_default();
    m.insert("position".into(), position);
    Value::Object(m)
}

/// Every effect pinned to the slot it shows in now, so moving one leaves the rest in place.
fn pinned(chain: &Value) -> Vec<Value> {
    let layout = layout_slots(chain);
    slots_of(chain)
        .iter()
        .map(|s| {
            let at = layout
                .iter()
                .position(|x| x.as_ref().is_some_and(|x| id_of(x) == id_of(s)));
            let pos = match at {
                Some(i) => json!(i),
                None => s
                    .get("position")
                    .filter(|p| !p.is_null())
                    .cloned()
                    .unwrap_or(Value::Null),
            };
            with_position(s, pos)
        })
        .collect()
}

/// Pushes effects down where one now sits on or above the one before.
fn bumped(slots: Vec<Value>) -> Vec<Value> {
    let mut next = 0.0_f64;
    slots
        .into_iter()
        .map(|s| {
            let at = next.max(position_of(&s).unwrap_or(next));
            next = at + 1.0;
            if position_of(&s) == Some(at) {
                s
            } else {
                with_position(&s, json!(at))
            }
        })
        .collect()
}

/// `Math.max(0, Math.min(MAX_SLOTS - 1, at))`: keeps a fraction, and NaN stays NaN.
fn clamp_slot_index(at: f64) -> f64 {
    if at.is_nan() {
        at
    } else {
        at.clamp(0.0, (MAX_SLOTS - 1) as f64)
    }
}

/// What `slice(0, to)` takes: the whole part, and nothing for NaN.
fn whole(to: f64) -> usize {
    if to.is_nan() {
        0
    } else {
        to as usize
    }
}

/// The position as JSON: a fraction stays, NaN becomes null.
fn position_json(to: f64) -> Value {
    serde_json::Number::from_f64(to).map_or(Value::Null, Value::Number)
}

/// Puts an effect in numbered slot `at`. An effect already there moves down one.
/// None when the chain is full.
pub fn add_slot_at(chain: &Value, slot: &Value, at: f64) -> Option<Value> {
    if slots_of(chain).len() >= MAX_SLOTS {
        return None;
    }
    let to = clamp_slot_index(at);
    let mut slots = pinned(chain);
    let before = count_before(chain, whole(to), None).min(slots.len());
    slots.insert(before, with_position(slot, position_json(to)));
    Some(with_slots(chain, bumped(slots)))
}

/// Moves an effect to numbered slot `at`; an effect already there moves down one.
pub fn move_slot_to_index(chain: &Value, id: &str, at: f64) -> Value {
    let Some(s) = slots_of(chain).into_iter().find(|x| id_of(x) == id) else {
        return chain.clone();
    };
    let to = clamp_slot_index(at);
    let before = count_before(chain, whole(to), Some(id));
    let mut slots: Vec<Value> = pinned(chain)
        .into_iter()
        .filter(|x| id_of(x) != id)
        .collect();
    let before = before.min(slots.len());
    slots.insert(before, with_position(&s, position_json(to)));
    with_slots(chain, bumped(slots))
}

pub fn remove_slot(chain: &Value, id: &str) -> Value {
    with_slots(
        chain,
        slots_of(chain)
            .into_iter()
            .filter(|s| id_of(s) != id)
            .collect(),
    )
}

/// Moves a slot up (-1) or down (+1). Unchanged at either end.
pub fn move_slot(chain: &Value, id: &str, by: i64) -> Value {
    let mut slots = slots_of(chain);
    let Some(i) = slots.iter().position(|s| id_of(s) == id) else {
        return chain.clone();
    };
    let j = i as i64 + by;
    if j < 0 || j >= slots.len() as i64 {
        return chain.clone();
    }
    let s = slots.remove(i);
    slots.insert(j as usize, s);
    with_slots(chain, slots)
}

/// Moves a slot to `to` (an index in the current list). Unchanged when it is already there.
pub fn move_slot_to(chain: &Value, id: &str, to: i64) -> Value {
    let mut slots = slots_of(chain);
    let Some(i) = slots.iter().position(|s| id_of(s) == id) else {
        return chain.clone();
    };
    let j = to.min(slots.len() as i64 - 1).max(0) as usize;
    if i == j {
        return chain.clone();
    }
    let s = slots.remove(i);
    slots.insert(j, s);
    with_slots(chain, slots)
}

/// Puts `slot` where the slot `id` is, keeping its place in the list.
pub fn replace_slot(chain: &Value, id: &str, slot: &Value) -> Value {
    with_slots(
        chain,
        slots_of(chain)
            .into_iter()
            .map(|s| if id_of(&s) == id { slot.clone() } else { s })
            .collect(),
    )
}

/// Changes some fields of one slot (`setSlot`).
pub fn set_slot(chain: &Value, id: &str, change: &Map<String, Value>) -> Value {
    let slots = slots_of(chain)
        .into_iter()
        .map(|s| {
            if id_of(&s) != id {
                return s;
            }
            let mut m = s.as_object().cloned().unwrap_or_default();
            for (k, v) in change {
                m.insert(k.clone(), v.clone());
            }
            Value::Object(m)
        })
        .collect();
    with_slots(chain, slots)
}

/// A value held to the control's step and range.
pub fn clamp_param(p: &ParamInfo, value: f64) -> f64 {
    if !value.is_finite() {
        return p.default;
    }
    let snapped = if p.step > 0.0 {
        js_round(value / p.step) * p.step
    } else {
        value
    };
    let trimmed = js_string_to_number(&js_to_fixed(snapped, 6)).unwrap_or(snapped);
    #[allow(clippy::manual_clamp)] // clamp panics when min > max; this follows the interface.
    let held = trimmed.max(p.min).min(p.max);
    held
}

/// Sets one parameter, snapped to its step and range. Clears the preset name.
pub fn set_param(chain: &Value, id: &str, p: &ParamInfo, value: f64) -> Value {
    let v = clamp_param(p, value);
    let slots = slots_of(chain)
        .into_iter()
        .map(|s| {
            if id_of(&s) != id {
                return s;
            }
            let mut m = s.as_object().cloned().unwrap_or_default();
            let mut params = m
                .get("params")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            params.insert(p.key.clone(), json!(v));
            m.insert("params".into(), Value::Object(params));
            m.insert("preset".into(), Value::Null);
            Value::Object(m)
        })
        .collect();
    with_slots(chain, slots)
}

/// The slot's value for a control, or the default when it has none.
pub fn param_value(slot: &Value, p: &ParamInfo) -> f64 {
    match slot
        .get("params")
        .and_then(|m| m.get(&p.key))
        .and_then(Value::as_f64)
    {
        Some(v) if v.is_finite() => clamp_param(p, v),
        _ => p.default,
    }
}

/// Slider position 0..1 for a value, logarithmic where the control asks for it.
pub fn to_slider(p: &ParamInfo, v: f64) -> f64 {
    if p.max <= p.min {
        return 0.0;
    }
    if p.log {
        return (v / p.min).ln() / (p.max / p.min).ln();
    }
    (v - p.min) / (p.max - p.min)
}

pub fn from_slider(p: &ParamInfo, t: f64) -> f64 {
    let x = t.clamp(0.0, 1.0);
    let v = if p.log {
        p.min * (p.max / p.min).powf(x)
    } else {
        p.min + x * (p.max - p.min)
    };
    clamp_param(p, v)
}

/// A value as the control shows it: "-6.0 dB", "4.0:1", "1.2 kHz", "On".
pub fn format_param(p: &ParamInfo, v: f64) -> String {
    if p.toggle {
        return if v >= 0.5 { "On" } else { "Off" }.to_string();
    }
    if !p.choices.is_empty() {
        let i = js_round(v - p.min);
        if i >= 0.0 && (i as usize) < p.choices.len() && i.fract() == 0.0 {
            return p.choices[i as usize].clone();
        }
        return js_number_to_string(v);
    }
    let unit = p.unit.as_deref().unwrap_or("");
    if unit == "Hz" && v >= 1000.0 {
        return format!(
            "{} kHz",
            js_to_fixed(v / 1000.0, if v >= 10_000.0 { 1 } else { 2 })
        );
    }
    let digits = if p.step >= 1.0 {
        0
    } else if p.step >= 0.1 {
        1
    } else {
        2
    };
    let text = js_to_fixed(v, digits);
    if unit.is_empty() {
        text
    } else if unit.starts_with(':') {
        format!("{text}{unit}")
    } else {
        format!("{text} {unit}")
    }
}

// --- Presets ---

/// The built-in presets, the Voice Chain's category colours and blurbs, from `data/voice-presets.json`.
pub fn preset_data() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/voice-presets.json"))
            .expect("voice-presets.json is valid")
    })
}

/// The built-in presets, in order.
pub fn builtin_presets() -> Vec<Value> {
    preset_data()
        .get("builtin")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// True when choosing the preset adds one effect instead of replacing the chain.
pub fn adds_one(p: &Value) -> bool {
    let n = p
        .get("chain")
        .and_then(|c| c.get("slots"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    n == 1 && p.get("group").and_then(Value::as_str) != Some("Voice chains")
}

/// Presets Batch Convert offers: everything but the per-voice track chains.
pub fn batch_presets(list: &[Value]) -> Vec<Value> {
    list.iter()
        .filter(|p| !p.get("trackOnly").is_some_and(crate::mix_routing::truthy))
        .cloned()
        .collect()
}

/// The chain a preset gives, with fresh slot ids. Unknown effects are dropped.
pub fn preset_chain(
    p: &Value,
    known: &HashSet<String>,
    new_id: &mut dyn FnMut() -> String,
) -> Value {
    let chain = p.get("chain").cloned().unwrap_or(Value::Null);
    let mut out = chain.as_object().cloned().unwrap_or_default();
    let name = p.get("name").cloned().unwrap_or(Value::Null);
    let slots: Vec<Value> = chain
        .get("slots")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter(|s| {
            s.get("effect")
                .and_then(Value::as_str)
                .is_some_and(|e| known.contains(e))
        })
        .map(|s| {
            json!({
                "id": new_id(),
                "effect": s.get("effect").cloned().unwrap_or(Value::Null),
                "enabled": true,
                "params": s.get("params").cloned().unwrap_or_else(|| json!({})),
                "preset": name,
            })
        })
        .collect();
    out.insert("slots".into(), Value::Array(slots));
    Value::Object(out)
}

/// What choosing a preset does to the current chain. None when it would overflow.
pub fn apply_preset(
    current: &Value,
    p: &Value,
    known: &HashSet<String>,
    new_id: &mut dyn FnMut() -> String,
) -> Option<Value> {
    let next = preset_chain(p, known, new_id);
    if !adds_one(p) {
        return Some(next);
    }
    match slots_of(&next).first() {
        Some(slot) => add_slot(current, slot, None),
        None => Some(current.clone()),
    }
}

/// A user preset saved from the current chain (slot ids are not kept).
pub fn preset_from(name: &str, chain: &Value) -> Value {
    let mut c = chain.as_object().cloned().unwrap_or_default();
    let slots: Vec<Value> = slots_of(chain)
        .iter()
        .map(|s| json!({ "effect": s.get("effect").cloned().unwrap_or(Value::Null), "params": s.get("params").cloned().unwrap_or(Value::Null) }))
        .collect();
    c.insert("slots".into(), Value::Array(slots));
    json!({ "name": name, "group": "Voice chains", "chain": c })
}

fn finite_or(v: Option<&Value>, d: f64) -> Value {
    match v.and_then(Value::as_f64) {
        Some(x) if x.is_finite() => json!(x),
        _ => json!(d),
    }
}

/// What JavaScript's `{ ...value }` gives for the value of a `params` field.
fn spread_params(v: Option<&Value>) -> Value {
    match v {
        Some(Value::Object(m)) => Value::Object(m.clone()),
        Some(Value::Array(a)) => Value::Object(
            a.iter()
                .enumerate()
                .map(|(i, x)| (i.to_string(), x.clone()))
                .collect(),
        ),
        _ => json!({}),
    }
}

/// Saved user presets, kept only when well formed.
pub fn sanitize_presets(raw: &Value) -> Vec<Value> {
    let Some(list) = raw.as_array() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for r in list {
        let Some(name) = r
            .get("name")
            .and_then(Value::as_str)
            .filter(|n| !is_js_space_only(n))
        else {
            continue;
        };
        let Some(c) = r.get("chain").and_then(Value::as_object) else {
            continue;
        };
        let Some(slots) = c.get("slots").and_then(Value::as_array) else {
            continue;
        };
        let slots: Vec<Value> = slots
            .iter()
            .filter(|s| s.get("effect").and_then(Value::as_str).is_some())
            .take(MAX_SLOTS)
            .map(|s| json!({ "effect": s["effect"], "params": spread_params(s.get("params")) }))
            .collect();
        out.push(json!({
            "name": name,
            "group": "Voice chains",
            "chain": {
                "enabled": c.get("enabled") != Some(&json!(false)),
                "input_gain_db": finite_or(c.get("input_gain_db"), 0.0),
                "output_gain_db": finite_or(c.get("output_gain_db"), 0.0),
                "mix": finite_or(c.get("mix"), 1.0),
                "slots": slots,
            },
        }));
    }
    out
}

fn is_js_space_only(s: &str) -> bool {
    js_trim(s).is_empty() || s.chars().all(is_js_space)
}
