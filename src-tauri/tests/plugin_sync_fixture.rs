// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the plug-in slot bookkeeping and chain drag edits in Rust against
//! `shared-fixtures/plugin-sync.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::plugin_sync::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/plugin-sync.json");
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
fn slots_and_sync() {
    let fx = fixture();
    for c in fx["slots"].as_array().unwrap() {
        let d = &fx["docs"][c["name"].as_str().unwrap()];
        let inst: Vec<Value> = instrument_slots(d)
            .into_iter()
            .map(|(t, s)| json!([t, s]))
            .collect();
        ok(&json!(inst), &c["instruments"], "instruments");
        let all: Vec<Value> = all_slots(d)
            .into_iter()
            .map(|x| json!({ "target": x.target, "chain": x.chain, "slot": x.slot }))
            .collect();
        ok(&json!(all), &c["all"], "all slots");
        ok(&json!(plugin_slots(d)), &c["plugins"], "plugin slots");
    }
    for c in fx["sync"].as_array().unwrap() {
        let d = &fx["docs"][c["name"].as_str().unwrap()];
        let events: Vec<PluginEvent> = c["events"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| PluginEvent {
                slot_id: e["slotId"].as_str().unwrap().to_string(),
                edits: e["edits"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| (p[0].as_str().unwrap().to_string(), p[1].clone()))
                    .collect(),
            })
            .collect();
        let states: Vec<(String, String)> = c["states"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                (
                    p[0].as_str().unwrap().to_string(),
                    p[1].as_str().unwrap().to_string(),
                )
            })
            .collect();
        ok(
            &json!(sync_ops(d, &events, &states)),
            &c["out"],
            &format!("sync {} {}", c["name"], c["i"]),
        );
    }
}

#[test]
fn drag_edits() {
    let fx = fixture();
    for c in fx["owners"].as_array().unwrap() {
        assert_eq!(owner_key(&c["t"]), c["key"].as_str().unwrap());
    }
    for c in fx["parsed"].as_array().unwrap() {
        ok(
            &parse_owner(c["key"].as_str().unwrap()).unwrap_or(Value::Null),
            &c["out"],
            &format!("parse {c}"),
        );
    }
    for c in fx["chains"].as_array().unwrap() {
        let d = &fx["docs"][c["name"].as_str().unwrap()];
        ok(
            &chain_at(d, &c["t"]).unwrap_or(Value::Null),
            &c["out"],
            &format!("chain {} {}", c["name"], c["t"]),
        );
    }
    let doc = &fx["docs"]["full"];
    for c in fx["drops"].as_array().unwrap() {
        let got = drop_edit(
            doc,
            &c["from"],
            &c["slot"],
            c["owner"].as_str().unwrap(),
            c["index"].as_f64().unwrap(),
            "NEW",
        )
        .map_or(
            Value::Null,
            |(label, op)| json!({ "label": label, "op": op }),
        );
        ok(
            &got,
            &c["out"],
            &format!("drop {} {} {}", c["slot"]["id"], c["owner"], c["index"]),
        );
    }
    for c in fx["rows"].as_array().unwrap() {
        ok(
            &row_target(c["id"].as_str().unwrap(), c["bus"].as_bool().unwrap()),
            &c["out"],
            "row",
        );
    }
}
