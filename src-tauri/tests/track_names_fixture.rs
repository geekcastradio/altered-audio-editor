// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust track names must agree with `shared-fixtures/track-names.json`,
//! the same file the TypeScript tests read. Strings are compared exactly.

use altered_audio_editor_lib::track_names::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/track-names.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn track_name_from_file_matches() {
    for c in fixture()["trackNameFromFile"].as_array().unwrap() {
        assert_eq!(
            track_name_from_file(c["name"].as_str().unwrap()),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn new_track_names_matches() {
    for c in fixture()["newTrackNames"].as_array().unwrap() {
        let strings = |v: &Value| -> Vec<String> {
            v.as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap().to_string())
                .collect()
        };
        let got = new_track_names(
            &strings(&c["existing"]),
            c["count"].as_u64().unwrap() as usize,
            c["base"].as_str().unwrap(),
        );
        assert_eq!(got, strings(&c["want"]), "{c}");
    }
}
