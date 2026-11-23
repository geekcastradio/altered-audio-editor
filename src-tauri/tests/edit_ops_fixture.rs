// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the clip edit helpers in Rust against `shared-fixtures/edit-ops.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::edit_ops::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/edit-ops.json");
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

fn ids(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

fn set(v: &Value) -> BTreeSet<String> {
    ids(v).into_iter().collect()
}

#[test]
fn silence_and_stack() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["silence"].as_array().unwrap() {
        let (si, ri) = (
            c["si"].as_u64().unwrap() as usize,
            c["ri"].as_u64().unwrap() as usize,
        );
        let r = &fx["ranges"][ri];
        let got = Value::Array(silence_changes(
            doc,
            &set(&fx["sets"][si]),
            r[0].as_f64().unwrap(),
            r[1].as_f64().unwrap(),
        ));
        ok(&got, &c["out"], &format!("silence {si} {ri}"));
    }
    for c in fx["stack"].as_array().unwrap() {
        let si = c["si"].as_u64().unwrap() as usize;
        let got = Value::Array(stack_changes(
            doc,
            &set(&fx["sets"][si]),
            c["move"] == "front",
        ));
        ok(&got, &c["out"], &format!("stack {si} {}", c["move"]));
    }
}

#[test]
fn lookups() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["lookup"].as_array().unwrap() {
        let got = clip_by_id(Some(doc), c["id"].as_str()).map_or(
            Value::Null,
            |(clip, t)| json!({ "clip": clip, "trackId": t }),
        );
        ok(&got, &c["out"], &format!("lookup {}", c["id"]));
    }
    assert!(clip_by_id(None, Some("a")).is_none());
    for c in fx["midi"].as_array().unwrap() {
        ok(
            &Value::Array(find_midi_clips(doc, &set(&c["ids"]))),
            &c["out"],
            &format!("midi {}", c["ids"]),
        );
    }
}

#[test]
fn snapping_and_align() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["snap"].as_array().unwrap() {
        let u = &c["use"];
        let flag = |k: &str, d: bool| u.get(k).and_then(Value::as_bool).unwrap_or(d);
        let got: Vec<Value> = snap_targets(
            doc,
            &set(&c["ids"]),
            c["ph"].as_f64().unwrap(),
            flag("clips", true),
            flag("playhead", true),
            flag("markers", true),
        )
        .into_iter()
        .map(|x| json!(x))
        .collect();
        ok(&Value::Array(got), &c["out"], &format!("snap {c}"));
    }
    for c in fx["pairs"].as_array().unwrap() {
        let got = align_pair(doc, &ids(&c["p"])).map_or(
            Value::Null,
            |(r, o, t)| json!({ "reference": r, "other": o, "trackId": t }),
        );
        ok(&got, &c["out"], &format!("pair {}", c["p"]));
    }
    for c in fx["ops"].as_array().unwrap() {
        let got = align_ops(
            doc,
            c["other"].as_str().unwrap(),
            c["track"].as_str().unwrap(),
            c["lag"].as_f64().unwrap(),
            c["inverted"].as_bool().unwrap(),
        )
        .unwrap_or(Value::Null);
        ok(&got, &c["out"], &format!("ops {c}"));
    }
}
