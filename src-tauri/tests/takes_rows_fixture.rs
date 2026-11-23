// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks take lanes and editor rows in Rust against
//! `shared-fixtures/takes-rows.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::clip_edit::LockedError;
use altered_audio_editor_lib::rows::{lane_rows, move_row_ops};
use altered_audio_editor_lib::takes::*;
use serde_json::{json, Value};
use std::collections::HashSet;

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/takes-rows.json");
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

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
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

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("n{n}0")
    }
}

#[test]
fn takes() {
    let fx = fixture();
    let doc = &fx["doc"];
    ok(&Value::Array(take_sets(doc)), &fx["sets"], "sets");
    for c in fx["nums"].as_array().unwrap() {
        assert_eq!(next_take_number(doc, s(&c["set"])), f(&c["out"]));
    }
    let clips: Vec<&Value> = doc["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["clips"].as_array().unwrap())
        .collect();
    for c in fx["offs"].as_array().unwrap() {
        let clip = clips.iter().find(|k| k["id"] == c["id"]).unwrap();
        assert_eq!(is_off_take(clip), c["out"].as_bool().unwrap(), "{c}");
    }
    for c in fx["makes"].as_array().unwrap() {
        let got = make_takes_ops(doc, &strings(&c["ids"]), &mut || "newset".to_string())
            .unwrap_or(Value::Null);
        ok(&got, &c["out"], &format!("make {}", c["ids"]));
    }
    for c in fx["picks"].as_array().unwrap() {
        let got = match pick_range_ops(
            doc,
            s(&c["set"]),
            f(&c["number"]),
            f(&c["a"]),
            f(&c["b"]),
            &mut counter(),
        ) {
            Ok(ops) => Value::Array(ops),
            Err(LockedError) => json!({ "locked": true }),
        };
        ok(&got, &c["out"], &format!("pick {c}"));
    }
    for c in fx["flat"].as_array().unwrap() {
        ok(
            &Value::Array(flatten_ops(doc, s(&c["set"]))),
            &c["out"],
            "flatten",
        );
    }
    for c in fx["dels"].as_array().unwrap() {
        ok(
            &Value::Array(delete_take_ops(doc, s(&c["set"]), f(&c["number"]))),
            &c["out"],
            "delete take",
        );
    }
    for c in fx["gaps"].as_array().unwrap() {
        let got: Vec<Value> = silent_gaps(doc, s(&c["set"]))
            .into_iter()
            .map(|(a, b)| json!([a, b]))
            .collect();
        ok(&Value::Array(got), &c["out"], "gaps");
    }
}

#[test]
fn rows() {
    let fx = fixture();
    let docs = fx["rdocs"].as_array().unwrap();
    for c in fx["rows"].as_array().unwrap() {
        let listed: HashSet<String> = strings(&c["listed"]).into_iter().collect();
        let got: Vec<Value> = lane_rows(&docs[c["di"].as_u64().unwrap() as usize], &listed)
            .iter()
            .map(|r| r.to_json())
            .collect();
        ok(&Value::Array(got), &c["out"], &format!("rows {c}"));
    }
    for c in fx["moves"].as_array().unwrap() {
        let listed: HashSet<String> = strings(&c["listed"]).into_iter().collect();
        let got = move_row_ops(
            &docs[c["di"].as_u64().unwrap() as usize],
            &listed,
            s(&c["id"]),
            c["before"].as_str(),
        );
        ok(&Value::Array(got), &c["out"], &format!("move row {c}"));
    }
}
