// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the panel placement rules in Rust against `shared-fixtures/docking.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::docking::*;
use serde_json::Value;
use std::collections::HashSet;

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/docking.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn set(v: &Value) -> HashSet<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn homes() {
    let fx = fixture();
    let list = fx["homes"].as_array().unwrap();
    assert!(list.len() > 100);
    for c in list {
        let got = serde_json::to_value(resolve_home(c["id"].as_str().unwrap(), &set(&c["open"])))
            .unwrap();
        assert_eq!(got, c["out"], "case {c}");
    }
}

#[test]
fn workspace_ids() {
    let fx = fixture();
    for c in fx["wids"].as_array().unwrap() {
        let got = workspace_id(c["name"].as_str().unwrap(), &set(&c["taken"]));
        assert_eq!(got, c["out"].as_str().unwrap(), "case {c}");
    }
}

#[test]
fn group_minimums() {
    let fx = fixture();
    for c in fx["groups"].as_array().unwrap() {
        let ids: Vec<String> = c["ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect();
        let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
        let (w, h) = group_min(&refs);
        assert_eq!(w, c["out"]["w"].as_f64().unwrap(), "case {c}");
        assert_eq!(h, c["out"]["h"].as_f64().unwrap(), "case {c}");
    }
}

#[test]
fn lookups() {
    let fx = fixture();
    for c in fx["presets"].as_array().unwrap() {
        assert_eq!(
            preset_by_id(c["id"].as_str().unwrap()).is_some(),
            c["found"].as_bool().unwrap(),
            "preset {c}"
        );
    }
    for c in fx["defs"].as_array().unwrap() {
        assert_eq!(
            panel_def(c["id"].as_str().unwrap()).map(|p| p.title.as_str()),
            c["title"].as_str(),
            "panel {c}"
        );
    }
    assert!(registry().layout_version > 0);
    assert!(!registry().layout.is_empty());
}
