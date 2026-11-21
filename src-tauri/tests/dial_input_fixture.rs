// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust dial input rules must agree with `shared-fixtures/dial-input.json`,
//! the same file the TypeScript tests read.

use altered_audio_editor_lib::dial_input::{parse_number, parse_pan};
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/dial-input.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn check(list: &str, parse: fn(&str) -> Option<f64>) {
    for c in fixture()[list].as_array().unwrap() {
        let input = c["in"].as_str().unwrap();
        assert_eq!(parse(input), c["want"].as_f64(), "{list}: {input:?}");
    }
}

#[test]
fn parse_number_matches() {
    check("parseNumber", parse_number);
}

#[test]
fn parse_pan_matches() {
    check("parsePan", parse_pan);
}
