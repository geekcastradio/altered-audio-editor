// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust In and Out rules must agree with `shared-fixtures/in-out.json`,
//! the same file the TypeScript tests read.

use altered_audio_editor_lib::in_out::{clear_in, clear_out, mark_in, mark_out};
use altered_audio_editor_lib::selection::Selection;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/in-out.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn sel(v: &Value) -> Option<Selection> {
    v.as_array().map(|a| Selection {
        start: a[0].as_i64().unwrap(),
        end: a[1].as_i64().unwrap(),
    })
}

fn pair(s: Option<Selection>) -> Value {
    s.map_or(Value::Null, |s| serde_json::json!([s.start, s.end]))
}

#[test]
fn mark_in_matches() {
    for c in fixture()["markIn"].as_array().unwrap() {
        let got = mark_in(
            sel(&c["sel"]),
            c["at"].as_f64().unwrap(),
            c["contentEnd"].as_i64().unwrap(),
        );
        assert_eq!(pair(got), c["want"], "{}", c["name"]);
    }
}

#[test]
fn mark_out_matches() {
    for c in fixture()["markOut"].as_array().unwrap() {
        let got = mark_out(sel(&c["sel"]), c["at"].as_f64().unwrap());
        assert_eq!(pair(got), c["want"], "{}", c["name"]);
    }
}

#[test]
fn clear_in_matches() {
    for c in fixture()["clearIn"].as_array().unwrap() {
        let got = clear_in(sel(&c["sel"]), c["contentEnd"].as_i64().unwrap());
        assert_eq!(pair(got), c["want"], "{}", c["name"]);
    }
}

#[test]
fn clear_out_matches() {
    for c in fixture()["clearOut"].as_array().unwrap() {
        let got = clear_out(sel(&c["sel"]), c["contentEnd"].as_i64().unwrap());
        assert_eq!(pair(got), c["want"], "{}", c["name"]);
    }
}
