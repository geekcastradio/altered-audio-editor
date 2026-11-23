// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the stretch edits and Auto Mic Mix helpers in Rust against
//! `shared-fixtures/stretch-mix.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::stretch_mix::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/stretch-mix.json");
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

fn media(v: &Value) -> Option<&Value> {
    (!v.is_null()).then_some(v)
}

#[test]
fn stretch_edits() {
    let fx = fixture();
    for c in fx["stretch"].as_array().unwrap() {
        let got = stretch_to(
            &c["c"],
            media(&c["m"]),
            48000.0,
            c["length"].as_f64().unwrap(),
            c["kp"].as_bool().unwrap(),
        )
        .map_or(
            Value::Null,
            |r| json!({ "change": r.change, "ratio": r.ratio, "limited": r.limited }),
        );
        ok(
            &got,
            &c["out"],
            &format!("stretch {} {} {}", c["name"], c["length"], c["kp"]),
        );
    }
    for c in fx["un"].as_array().unwrap() {
        let got = unstretch(&c["c"], media(&c["m"]), 48000.0).unwrap_or(Value::Null);
        ok(&got, &c["out"], &format!("unstretch {}", c["name"]));
    }
    for c in fx["natural"].as_array().unwrap() {
        ok(
            &json!(natural_length(
                &c["c"],
                Some(&json!({ "sample_rate": 48000 })),
                48000.0
            )),
            &c["out"],
            "natural",
        );
    }
    for c in fx["percent"].as_array().unwrap() {
        assert_eq!(
            percent(c["r"].as_f64().unwrap()),
            c["out"].as_str().unwrap()
        );
    }
    for c in fx["parse"].as_array().unwrap() {
        ok(
            &json!(parse_seconds(c["t"].as_str().unwrap())),
            &c["out"],
            &format!("parse {}", c["t"]),
        );
    }
}

#[test]
fn auto_mic_mix() {
    let fx = fixture();
    ok(&json!(mic_tracks(&fx["doc"])), &fx["mic"], "mic tracks");
    for c in fx["dur"].as_array().unwrap() {
        assert_eq!(
            format_duration(c["s"].as_f64().unwrap()),
            c["out"].as_str().unwrap(),
            "duration {c}"
        );
    }
    for c in fx["mix"].as_array().unwrap() {
        let mut n = 0;
        let mut new_id = || {
            n += 1;
            format!("NEW-{n}")
        };
        ok(
            &mix_ops(&fx["doc"], &c["a"], &mut new_id).unwrap_or(Value::Null),
            &c["ops"],
            &format!("mix {}", c["a"]),
        );
        ok(&json!(describe_talk(&c["a"])), &c["talk"], "talk");
        assert_eq!(describe_mix(&c["a"]), c["line"].as_str().unwrap());
    }
    assert_eq!(REDUCE_CHOICES.len(), 3);
}
