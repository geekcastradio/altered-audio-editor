// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks list selection, folder and solo helpers in Rust against
//! `shared-fixtures/panel-nav.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::panel_nav::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/panel-nav.json");
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

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn selection() {
    let fx = fixture();
    let vis = strs(&fx["visible"]);
    for c in fx["click"].as_array().unwrap() {
        let (sel, anchor) = click_selection(
            &strs(&c["current"]),
            c["anchor"].as_str(),
            c["clicked"].as_str().unwrap(),
            &vis,
            c["ctrl"].as_bool().unwrap(),
            c["shift"].as_bool().unwrap(),
        );
        ok(
            &json!({ "selected": sel, "anchor": anchor }),
            &c["out"],
            &format!("click {c}"),
        );
    }
    for c in fx["step"].as_array().unwrap() {
        let got = step_selection(
            &strs(&c["v"]),
            c["from"].as_str(),
            c["delta"].as_i64().unwrap(),
        )
        .map_or(Value::Null, Value::String);
        ok(&got, &c["out"], &format!("step {c}"));
    }
    for c in fx["paths"].as_array().unwrap() {
        let got = parent_folder(c["p"].as_str().unwrap()).map_or(Value::Null, Value::String);
        ok(&got, &c["out"], &format!("path {c}"));
    }
}

#[test]
fn solo_rules() {
    let fx = fixture();
    let docs = fx["docs"].as_array().unwrap();
    for (d, want) in docs.iter().zip(fx["silent"].as_array().unwrap()) {
        let got: Vec<String> = silent_tracks(Some(d)).into_iter().collect();
        ok(&json!(got), want, "silent");
    }
    assert!(silent_tracks(None).is_empty());
    for c in fx["op"].as_array().unwrap() {
        let got = track_op(
            Some(&docs[c["di"].as_u64().unwrap() as usize]),
            c["id"].as_str().unwrap(),
            &c["changes"],
            c["exclusive"].as_bool().unwrap(),
        );
        ok(
            &got,
            &c["out"],
            &format!("op {} {} {}", c["di"], c["id"], c["changes"]),
        );
    }
    ok(
        &track_op(None, "t1", &json!({ "solo": true }), true),
        &fx["opNull"],
        "opNull",
    );
}

#[test]
fn remix_text() {
    let fx = fixture();
    for c in fx["plan"].as_array().unwrap() {
        let segs: Vec<(f64, f64)> = c["p"]["segments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| (s["start"].as_f64().unwrap(), s["end"].as_f64().unwrap()))
            .collect();
        ok(
            &json!(plan_duration(&segs, c["p"]["stretch"].as_f64().unwrap())),
            &c["out"],
            &format!("plan {c}"),
        );
    }
    for c in fx["fmt"].as_array().unwrap() {
        let s = match &c["s"] {
            Value::String(t) => match t.as_str() {
                "NaN" => f64::NAN,
                _ => f64::INFINITY,
            },
            v => v.as_f64().unwrap(),
        };
        assert_eq!(format_seconds(s), c["out"].as_str().unwrap(), "fmt {c}");
    }
}
