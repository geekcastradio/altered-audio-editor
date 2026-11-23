// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks clip editing in Rust against `shared-fixtures/clip-edit.json`, which
//! the TypeScript test reads too.

use altered_audio_editor_lib::clip_edit::*;
use serde_json::{json, Value};
use std::collections::HashSet;

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/clip-edit.json");
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

fn clip<'a>(doc: &'a Value, id: &str) -> &'a Value {
    doc["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["clips"].as_array().unwrap())
        .find(|c| c["id"] == id)
        .unwrap()
}

fn opt(v: Option<Value>) -> Value {
    v.unwrap_or(Value::Null)
}

#[test]
fn ends_and_ids() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["ends"].as_array().unwrap() {
        assert_eq!(
            end_of(doc, clip(doc, c["id"].as_str().unwrap())),
            f(&c["out"]),
            "{c}"
        );
    }
    for c in fx["envIds"].as_array().unwrap() {
        assert_eq!(
            envelope_id(c["id"].as_str().unwrap(), c["kind"].as_str().unwrap()),
            c["out"].as_str().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn splitting_and_pieces() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["split"]
        .as_array()
        .unwrap()
        .iter()
        .chain(fx["splitAll"].as_array().unwrap())
    {
        let got = split_clips(doc, &strings(&c["ids"]), f(&c["at"]), &mut counter());
        ok(
            &opt(got),
            &c["out"],
            &format!("split {} {}", c["ids"], c["at"]),
        );
    }
    for c in fx["pieces"].as_array().unwrap() {
        let k = clip(doc, c["id"].as_str().unwrap());
        let m = doc["media"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["id"] == k["media_id"]);
        let got = piece_of(
            k,
            m,
            48000.0,
            f(&c["from"]),
            f(&c["to"]),
            f(&c["start"]),
            "p1",
        );
        ok(&got, &c["out"], &format!("piece {c}"));
    }
    for c in fx["isolates"].as_array().unwrap() {
        let got = match isolate(
            doc,
            &strings(&c["tracks"]),
            f(&c["a"]),
            f(&c["b"]),
            &mut counter(),
        ) {
            Ok(i) => json!({ "ops": i.ops, "inside": i.inside }),
            Err(LockedError) => json!({ "locked": true }),
        };
        ok(&got, &c["out"], &format!("isolate {c}"));
    }
}

#[test]
fn trimming_and_fades() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["trims"].as_array().unwrap() {
        let targets: Vec<f64> = c["targets"].as_array().unwrap().iter().map(f).collect();
        let got = trim_clip(
            doc,
            clip(doc, c["id"].as_str().unwrap()),
            c["edge"] == "left",
            f(&c["delta"]),
            &targets,
            f(&c["tol"]),
        );
        ok(&opt(got), &c["out"], &format!("trim {c}"));
    }
    let shape = fx["fadeShape"].as_str().unwrap();
    for c in fx["fades"].as_array().unwrap() {
        let got = fade_clip(
            doc,
            clip(doc, c["id"].as_str().unwrap()),
            c["which"] == "in",
            f(&c["length"]),
            shape,
        );
        ok(&opt(got), &c["out"], &format!("fade {c}"));
    }
    for c in fx["loops"].as_array().unwrap() {
        let (changes, on) = loop_changes(doc, &strings(&c["ids"]));
        ok(
            &json!({ "changes": changes, "on": on }),
            &c["out"],
            &format!("loop {}", c["ids"]),
        );
    }
    for c in fx["ttp"].as_array().unwrap() {
        let got = trim_to_playhead(
            doc,
            &strings(&json!([c["id"]])),
            f(&c["at"]),
            c["edge"] == "start",
        );
        ok(
            &Value::Array(got),
            &c["out"],
            &format!("trim to playhead {c}"),
        );
    }
    for c in fx["ttr"].as_array().unwrap() {
        let got = trim_to_range(doc, &strings(&c["ids"]), f(&c["a"]), f(&c["b"]));
        ok(&Value::Array(got), &c["out"], &format!("trim to range {c}"));
    }
}

#[test]
fn clips_under_a_point() {
    let fx = fixture();
    for c in fx["clipsAt"].as_array().unwrap() {
        let pref: HashSet<String> = strings(&c["pref"]).into_iter().collect();
        assert_eq!(
            clips_at(&fx["doc"], f(&c["at"]), &pref),
            strings(&c["out"]),
            "{c}"
        );
    }
}
