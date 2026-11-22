// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust effect preset rules against
//! `shared-fixtures/effect-presets.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::effect_presets::*;
use altered_audio_editor_lib::effect_ui::Param;
use serde_json::Value;
use std::collections::BTreeMap;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/effect-presets.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn near(a: f64, b: f64, what: &str) {
    let tol = 1e-9 * b.abs().max(1.0);
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b}");
}

fn info(v: &Value) -> Vec<Param> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| Param {
            key: p["key"].as_str().unwrap().to_string(),
            name: p["name"].as_str().unwrap().to_string(),
            unit: p["unit"].as_str().unwrap().to_string(),
            min: f(&p["min"]),
            max: f(&p["max"]),
            default: f(&p["default"]),
            step: f(&p["step"]),
            toggle: p["toggle"].as_bool().unwrap(),
            group: None,
            hidden: false,
        })
        .collect()
}

fn numbers(v: &Value) -> BTreeMap<String, f64> {
    v.as_object()
        .unwrap()
        .iter()
        .map(|(k, n)| (k.clone(), f(n)))
        .collect()
}

fn preset(v: &Value) -> EffectPreset {
    EffectPreset {
        name: v["name"].as_str().unwrap().to_string(),
        params: numbers(&v["params"]),
        data: v["data"].as_str().map(str::to_string),
    }
}

fn same_preset(got: &EffectPreset, want: &Value, what: &str) {
    assert_eq!(got.name, want["name"].as_str().unwrap(), "{what}");
    let w = numbers(&want["params"]);
    assert_eq!(got.params.len(), w.len(), "{what}: count");
    for (k, v) in &w {
        near(*got.params.get(k).unwrap(), *v, what);
    }
    assert_eq!(got.data.as_deref(), want["data"].as_str(), "{what}: data");
}

fn same_bank(got: &Bank, want: &Value, what: &str) {
    let want = want.as_object().unwrap();
    assert_eq!(
        got.keys().collect::<Vec<_>>(),
        want.keys().collect::<Vec<_>>(),
        "{what}: effects"
    );
    for (effect, list) in want {
        let g = &got[effect];
        let w = list.as_array().unwrap();
        assert_eq!(g.len(), w.len(), "{what}: {effect} count");
        for (gp, wp) in g.iter().zip(w) {
            same_preset(gp, wp, what);
        }
    }
}

#[test]
fn presets_from_slots() {
    let fx = fixture();
    let ps = info(&fx["info"]);
    for c in fx["from"].as_array().unwrap() {
        let (got, left_out) = effect_preset_from(
            c["name"].as_str().unwrap(),
            &numbers(&c["params"]),
            c["data"].as_str(),
            &ps,
        );
        same_preset(&got, &c["out"]["preset"], c["name"].as_str().unwrap());
        assert_eq!(left_out, c["out"]["dataLeftOut"].as_bool().unwrap());
    }
}

#[test]
fn the_bank() {
    let fx = fixture();
    let mut bank = Bank::new();
    for (i, s) in fx["bank"].as_array().unwrap().iter().enumerate() {
        let effect = s["effect"].as_str().unwrap();
        bank = match s["op"].as_str().unwrap() {
            "with" => bank_with(&bank, effect, preset(&s["preset"])),
            _ => bank_without(&bank, effect, s["name"].as_str().unwrap()),
        };
        // The last step follows 64 more saves than the steps kept before it.
        if i < 11 {
            same_bank(&bank, &s["out"], &format!("bank step {i}"));
        }
    }
    // Continue with the 64 saves the fixture keeps only the end of.
    let mut long = bank.clone();
    for i in 0..64 {
        let p = EffectPreset {
            name: format!("Preset {}", (i * 7) % 64),
            params: BTreeMap::from([("a".to_string(), i as f64)]),
            data: None,
        };
        long = bank_with(&long, "m", p);
    }
    let last = fx["bank"].as_array().unwrap().last().unwrap();
    same_bank(&long, &last["out"], "long run");
}

#[test]
fn applying_a_preset() {
    let fx = fixture();
    let ps = info(&fx["info"]);
    for (i, c) in fx["apply"].as_array().unwrap().iter().enumerate() {
        let slot = &c["slot"];
        let (params, name, data) = apply_effect_preset(
            &numbers(&slot["params"]),
            slot["data"].as_str(),
            &preset(&c["preset"]),
            &ps,
        );
        let want = &c["out"];
        let w = numbers(&want["params"]);
        assert_eq!(params.len(), w.len(), "apply {i}");
        for (k, v) in &w {
            near(params[k], *v, &format!("apply {i}"));
        }
        assert_eq!(name, want["preset"].as_str().unwrap());
        assert_eq!(data.as_deref(), want["data"].as_str(), "apply {i}: data");
    }
}

#[test]
fn reading_a_bank_back() {
    let fx = fixture();
    for (i, c) in fx["sanitize"].as_array().unwrap().iter().enumerate() {
        same_bank(
            &sanitize_bank(&c["raw"]),
            &c["out"],
            &format!("sanitize {i}"),
        );
    }
}
