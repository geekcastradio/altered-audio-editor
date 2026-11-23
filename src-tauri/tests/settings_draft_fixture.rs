// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the settings draft cleanup and size text in Rust against
//! `shared-fixtures/settings-draft.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::settings_draft::*;
use serde_json::Value;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/settings-draft.json");
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

#[test]
fn drafts() {
    let fx = fixture();
    let list = fx["drafts"].as_array().unwrap();
    assert!(list.len() > 50);
    for c in list {
        let got = clean_draft(&c["in"]);
        assert!(
            same(&got, &c["out"]),
            "case {}\n got {got}\nwant {}",
            c["in"],
            c["out"]
        );
    }
}

#[test]
fn sizes() {
    let fx = fixture();
    for c in fx["sz"].as_array().unwrap() {
        let got = size_text(c["b"].as_f64().unwrap());
        assert_eq!(got, c["out"].as_str().unwrap(), "bytes {}", c["b"]);
    }
}
