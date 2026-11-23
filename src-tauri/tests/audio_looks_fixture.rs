// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Audio FX looks in Rust against `shared-fixtures/audio-looks.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::audio_looks::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/audio-looks.json");
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
fn effects_and_reading() {
    let fx = fixture();
    let looks: Vec<Value> = LOOKS
        .iter()
        .map(|(id, label)| json!({ "id": id, "label": label }))
        .collect();
    ok(&json!(looks), &fx["looks"], "looks");
    for c in fx["effects"].as_array().unwrap() {
        let got: Vec<Value> =
            look_effects(c["id"].as_str().unwrap(), c["s"].as_u64().unwrap() as u8)
                .into_iter()
                .map(|m| json!({ "effect": m.effect, "params": m.params }))
                .collect();
        ok(&json!(got), &c["out"], &format!("effects {c}"));
    }
    for c in fx["read"].as_array().unwrap() {
        let chain = (!c["c"].is_null()).then_some(&c["c"]);
        ok(&read_looks(chain), &c["out"], &format!("read {c}"));
    }
    for c in fx["panel"].as_array().unwrap() {
        assert_eq!(is_panel_slot(&c["s"]), c["out"].as_bool().unwrap(), "{c}");
    }
}

#[test]
fn writing_and_naming() {
    let fx = fixture();
    for c in fx["with"].as_array().unwrap() {
        let chain = &fx["chains"][c["ci"].as_u64().unwrap() as usize];
        let looks = &fx["sets"][c["li"].as_u64().unwrap() as usize];
        let mut n = 0;
        let mut new_id = || {
            n += 1;
            format!("N{n}")
        };
        let got = with_looks(
            chain,
            looks,
            &mut new_id,
            c["max"].as_u64().unwrap() as usize,
        )
        .unwrap_or(Value::Null);
        ok(
            &got,
            &c["out"],
            &format!("with {} {} {}", c["ci"], c["li"], c["max"]),
        );
    }
    for c in fx["without"].as_array().unwrap() {
        ok(
            &without_looks(&fx["chains"][c["ci"].as_u64().unwrap() as usize]),
            &c["out"],
            "without",
        );
    }
    for c in fx["describe"].as_array().unwrap() {
        assert_eq!(
            describe_looks(&fx["sets"][c["li"].as_u64().unwrap() as usize]),
            c["out"].as_str().unwrap()
        );
    }
    for c in fx["same"].as_array().unwrap() {
        let (a, b) = (
            &fx["sets"][c["ai"].as_u64().unwrap() as usize],
            &fx["sets"][c["bi"].as_u64().unwrap() as usize],
        );
        assert_eq!(same_looks(a, b), c["out"].as_bool().unwrap());
    }
}
