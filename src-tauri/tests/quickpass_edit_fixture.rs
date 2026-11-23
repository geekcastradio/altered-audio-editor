// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the QuickPass edits in Rust against
//! `shared-fixtures/quickpass-edit.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::clip_edit::LockedError;
use altered_audio_editor_lib::quickpass_edit::*;
use altered_audio_editor_lib::quickpass_scan::Finding;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/quickpass-edit.json");
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

fn s(v: &Value) -> String {
    v.as_str().unwrap().to_string()
}

fn finding(v: &Value) -> Finding {
    Finding {
        id: s(&v["id"]),
        kind: s(&v["kind"]),
        track_id: s(&v["trackId"]),
        clip_id: s(&v["clipId"]),
        clip_name: s(&v["clipName"]),
        start: f(&v["start"]),
        end: f(&v["end"]),
    }
}

fn list(v: &Value) -> Vec<Finding> {
    v.as_array().unwrap().iter().map(finding).collect()
}

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("n{n}0")
    }
}

#[test]
fn removing_splitting_ducking() {
    let fx = fixture();
    let doc = &fx["doc"];
    let lists = fx["lists"].as_array().unwrap();
    for c in fx["removes"].as_array().unwrap() {
        let found = list(&lists[c["li"].as_u64().unwrap() as usize]);
        let got = match remove_op(
            doc,
            &found,
            c["ripple"].as_str().unwrap(),
            f(&c["keepMs"]),
            &mut counter(),
        ) {
            Ok(v) => v.unwrap_or(Value::Null),
            Err(LockedError) => json!({ "locked": true }),
        };
        ok(&got, &c["out"], &format!("remove {c}"));
    }
    for c in fx["splits"].as_array().unwrap() {
        let found = list(&lists[c["li"].as_u64().unwrap() as usize]);
        ok(
            &split_op(doc, &found, &mut counter()).unwrap_or(Value::Null),
            &c["out"],
            &format!("split {}", c["li"]),
        );
    }
    for c in fx["clipsOn"].as_array().unwrap() {
        let found = list(&lists[c["li"].as_u64().unwrap() as usize]);
        let strings = |v: &Value| v.as_array().unwrap().iter().map(s).collect::<Vec<_>>();
        assert_eq!(clips_on(doc, &found), strings(&c["before"]), "{c}");
        let after = split_op(doc, &found, &mut counter())
            .map_or_else(|| doc.clone(), |op| apply_local(doc, &op));
        assert_eq!(clips_on(&after, &found), strings(&c["after"]), "{c}");
    }
    for c in fx["ducks"].as_array().unwrap() {
        let found = list(&lists[c["li"].as_u64().unwrap() as usize]);
        ok(
            &duck_op(doc, &found, f(&c["duckDb"])).unwrap_or(Value::Null),
            &c["out"],
            &format!("duck {c}"),
        );
    }
}

#[test]
fn markers_shifts_cuts_and_local_edits() {
    let fx = fixture();
    let doc = &fx["doc"];
    let lists = fx["lists"].as_array().unwrap();
    for c in fx["markers"].as_array().unwrap() {
        let found = list(&lists[c["li"].as_u64().unwrap() as usize]);
        ok(
            &Value::Array(markers_for(
                doc,
                &found,
                &mut counter(),
                c["base"].as_str().unwrap(),
            )),
            &c["out"],
            &format!("markers {c}"),
        );
    }
    for c in fx["shifts"].as_array().unwrap() {
        let all = list(&lists[c["li"].as_u64().unwrap() as usize]);
        let gone = all
            .iter()
            .find(|x| x.id == s(&c["goneId"]))
            .unwrap()
            .clone();
        let rest: Vec<Finding> = all.iter().filter(|x| x.id != gone.id).cloned().collect();
        let got: Vec<Value> = shift_findings(
            &rest,
            &gone,
            c["ripple"].as_str().unwrap(),
            f(&c["keepMs"]),
            48000.0,
        )
        .iter()
        .map(Finding::to_json)
        .collect();
        ok(&Value::Array(got), &c["out"], &format!("shift {c}"));
    }
    for c in fx["cuts"].as_array().unwrap() {
        let got = cut_range(f(&c["start"]), f(&c["end"]), f(&c["keepMs"]), 48000.0)
            .map_or(Value::Null, |(a, b)| json!([a, b]));
        ok(&got, &c["out"], &format!("cut {c}"));
    }
    for c in fx["ops"].as_array().unwrap() {
        ok(
            &apply_local(doc, &c["op"]),
            &c["out"],
            &format!("apply {}", c["op"]),
        );
    }
}
