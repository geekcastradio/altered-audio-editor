// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the panel rules in Rust against `shared-fixtures/panel-rules.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::panel_rules::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/panel-rules.json");
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

fn set(v: &Value) -> BTreeSet<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn eq_nodes_and_switches() {
    let fx = fixture();
    let nodes = eq_nodes(&fx["info"]);
    ok(&json!(nodes), &fx["nodes"], "nodes");
    ok(
        &json!(eq_nodes(&json!({ "id": "x", "name": "x", "params": [] }))),
        &fx["empty"],
        "empty",
    );
    for c in fx["on"].as_array().unwrap() {
        let slot = &fx["slots"][c["si"].as_u64().unwrap() as usize];
        let node = &nodes[c["ni"].as_u64().unwrap() as usize];
        assert_eq!(node_on(slot, node), c["out"].as_bool().unwrap(), "{c}");
    }
}

#[test]
fn clips_and_workspaces() {
    let fx = fixture();
    let ids: Vec<String> = all_midi_clip_ids(&fx["doc"]);
    ok(&json!(ids), &fx["midiIds"], "midi ids");
    for (i, s) in fx["sets"].as_array().unwrap().iter().enumerate() {
        ok(
            &json!(delete_clips_ops(&fx["doc"], &set(s))),
            &fx["del"][i],
            &format!("delete {s}"),
        );
    }
    for c in fx["ws"].as_array().unwrap() {
        let name = fx["names"][c["ni"].as_u64().unwrap() as usize]
            .as_str()
            .unwrap();
        let taken = set(&fx["taken"][c["ti"].as_u64().unwrap() as usize]);
        assert_eq!(
            workspace_id(name, &taken),
            c["out"].as_str().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn panel_sizes() {
    let fx = fixture();
    for (id, v) in fx["mins"].as_object().unwrap() {
        assert_eq!(
            min_size(id),
            Some((v["w"].as_f64().unwrap(), v["h"].as_f64().unwrap())),
            "{id}"
        );
    }
    for c in fx["groups"].as_array().unwrap() {
        let ids: Vec<&str> = c["ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        let (w, h) = group_min(&ids);
        ok(&json!({ "w": w, "h": h }), &c["out"], &format!("group {c}"));
    }
}
