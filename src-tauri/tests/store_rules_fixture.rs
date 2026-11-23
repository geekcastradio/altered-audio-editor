// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the store rules in Rust against `shared-fixtures/store-rules.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::store_rules::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/store-rules.json");
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

fn s(v: &Value) -> &str {
    v.as_str().unwrap()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| s(x).to_string())
        .collect()
}

#[test]
fn paths_tags_names() {
    let fx = fixture();
    for c in fx["abs"].as_array().unwrap() {
        assert_eq!(
            is_absolute_path(s(&c["p"])),
            c["out"].as_bool().unwrap(),
            "abs {c}"
        );
    }
    for c in fx["join"].as_array().unwrap() {
        assert_eq!(
            join_path(s(&c["dir"]), s(&c["rel"])),
            s(&c["out"]),
            "join {c}"
        );
    }
    for c in fx["item"].as_array().unwrap() {
        assert_eq!(
            item_path(s(&c["dir"]), s(&c["file"])),
            s(&c["out"]),
            "item {c}"
        );
    }
    for c in fx["tags"].as_array().unwrap() {
        ok(
            &json!(parse_tags(s(&c["raw"]))),
            &c["out"],
            &format!("tags {c}"),
        );
    }
    for c in fx["retag"].as_array().unwrap() {
        let got = retag(
            &strings(&c["tags"]),
            &strings(&c["add"]),
            &strings(&c["remove"]),
        );
        ok(&json!(got), &c["out"], &format!("retag {c}"));
    }
    for c in fx["folder"].as_array().unwrap() {
        assert_eq!(folder_name(s(&c["n"])), s(&c["out"]), "folder {c}");
    }
    for c in fx["file"].as_array().unwrap() {
        assert_eq!(file_name(s(&c["p"])), s(&c["out"]), "file {c}");
    }
}

#[test]
fn meters() {
    let fx = fixture();
    for run in fx["meter"].as_array().unwrap() {
        let speed = s(&run["speed"]);
        let mut m = MeterLevel {
            level: 0.0,
            peak: 0.0,
            over: false,
        };
        let mut hold = 0.0;
        for (i, step) in run["steps"].as_array().unwrap().iter().enumerate() {
            let (next, h) = fold_meter(speed, m, step["r"].as_f64().unwrap(), hold);
            m = next;
            hold = h;
            let got = json!({ "level": m.level, "peak": m.peak, "over": m.over });
            ok(&got, &step["meter"], &format!("{speed} step {i}"));
            ok(&json!(hold), &step["hold"], &format!("{speed} hold {i}"));
        }
    }
    for c in fx["frac"].as_array().unwrap() {
        ok(
            &json!(meter_fraction(
                c["p"].as_f64().unwrap(),
                c["f"].as_f64().unwrap()
            )),
            &c["out"],
            &format!("frac {c}"),
        );
    }
}

#[test]
fn blank_and_editor_settings() {
    let fx = fixture();
    for c in fx["blank"].as_array().unwrap() {
        assert_eq!(
            is_blank(Some(&c["doc"])),
            c["out"].as_bool().unwrap(),
            "blank {c}"
        );
    }
    for c in fx["editor"].as_array().unwrap() {
        ok(
            &parse_editor_settings(&c["raw"], &fx["base"]),
            &c["out"],
            &format!("editor {c}"),
        );
    }
}
