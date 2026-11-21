// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust tag helpers must agree with `shared-fixtures/tags.json`, the same
//! file the TypeScript tests read. Strings are compared exactly.

use altered_audio_editor_lib::tags::*;
use serde_json::Value;
use std::collections::HashMap;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/tags.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn draft(v: &Value) -> HashMap<String, String> {
    v.as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
        .collect()
}

fn as_map(v: &Value) -> HashMap<String, String> {
    draft(v)
}

fn got_map(got: Vec<(&'static str, String)>) -> HashMap<String, String> {
    got.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

#[test]
fn clean_tags_matches() {
    for c in fixture()["clean"].as_array().unwrap() {
        assert_eq!(
            got_map(clean_tags(&draft(&c["draft"]))),
            as_map(&c["want"]),
            "{c}"
        );
    }
}

#[test]
fn save_tags_matches() {
    for c in fixture()["save"].as_array().unwrap() {
        assert_eq!(
            got_map(save_tags(&draft(&c["draft"]))),
            as_map(&c["want"]),
            "{c}"
        );
    }
}

#[test]
fn taggable_matches() {
    for c in fixture()["taggable"].as_array().unwrap() {
        assert_eq!(
            is_taggable(c["path"].as_str().unwrap()),
            c["want"].as_bool().unwrap(),
            "{c}"
        );
    }
}
