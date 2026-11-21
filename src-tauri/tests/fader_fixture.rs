// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust fader rules must agree with `shared-fixtures/fader.json`, the same
//! file the TypeScript tests read. Numbers are compared to 1e-9; labels exactly.

use altered_audio_editor_lib::fader::*;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/fader.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn db_to_fader_matches() {
    for c in fixture()["dbToFader"].as_array().unwrap() {
        let got = db_to_fader(c["db"].as_f64().unwrap());
        assert!(
            (got - c["want"].as_f64().unwrap()).abs() < 1e-9,
            "{c}: {got}"
        );
    }
}

#[test]
fn fader_to_db_matches() {
    for c in fixture()["faderToDb"].as_array().unwrap() {
        let got = fader_to_db(c["pos"].as_f64().unwrap());
        assert!(
            (got - c["want"].as_f64().unwrap()).abs() < 1e-9,
            "{c}: {got}"
        );
    }
}

#[test]
fn format_db_matches() {
    for c in fixture()["formatDb"].as_array().unwrap() {
        assert_eq!(
            format_db(c["db"].as_f64().unwrap()),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn format_pan_matches() {
    for c in fixture()["formatPan"].as_array().unwrap() {
        assert_eq!(
            format_pan(c["p"].as_f64().unwrap()),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
}
