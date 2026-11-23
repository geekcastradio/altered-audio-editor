// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks Close Dead Space and Split Channels in Rust against
//! `shared-fixtures/dead-split.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::clip_edit::{find_clips, LockedError};
use altered_audio_editor_lib::dead_space::*;
use altered_audio_editor_lib::split_channels::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/dead-split.json");
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

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn dead_space() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["dead"].as_array().unwrap() {
        let tracks = strings(&c["tracks"]);
        let (pause, min) = (
            c["pause"].as_f64().unwrap(),
            c["minRemove"].as_f64().unwrap(),
        );
        let spaces: Vec<Value> = dead_spaces(doc, &tracks, pause, min)
            .iter()
            .map(DeadSpace::to_json)
            .collect();
        ok(&Value::Array(spaces), &c["spaces"], &format!("spaces {c}"));
        let closed = match close_dead_space(doc, &tracks, pause, min) {
            Ok(Some(x)) => x.to_json(),
            Ok(None) => Value::Null,
            Err(LockedError) => json!({ "locked": true }),
        };
        ok(&closed, &c["closed"], &format!("closed {c}"));
    }
}

#[test]
fn split_channels() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["split"].as_array().unwrap() {
        let found = find_clips(doc, &strings(&c["ids"]));
        let ids: Vec<String> = splittable(doc, &found)
            .iter()
            .map(|f| f.clip["id"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(ids, strings(&c["splittable"]), "{c}");
        let mut n = 0;
        let got = split_channels_op(doc, &found, &mut || {
            n += 1;
            format!("u{n}")
        })
        .unwrap_or(Value::Null);
        ok(&got, &c["out"], &format!("split {}", c["ids"]));
    }
}
