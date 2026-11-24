// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the effect chain reader in Rust against `shared-fixtures/rack-import.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::rack_import::*;
use serde_json::Value;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/rack-import.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            (x - y).abs() <= 1e-9 * x.abs().max(y.abs()).max(1.0)
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| same(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        _ => a == b,
    }
}

#[test]
fn racks_and_imports_match() {
    let f = fixture();
    let cases = f["cases"].as_array().unwrap();
    assert!(cases.len() >= 20);
    for c in cases {
        let name = c["name"].as_str().unwrap();
        let xml = c["xml"].as_str().unwrap();
        let racks = racks_to_value(&read_racks(xml));
        assert!(
            same(&racks, &c["racks"]),
            "{name}: racks\n got {racks}\nwant {}",
            c["racks"]
        );
        let imported = import_to_value(&import_racks(xml));
        assert!(
            same(&imported, &c["imported"]),
            "{name}: import\n got {imported}\nwant {}",
            c["imported"]
        );
    }
}
