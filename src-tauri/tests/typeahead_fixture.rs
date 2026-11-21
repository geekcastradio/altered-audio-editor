// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust type-to-jump rules must agree with `shared-fixtures/typeahead.json`,
//! the same file the TypeScript tests read.

use altered_audio_editor_lib::typeahead::*;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/typeahead.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn sequences_match() {
    for s in fixture()["sequences"].as_array().unwrap() {
        let labels: Vec<Option<&str>> = s["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l.as_str())
            .collect();
        let mut t = Typeahead::new();
        for st in s["steps"].as_array().unwrap() {
            let got = t.next(
                st["key"].as_str().unwrap(),
                &labels,
                st["current"].as_i64().unwrap(),
                st["now"].as_f64().unwrap(),
            );
            assert_eq!(got.map(|i| i as u64), st["want"].as_u64(), "{s}\n{st}");
        }
    }
}

#[test]
fn typing_keys_match() {
    for c in fixture()["typingKeys"].as_array().unwrap() {
        let got = is_typing_key(
            c["key"].as_str().unwrap(),
            c["ctrl"].as_bool().unwrap(),
            c["alt"].as_bool().unwrap(),
            c["meta"].as_bool().unwrap(),
        );
        assert_eq!(got, c["want"].as_bool().unwrap(), "{c}");
    }
}
