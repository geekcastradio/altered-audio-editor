// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the view helpers in Rust against `shared-fixtures/view-helpers.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::view_helpers::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/view-helpers.json");
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

fn f(v: &Value, k: &str) -> f64 {
    v[k].as_f64().unwrap()
}

fn bools(v: &Value) -> Vec<bool> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|b| b.as_bool().unwrap())
        .collect()
}

fn take(t: Option<ChannelTake>) -> Value {
    t.map_or(Value::Null, |t| {
        Value::Array(
            t.into_iter()
                .map(|(after, channel)| json!({ "after": after, "channel": channel }))
                .collect(),
        )
    })
}

#[test]
fn reveal_and_tabs() {
    let fx = fixture();
    for c in fx["reveal"].as_array().unwrap() {
        let got = reveal_scroll(f(c, "scroll"), f(c, "span"), f(c, "to"), f(c, "at"))
            .map_or(Value::Null, |x| json!(x));
        ok(&got, &c["out"], &format!("reveal {c}"));
    }
    for (rows, c) in fx["rows"]
        .as_array()
        .unwrap()
        .iter()
        .zip(fx["tabs"].as_array().unwrap())
    {
        let rows = rows.as_array().unwrap();
        let ids: Vec<String> = rows
            .iter()
            .map(|r| r["id"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(show_tabs(rows.len()), c["show"].as_bool().unwrap());
        let nb: Vec<Value> = ["a", "b", "c", "zz"]
            .iter()
            .map(|id| neighbor_of(&ids, id).map_or(Value::Null, Value::String))
            .collect();
        ok(&Value::Array(nb), &c["nb"], "neighbors");
        let labels: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!(tab_label(
                    r["name"].as_str().unwrap(),
                    r["dirty"].as_bool().unwrap()
                ))
            })
            .collect();
        ok(&Value::Array(labels), &c["labels"], "labels");
    }
}

#[test]
fn video_bar() {
    let fx = fixture();
    for c in fx["bars"].as_array().unwrap() {
        let got = lane_bar(
            f(c, "offset"),
            f(c, "duration"),
            f(c, "spp"),
            f(c, "scroll"),
            f(c, "rate"),
            f(c, "w"),
        )
        .map_or(
            Value::Null,
            |(left, width)| json!({ "left": left, "width": width }),
        );
        ok(&got, &c["out"], &format!("bar {c}"));
    }
    for c in fx["drag"].as_array().unwrap() {
        ok(
            &json!(dragged_offset(f(c, "a"), f(c, "b"), f(c, "c"), f(c, "d"))),
            &c["out"],
            &format!("drag {c}"),
        );
    }
    for c in fx["base"].as_array().unwrap() {
        assert_eq!(
            base_name(c["p"].as_str().unwrap()),
            c["out"].as_str().unwrap()
        );
    }
}

#[test]
fn channels() {
    let fx = fixture();
    for e in fx["ch"].as_array().unwrap() {
        let c = bools(&e["c"]);
        ok(&json!(all_on(c.len())), &e["allOn"], "allOn");
        ok(
            &one_channel(&c).map_or(Value::Null, |x| json!(x)),
            &e["one"],
            &format!("one {c:?}"),
        );
        assert_eq!(some_off(&c), e["some"].as_bool().unwrap());
        ok(&take(edit_take(&c)), &e["edit"], "edit");
        assert_eq!(
            channel_label(&c),
            e["label"].as_str().unwrap(),
            "label {c:?}"
        );
        let tg: Vec<Vec<bool>> = (0..c.len()).map(|k| toggle_channel(&c, k)).collect();
        ok(&json!(tg), &e["toggles"], &format!("toggle {c:?}"));
        let so: Vec<Vec<bool>> = (0..c.len()).map(|k| solo_channel(&c, k)).collect();
        ok(&json!(so), &e["solos"], &format!("solo {c:?}"));
        let ps: Vec<Value> = [None, Some(0), Some(1), Some(2)]
            .into_iter()
            .map(|from| take(paste_take(&c, from)))
            .collect();
        ok(&Value::Array(ps), &e["pastes"], &format!("paste {c:?}"));
    }
    ok(&json!(all_on(0)), &fx["allOn0"], "allOn 0");
}
