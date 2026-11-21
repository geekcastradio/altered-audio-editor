// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust dial maths must agree with `shared-fixtures/knob.json`,
//! the same file the TypeScript tests read. Values are compared to
//! 1e-9, since the two languages may differ in the last bit.

use altered_audio_editor_lib::knob::{drag_value, key_value, pointer_angle};
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/knob.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn drag_value_matches() {
    for c in fixture()["dragValue"].as_array().unwrap() {
        let got = drag_value(
            c["start"].as_f64().unwrap(),
            c["dy"].as_f64().unwrap(),
            c["ctrl"].as_bool().unwrap(),
            c["shift"].as_bool().unwrap(),
        );
        assert!(
            close(got, c["want"].as_f64().unwrap()),
            "{}: {got}",
            c["name"]
        );
    }
}

#[test]
fn key_value_matches() {
    for c in fixture()["keyValue"].as_array().unwrap() {
        let got = key_value(
            c["value"].as_f64().unwrap(),
            c["key"].as_str().unwrap(),
            c["home"].as_f64().unwrap(),
        );
        match c["want"].as_f64() {
            None => assert_eq!(got, None, "{}", c["name"]),
            Some(w) => assert!(close(got.unwrap(), w), "{}: {got:?}", c["name"]),
        }
    }
}

#[test]
fn pointer_angle_matches() {
    for c in fixture()["pointerAngle"].as_array().unwrap() {
        let got = pointer_angle(c["value"].as_f64().unwrap());
        assert!(close(got, c["want"].as_f64().unwrap()), "{c}: {got}");
    }
}
