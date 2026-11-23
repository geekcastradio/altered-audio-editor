// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the controller bindings and feedback frames in Rust against
//! `shared-fixtures/controller-map.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::controller_map::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/controller-map.json");
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

fn ok(a: &Value, b: &Value, what: &str) {
    assert!(same(a, b), "{what}\n got {a}\nwant {b}");
}

#[test]
fn names_and_choices() {
    let fx = fixture();
    for c in fx["sources"].as_array().unwrap() {
        assert_eq!(source_key(&c["s"]), c["key"].as_str().unwrap());
        assert_eq!(source_name(&c["s"]), c["name"].as_str().unwrap());
    }
    for c in fx["targets"].as_array().unwrap() {
        assert_eq!(target_name(&c["t"]), c["name"].as_str().unwrap(), "{c}");
    }
    let actions: Vec<Value> = ACTIONS
        .iter()
        .map(|(id, label)| json!({ "id": id, "label": label }))
        .collect();
    ok(&json!(actions), &fx["actions"], "actions");
    for c in fx["choices"].as_array().unwrap() {
        ok(
            &json!(target_choices(c["n"].as_u64().unwrap() as usize)),
            &c["out"],
            "choices",
        );
    }
    ok(&json!(mackie_preset()), &fx["mackie"], "mackie");
}

#[test]
fn messages() {
    let fx = fixture();
    for c in fx["pan"].as_array().unwrap() {
        ok(
            &json!(pan_at(c["p"].as_f64().unwrap())),
            &c["out"],
            &format!("pan {c}"),
        );
    }
    for c in fx["turn"].as_array().unwrap() {
        ok(&json!(turn(c["v"].as_f64().unwrap())), &c["out"], "turn");
    }
    for c in fx["position"].as_array().unwrap() {
        ok(
            &json!(position(
                c["kind"].as_str().unwrap(),
                c["value"].as_f64().unwrap()
            )),
            &c["out"],
            "position",
        );
    }
    for c in fx["matches"].as_array().unwrap() {
        assert_eq!(
            matches(&c["b"], &c["m"]),
            c["out"].as_bool().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn feedback() {
    let fx = fixture();
    for c in fx["fader14"].as_array().unwrap() {
        ok(
            &json!(fader14(c["p"].as_f64().unwrap())),
            &c["out"],
            "fader14",
        );
    }
    for c in fx["ring"].as_array().unwrap() {
        ok(
            &json!(ring_byte(c["p"].as_f64().unwrap())),
            &c["out"],
            "ring",
        );
    }
    let strips: Vec<Option<Value>> = fx["strips"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (!s.is_null()).then(|| s.clone()))
        .collect();
    for c in fx["frames"].as_array().unwrap() {
        let got = frames(
            c["b"].as_array().unwrap(),
            &strips,
            c["m"].as_f64().unwrap(),
        );
        ok(&json!(got), &c["out"], "frames");
    }
}
