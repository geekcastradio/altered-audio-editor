// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Batch recipes in Rust: every built-in preset of the effects that make sense
//! on a whole file, as a one-effect chain for Batch Convert. Mirrors
//! `src/panels/voice-chain/batch-recipes.ts`; tested against
//! `shared-fixtures/pack-recipes.json`. The single-effect presets are data in
//! `data/effect-presets.json`, kept equal to the TypeScript list by a test.

use crate::voice_chain::builtin_presets;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::OnceLock;

/// The effects offered as recipes with the Batch Convert group each is listed
/// under, in the order Batch Convert lists them.
pub const RECIPE_GROUPS: [(&str, &str); 10] = [
    ("altaudio.dynamics", "Compression"),
    ("altaudio.dynamics_curve", "Compression"),
    ("altaudio.speech_leveler", "Leveling"),
    ("altaudio.limiter", "Limiting"),
    ("altaudio.breath_gate", "Breath gates"),
    ("altaudio.expander", "Breath gates"),
    ("altaudio.noise_reducer", "Noise reduction"),
    ("altaudio.click_repair", "Repair"),
    ("altaudio.precision_filter", "Filters"),
    ("altaudio.parametric_eq", "EQ"),
];

/// Group names the recipes add to the ones the chains already use.
pub const RECIPE_ONLY_GROUPS: [&str; 5] = [
    "Compression",
    "Leveling",
    "Limiting",
    "Noise reduction",
    "Filters",
];

/// The label shown before a name when two presets would otherwise share it.
fn label(effect: &str) -> &str {
    match effect {
        "altaudio.dynamics" => "Dynamics",
        "altaudio.dynamics_curve" => "Curve",
        "altaudio.speech_leveler" => "Leveler",
        "altaudio.limiter" => "Limiter",
        "altaudio.breath_gate" => "Gate",
        "altaudio.expander" => "Expander",
        "altaudio.noise_reducer" => "Noise",
        "altaudio.click_repair" => "Clicks",
        "altaudio.precision_filter" => "Filter",
        "altaudio.parametric_eq" => "EQ",
        other => other,
    }
}

/// The built-in single-effect presets by effect id.
pub fn builtin_effect_presets() -> &'static Value {
    static D: OnceLock<Value> = OnceLock::new();
    D.get_or_init(|| {
        let v: Value = serde_json::from_str(include_str!("../data/effect-presets.json"))
            .expect("effect-presets.json");
        v["builtin"].clone()
    })
}

/// The recipes for the given chains (names taken): a name already taken gets its
/// effect's label in front.
pub fn build_recipes(taken: &[Value]) -> Vec<Value> {
    let mut used: HashSet<String> = taken
        .iter()
        .filter_map(|p| p["name"].as_str().map(String::from))
        .collect();
    let mut out = vec![];
    for (effect, group) in RECIPE_GROUPS {
        for preset in builtin_effect_presets()[effect]
            .as_array()
            .into_iter()
            .flatten()
        {
            let preset_name = preset["name"].as_str().unwrap_or("");
            let mut name = preset_name.to_string();
            if used.contains(&name) {
                name = format!("{}: {}", label(effect), preset_name);
            }
            if used.contains(&name) {
                continue;
            }
            used.insert(name.clone());
            out.push(json!({
                "name": name,
                "group": group,
                "chain": {
                    "enabled": true, "input_gain_db": 0, "output_gain_db": 0, "mix": 1,
                    "slots": [{ "effect": effect, "params": preset["params"].clone() }],
                },
            }));
        }
    }
    out
}

/// Every recipe Batch Convert offers.
pub fn batch_recipes() -> Vec<Value> {
    build_recipes(&builtin_presets())
}
