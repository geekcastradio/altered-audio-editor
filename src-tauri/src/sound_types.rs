// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Sound Types in Rust: the plain Voice Chain effects a type, its ticked jobs
//! and an Amount make, and reading a setup back from a chain. Mirrors
//! `src/panels/audio-fx/soundtypes.ts`; tested against
//! `shared-fixtures/sound-host.json`. The type list is data in
//! `data/sound-types.json`, kept equal to the TypeScript list by a test.

use crate::js_compat::js_round;
use serde_json::{json, Map, Value};
use std::sync::OnceLock;

/// What a slot's preset name starts with when this panel made it.
pub const TAG: &str = "Sound Types: ";
const LOOK_TAG: &str = "Audio FX: ";

pub fn types() -> &'static Vec<Value> {
    static D: OnceLock<Vec<Value>> = OnceLock::new();
    D.get_or_init(|| {
        let v: Value = serde_json::from_str(include_str!("../data/sound-types.json"))
            .expect("sound-types.json");
        v["types"].as_array().cloned().unwrap_or_default()
    })
}

pub fn type_info(id: &str) -> Option<&'static Value> {
    types().iter().find(|t| t["id"] == id)
}

fn preset_of(s: &Value) -> Option<&str> {
    s.get("preset").and_then(Value::as_str)
}

pub fn is_sound_slot(s: &Value) -> bool {
    preset_of(s).is_some_and(|p| p.starts_with(TAG))
}

fn is_look_slot(s: &Value) -> bool {
    preset_of(s).is_some_and(|p| p.starts_with(LOOK_TAG))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Setup {
    pub kind: String,
    pub sections: Vec<String>,
    pub preset: String,
    pub amount: f64,
}

#[derive(Clone, Debug)]
pub struct Made {
    pub effect: String,
    pub params: Map<String, Value>,
}

pub(crate) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// `Math.round(n * 10 ** d) / 10 ** d`.
fn round(n: f64, d: i32) -> f64 {
    let m = 10f64.powi(d);
    js_round(n * m) / m
}

pub(crate) fn made(effect: &str, params: Value) -> Made {
    Made {
        effect: format!("altaudio.{effect}"),
        params: params.as_object().cloned().unwrap_or_default(),
    }
}

pub(crate) fn voice_eq(b3_gain: Option<f64>) -> Made {
    made(
        "parametric_eq",
        json!({
            "hp_on": 1, "hp_freq": 80, "hp_slope": 1,
            "b1_freq": 300, "b1_gain": -2, "b1_q": 1.2,
            "b3_freq": 4000, "b3_gain": b3_gain.unwrap_or(2.0), "b3_q": 0.9,
        }),
    )
}

pub(crate) fn leveler() -> Made {
    made(
        "speech_leveler",
        json!({ "target": -20, "max_gain": 8, "max_cut": 8, "speed": 2000, "floor": -50 }),
    )
}

pub(crate) fn comp(threshold: f64, ratio: f64, makeup: f64) -> Made {
    made(
        "dynamics",
        json!({ "threshold": threshold, "ratio": ratio, "attack": 8, "release": 160, "knee": 6, "makeup": makeup }),
    )
}

pub(crate) fn limit(ceiling: f64) -> Made {
    made(
        "limiter",
        json!({ "ceiling": ceiling, "lookahead": 5, "release": 100 }),
    )
}

pub(crate) fn clean_noise(reduction: f64) -> Made {
    made(
        "noise_reducer",
        json!({ "reduction": reduction, "strength": 2, "learn": 2, "smoothing": 50 }),
    )
}

/// The plain effects one section makes at an Amount of 0 to 100.
pub fn section_effects(kind: &str, section: &str, amount: f64) -> Vec<Made> {
    // Math.min(100, Math.max(0, amount)): NaN stays NaN.
    let a = if amount.is_nan() {
        amount
    } else {
        amount.clamp(0.0, 100.0)
    } / 100.0;
    match format!("{kind}/{section}").as_str() {
        "dialogue/repair" => vec![clean_noise(round(lerp(4.0, 14.0, a), 0))],
        "dialogue/level" => vec![
            leveler(),
            comp(
                round(lerp(-16.0, -26.0, a), 0),
                round(lerp(2.0, 4.0, a), 1),
                round(lerp(2.0, 6.0, a), 0),
            ),
        ],
        "dialogue/clarity" => vec![
            voice_eq(Some(round(lerp(1.0, 3.5, a), 1))),
            made(
                "deesser",
                json!({ "freq": 6500, "threshold": round(lerp(-28.0, -34.0, a), 0), "reduction": round(lerp(4.0, 8.0, a), 0) }),
            ),
        ],
        "music/tone" => vec![made(
            "parametric_eq",
            json!({ "hp_on": 1, "hp_freq": 35, "hp_slope": 1, "b1_freq": 300, "b1_gain": -round(lerp(0.5, 2.5, a), 1), "b1_q": 1 }),
        )],
        "music/room" => vec![made(
            "parametric_eq",
            json!({ "b2_freq": 1800, "b2_gain": -round(lerp(1.0, 5.0, a), 1), "b2_q": 0.7 }),
        )],
        "music/glue" => vec![comp(
            round(lerp(-12.0, -22.0, a), 0),
            round(lerp(1.5, 2.5, a), 1),
            round(lerp(0.0, 3.0, a), 0),
        )],
        "sfx/tidy" => vec![made(
            "parametric_eq",
            json!({ "hp_on": 1, "hp_freq": round(lerp(40.0, 90.0, a), 0), "hp_slope": 1 }),
        )],
        "sfx/punch" => vec![comp(
            round(lerp(-14.0, -24.0, a), 0),
            round(lerp(2.0, 4.0, a), 1),
            round(lerp(1.0, 4.0, a), 0),
        )],
        "ambience/hiss" => vec![clean_noise(round(lerp(3.0, 10.0, a), 0))],
        "ambience/soften" => vec![made(
            "parametric_eq",
            json!({
                "hp_on": 1, "hp_freq": round(lerp(60.0, 140.0, a), 0), "hp_slope": 1,
                "lp_on": 1, "lp_freq": round(lerp(14000.0, 8000.0, a), 0), "lp_slope": 1,
            }),
        )],
        "ambience/level" => vec![comp(
            round(lerp(-18.0, -28.0, a), 0),
            round(lerp(1.5, 3.0, a), 1),
            0.0,
        )],
        // Every type ends in Limit, with a ceiling that gets closer to 0 as Amount rises.
        _ if section == "limit" => vec![limit(round(lerp(-6.0, -1.5, a), 1))],
        _ => vec![],
    }
}

fn tag_of(kind: &str, section: &str, preset: &str, amount: f64) -> String {
    format!(
        "{TAG}{kind}/{section}/{preset}/{}",
        crate::js_compat::js_number_to_string(js_round(amount))
    )
}

/// The slots a setup would make, in chain order.
pub fn make_slots(setup: &Setup, new_id: &mut dyn FnMut() -> String) -> Vec<Value> {
    let Some(info) = type_info(&setup.kind) else {
        return vec![];
    };
    let mut out = vec![];
    for sec in info["sections"].as_array().into_iter().flatten() {
        let sid = sec["id"].as_str().unwrap_or("");
        if !setup.sections.iter().any(|s| s == sid) {
            continue;
        }
        for e in section_effects(&setup.kind, sid, setup.amount) {
            out.push(json!({
                "id": new_id(),
                "effect": e.effect,
                "enabled": true,
                "params": e.params,
                "preset": tag_of(&setup.kind, sid, &setup.preset, setup.amount),
            }));
        }
    }
    out
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// `/^Sound Types: (\w+)\/(\w+)\/([\w-]*)\/(\d+)$/`.
fn parse_tag(s: &str) -> Option<(&str, &str, &str, &str)> {
    let rest = s.strip_prefix(TAG)?;
    let mut parts = rest.split('/');
    let (kind, section, preset, amount) =
        (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    let word = |t: &str| !t.is_empty() && t.chars().all(is_word);
    let ok = word(kind)
        && word(section)
        && preset.chars().all(|c| is_word(c) || c == '-')
        && !amount.is_empty()
        && amount.chars().all(|c| c.is_ascii_digit());
    ok.then_some((kind, section, preset, amount))
}

fn slots_of(chain: &Value) -> Vec<Value> {
    chain
        .get("slots")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// The setup recorded on the chain's tagged slots, or None when there are none.
pub fn read_setup(chain: Option<&Value>) -> Option<Setup> {
    let mut kind: Option<String> = None;
    let mut preset = String::new();
    let mut amount = 50.0;
    let mut sections: Vec<String> = vec![];
    for s in chain
        .map(slots_of)
        .unwrap_or_default()
        .iter()
        .filter(|s| is_sound_slot(s))
    {
        let Some((k, sec, p, a)) = preset_of(s).and_then(parse_tag) else {
            continue;
        };
        if type_info(k).is_none() {
            continue;
        }
        kind = Some(k.to_string());
        if !sections.iter().any(|x| x == sec) {
            sections.push(sec.to_string());
        }
        preset = p.to_string();
        amount = a.parse().unwrap_or(f64::NAN);
    }
    kind.map(|kind| Setup {
        kind,
        sections,
        preset,
        amount,
    })
}

fn num(v: Option<&Value>) -> f64 {
    v.and_then(Value::as_f64).unwrap_or(0.0)
}

fn same_params(a: &Value, b: &Value) -> bool {
    let empty = Map::new();
    let (a, b) = (
        a.as_object().unwrap_or(&empty),
        b.as_object().unwrap_or(&empty),
    );
    let keys: Vec<&String> = a.keys().chain(b.keys()).collect();
    !keys
        .iter()
        .any(|k| (num(a.get(*k)) - num(b.get(*k))).abs() > 1e-6)
}

/// True when the tagged effects no longer match what the setup would make.
pub fn is_custom(chain: Option<&Value>, setup: Option<&Setup>) -> bool {
    let Some(setup) = setup else {
        return false;
    };
    let have: Vec<Value> = chain
        .map(slots_of)
        .unwrap_or_default()
        .into_iter()
        .filter(is_sound_slot)
        .collect();
    let want = make_slots(setup, &mut String::new);
    if have.len() != want.len() {
        return true;
    }
    have.iter().zip(&want).any(|(h, w)| {
        h["effect"] != w["effect"]
            || h.get("enabled")
                .is_none_or(|e| !crate::mix_routing::truthy(e))
            || !same_params(&h["params"], &w["params"])
    })
}

/// The chain with this panel's slots (and the Audio FX looks) replaced by the
/// setup, placed first. None when the result would exceed `max` effects.
pub fn with_setup(
    chain: &Value,
    setup: &Setup,
    new_id: &mut dyn FnMut() -> String,
    max: usize,
) -> Option<Value> {
    let others: Vec<Value> = slots_of(chain)
        .into_iter()
        .filter(|s| !is_sound_slot(s) && !is_look_slot(s))
        .collect();
    let mut slots = make_slots(setup, new_id);
    if others.len() + slots.len() > max {
        return None;
    }
    slots.extend(others);
    let mut out = chain.as_object().cloned().unwrap_or_default();
    out.insert("slots".into(), Value::Array(slots));
    Some(Value::Object(out))
}

pub fn without_setup(chain: &Value) -> Value {
    let mut out = chain.as_object().cloned().unwrap_or_default();
    let slots: Vec<Value> = slots_of(chain)
        .into_iter()
        .filter(|s| !is_sound_slot(s))
        .collect();
    out.insert("slots".into(), Value::Array(slots));
    Value::Object(out)
}

pub fn preset_setup(kind: &str, preset_id: &str) -> Option<Setup> {
    let info = type_info(kind)?;
    let presets = info["presets"].as_array()?;
    let p = presets
        .iter()
        .find(|x| x["id"] == preset_id)
        .or(presets.first())?;
    Some(Setup {
        kind: kind.to_string(),
        sections: p["sections"]
            .as_array()?
            .iter()
            .filter_map(|s| s.as_str().map(String::from))
            .collect(),
        preset: p["id"].as_str()?.to_string(),
        amount: p["amount"].as_f64()?,
    })
}
