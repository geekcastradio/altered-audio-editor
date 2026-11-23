// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks copy and paste, previews, layers, crossfades, groups, insert time,
//! clear range and copy-moved in Rust against `shared-fixtures/clip-edit-3.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::clip_edit::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/clip-edit-3.json");
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

fn list(v: &Value) -> Vec<Value> {
    v.as_array().unwrap().clone()
}

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("n{n}0")
    }
}

fn locked_or(r: Result<Option<Value>, LockedError>) -> Value {
    match r {
        Ok(v) => v.unwrap_or(Value::Null),
        Err(LockedError) => json!({ "locked": true }),
    }
}

#[test]
fn copy_and_paste() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["copies"].as_array().unwrap() {
        ok(
            &copy_clips(doc, &strings(&c["ids"])).unwrap_or(Value::Null),
            &c["out"],
            &format!("copy {}", c["ids"]),
        );
    }
    for c in fx["pastes"].as_array().unwrap() {
        let got = paste_clips(doc, &c["board"], f(&c["at"]), f(&c["row"]), &mut counter())
            .unwrap_or(Value::Null);
        ok(&got, &c["out"], &format!("paste {c}"));
    }
    for c in fx["pasteEmpty"].as_array().unwrap() {
        let d = if c["out"].is_null() && c["board"]["items"].as_array().unwrap().is_empty() {
            doc.clone()
        } else {
            json!({ "tracks": [], "media": [] })
        };
        let got = paste_clips(&d, &c["board"], 0.0, 0.0, &mut counter()).unwrap_or(Value::Null);
        ok(&got, &c["out"], "paste empty");
    }
}

#[test]
fn previews_layers_and_crossfades() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["previews"].as_array().unwrap() {
        ok(
            &Value::Array(preview_tracks(doc, &list(&c["changes"]))),
            &c["out"],
            &format!("preview {}", c["changes"]),
        );
    }
    for c in fx["layers"].as_array().unwrap() {
        ok(
            &Value::Array(with_layers(doc, &list(&c["changes"]))),
            &c["out"],
            &format!("layers {}", c["changes"]),
        );
    }
    let shape = fx["shape"].as_str().unwrap();
    for c in fx["crossfades"].as_array().unwrap() {
        ok(
            &Value::Array(with_crossfades(doc, &list(&c["changes"]), shape)),
            &c["out"],
            &format!("crossfade {}", c["changes"]),
        );
    }
    for c in fx["crossfades2"].as_array().unwrap() {
        ok(
            &Value::Array(with_crossfades(&fx["xdoc"], &list(&c["changes"]), shape)),
            &c["out"],
            &format!("crossfade2 {}", c["changes"]),
        );
    }
}

#[test]
fn shapes_and_groups() {
    let fx = fixture();
    let doc = &fx["doc"];
    let clips: Vec<Value> = doc["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["clips"].as_array().unwrap().clone())
        .collect();
    for c in fx["shapes"].as_array().unwrap() {
        ok(
            &Value::Array(fade_shape_changes(&clips, c["shape"].as_str().unwrap())),
            &c["out"],
            &format!("shape {}", c["shape"]),
        );
    }
    for c in fx["groups"].as_array().unwrap() {
        let got = with_groups(doc, &strings(&c["ids"]), c["sus"].as_bool().unwrap());
        assert_eq!(got, strings(&c["out"]), "{c}");
    }
}

#[test]
fn inserting_clearing_and_copying() {
    let fx = fixture();
    let doc = &fx["doc"];
    for c in fx["ins"].as_array().unwrap() {
        let got = locked_or(insert_time(
            doc,
            &strings(&c["tracks"]),
            f(&c["at"]),
            f(&c["len"]),
            &mut counter(),
        ));
        ok(&got, &c["out"], &format!("insert {c}"));
    }
    for c in fx["clr"].as_array().unwrap() {
        let got = locked_or(clear_range(
            doc,
            &strings(&c["tracks"]),
            f(&c["a"]),
            f(&c["b"]),
            &mut counter(),
        ));
        ok(&got, &c["out"], &format!("clear {c}"));
    }
    for c in fx["cm"].as_array().unwrap() {
        let (op, ids) = copy_moved(
            doc,
            &strings(&c["moving"]),
            &list(&c["changes"]),
            &mut counter(),
        );
        ok(
            &json!({ "op": op, "ids": ids }),
            &c["out"],
            &format!("copy moved {c}"),
        );
    }
}
