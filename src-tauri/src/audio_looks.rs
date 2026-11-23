// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Audio FX looks in Rust: four guided looks (Restoration, Level, Clarity,
//! Polish), each Off, Light, Medium or Strong, written as plain Voice Chain
//! effects at the front of a chain and tagged so they can be read back and
//! replaced. Mirrors `src/panels/audio-fx/audiofx.ts`, tested against
//! `shared-fixtures/audio-looks.json`. The effect blocks are the ones
//! `sound_types.rs` already builds.

use crate::sound_types::{clean_noise, comp, is_sound_slot, leveler, limit, made, voice_eq, Made};
use serde_json::{json, Value};

/// The looks in chain order: (id, label).
pub const LOOKS: [(&str, &str); 4] = [
    ("restoration", "Restoration"),
    ("level", "Level"),
    ("clarity", "Clarity"),
    ("polish", "Polish"),
];

/// What a slot's preset name starts with when this panel made it.
pub const TAG: &str = "Audio FX: ";

fn tag_of(id: &str, s: u8) -> String {
    format!("{TAG}{id}/{s}")
}

/// Whether the panel made this slot.
pub fn is_panel_slot(s: &Value) -> bool {
    s.get("preset")
        .and_then(Value::as_str)
        .is_some_and(|p| p.starts_with(TAG))
}

fn pick<T>(s: u8, light: T, medium: T, strong: T) -> T {
    match s {
        1 => light,
        2 => medium,
        _ => strong,
    }
}

/// The plain effects one look makes at one strength (1 to 3). Empty for an unknown look.
pub fn look_effects(id: &str, s: u8) -> Vec<Made> {
    match id {
        "restoration" => vec![clean_noise(pick(s, 6.0, 10.0, 14.0))],
        "level" => vec![
            leveler(),
            pick(
                s,
                comp(-18.0, 2.0, 2.0),
                comp(-22.0, 3.0, 4.0),
                comp(-26.0, 4.0, 6.0),
            ),
        ],
        "clarity" => {
            let mut eq = voice_eq(None);
            eq.params
                .insert("b3_gain".into(), json!(pick(s, 1.0, 2.0, 3.5)));
            vec![
                eq,
                made(
                    "deesser",
                    json!({ "freq": 6500, "threshold": pick(s, -28.0, -32.0, -34.0), "reduction": pick(s, 4.0, 6.0, 8.0) }),
                ),
            ]
        }
        "polish" => vec![limit(pick(s, -6.0, -3.0, -1.5))],
        _ => vec![],
    }
}

/// Which looks the chain already carries, read back from its tagged slots:
/// `{ restoration, level, clarity, polish }`, each 0 to 3.
pub fn read_looks(chain: Option<&Value>) -> Value {
    let mut out = json!({ "restoration": 0, "level": 0, "clarity": 0, "polish": 0 });
    for s in chain
        .and_then(|c| c.get("slots"))
        .and_then(Value::as_array)
        .map_or(&[][..], |a| a.as_slice())
    {
        if !is_panel_slot(s) {
            continue;
        }
        let rest = &s["preset"].as_str().unwrap_or("")[TAG.len()..];
        let Some((id, strength)) = rest.split_once('/') else {
            continue;
        };
        let word = !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if word && matches!(strength, "1" | "2" | "3") && out.get(id).is_some() {
            out[id] = json!(strength.parse::<u8>().unwrap_or(0));
        }
    }
    out
}

/// The chain with the panel's slots replaced by the chosen looks, placed
/// first; every other effect stays where it was, after them. None when the
/// result would hold more than `max` effects.
pub fn with_looks(
    chain: &Value,
    looks: &Value,
    new_id: &mut dyn FnMut() -> String,
    max: usize,
) -> Option<Value> {
    let slots = chain
        .get("slots")
        .and_then(Value::as_array)
        .map_or(&[][..], |a| a.as_slice());
    let others: Vec<Value> = slots
        .iter()
        .filter(|s| !is_panel_slot(s) && !is_sound_slot(s))
        .cloned()
        .collect();
    let mut made_slots = Vec::new();
    for (id, _) in LOOKS {
        let s = looks.get(id).and_then(Value::as_u64).unwrap_or(0) as u8;
        if s == 0 {
            continue;
        }
        for e in look_effects(id, s) {
            made_slots.push(json!({ "id": new_id(), "effect": e.effect, "enabled": true, "params": e.params, "preset": tag_of(id, s) }));
        }
    }
    if others.len() + made_slots.len() > max {
        return None;
    }
    let mut next = chain.clone();
    made_slots.extend(others);
    next["slots"] = json!(made_slots);
    Some(next)
}

/// The chain without the panel's slots.
pub fn without_looks(chain: &Value) -> Value {
    let mut next = chain.clone();
    let slots: Vec<Value> = chain
        .get("slots")
        .and_then(Value::as_array)
        .map_or(&[][..], |a| a.as_slice())
        .iter()
        .filter(|s| !is_panel_slot(s))
        .cloned()
        .collect();
    next["slots"] = json!(slots);
    next
}

/// The name of the History entry: "Dialogue: Restoration, Clarity".
pub fn describe_looks(looks: &Value) -> String {
    let on: Vec<&str> = LOOKS
        .iter()
        .filter(|(id, _)| looks.get(*id).and_then(Value::as_u64).unwrap_or(0) != 0)
        .map(|(_, label)| *label)
        .collect();
    if on.is_empty() {
        "Dialogue: Off".to_string()
    } else {
        format!("Dialogue: {}", on.join(", "))
    }
}

/// Whether two sets of looks are the same.
pub fn same_looks(a: &Value, b: &Value) -> bool {
    LOOKS.iter().all(|(id, _)| a.get(*id) == b.get(*id))
}
