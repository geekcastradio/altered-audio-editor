// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks moving, ripple delete and gaps in Rust against
//! `shared-fixtures/clip-edit-2.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::clip_edit::*;
use serde_json::{json, Value};
use std::collections::HashSet;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/clip-edit-2.json");
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

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
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
fn moving() {
    let fx = fixture();
    for c in fx["moves"].as_array().unwrap() {
        let targets: Vec<f64> = c["targets"].as_array().unwrap().iter().map(f).collect();
        let got = move_clips(
            &fx["doc"],
            &strings(&c["ids"]),
            f(&c["delta"]),
            f(&c["rows"]),
            &targets,
            f(&c["tol"]),
        );
        ok(&got.to_json(), &c["out"], &format!("move {c}"));
    }
}

#[test]
fn ripple_deleting() {
    let fx = fixture();
    for c in fx["rdr"].as_array().unwrap() {
        let only: Option<HashSet<String>> = c["only"]
            .as_array()
            .map(|_| strings(&c["only"]).into_iter().collect());
        let got = match ripple_delete_range(
            &fx["doc"],
            &strings(&c["tracks"]),
            f(&c["a"]),
            f(&c["b"]),
            &mut counter(),
            only.as_ref(),
        ) {
            Ok(v) => v.unwrap_or(Value::Null),
            Err(LockedError) => json!({ "locked": true }),
        };
        ok(&got, &c["out"], &format!("ripple range {c}"));
    }
    for c in fx["rdc"].as_array().unwrap() {
        let got = match ripple_delete_clips(&fx["doc"], &strings(&c["ids"])) {
            Ok(v) => v.unwrap_or(Value::Null),
            Err(LockedError) => json!({ "locked": true }),
        };
        ok(&got, &c["out"], &format!("ripple clips {c}"));
    }
}

#[test]
fn gaps() {
    let fx = fixture();
    for c in fx["gaps"].as_array().unwrap() {
        let t = c["track"].as_str().unwrap();
        let at = f(&c["at"]);
        let gap =
            gap_at(&fx["doc"], t, at).map_or(Value::Null, |(s, e)| json!({ "start": s, "end": e }));
        ok(&gap, &c["gap"], &format!("gap {c}"));
        let close = gap_to_close(&fx["doc"], t, at).map_or(Value::Null, |g| g.to_json());
        ok(&close, &c["close"], &format!("close {c}"));
        assert_eq!(ripple_tracks(&fx["doc"], t), strings(&c["rt"]), "{c}");
    }
}
