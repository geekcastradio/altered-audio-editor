// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the settings sanitizer in Rust against
//! `shared-fixtures/settings-clean.json`, which the TypeScript test reads too.
//! Each case stores only the settings that differ from the defaults.

use altered_audio_editor_lib::settings_clean::*;
use serde_json::{Map, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/settings-clean.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            (x - y).abs() <= 1e-9 * (1.0 + x.abs().max(y.abs()))
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        _ => a == b,
    }
}

fn diff(full: &Value, def: &Value) -> Value {
    let mut m = Map::new();
    for (k, v) in full.as_object().unwrap() {
        if !def.get(k).is_some_and(|d| same(v, d)) {
            m.insert(k.clone(), v.clone());
        }
    }
    Value::Object(m)
}

#[test]
fn defaults_match() {
    let fx = fixture();
    assert!(
        same(&default_settings(), &fx["defaults"]),
        "defaults differ"
    );
}

#[test]
fn sanitizes_like_typescript() {
    let fx = fixture();
    let def = default_settings();
    for c in fx["cases"].as_array().unwrap() {
        let got = diff(&sanitize_settings(&c["raw"]), &def);
        assert!(
            same(&got, &c["out"]),
            "raw {}\n got {got}\nwant {}",
            c["raw"],
            c["out"]
        );
    }
}

#[test]
fn cleans_bindings_like_typescript() {
    let fx = fixture();
    for c in fx["bindings"].as_array().unwrap() {
        let got = Value::Array(clean_bindings(Some(&c["raw"])));
        assert!(
            same(&got, &c["out"]),
            "bindings\n got {got}\nwant {}",
            c["out"]
        );
    }
}
