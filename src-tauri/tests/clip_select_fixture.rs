// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust clip selection rules must agree with
//! `shared-fixtures/clip-select.json`, the same file the TypeScript tests read.

use std::collections::HashSet;

use altered_audio_editor_lib::clip_select::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/clip-select.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn clips(fx: &Value, c: &Value) -> Vec<Positioned> {
    fx["sets"][c["set"].as_str().unwrap()]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| Positioned {
            id: p["id"].as_str().unwrap().to_string(),
            start: p["start"].as_f64().unwrap(),
        })
        .collect()
}

fn chosen(c: &Value) -> HashSet<String> {
    c["chosen"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn ids(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn anchor_start_matches() {
    let fx = fixture();
    for c in fx["anchorStart"].as_array().unwrap() {
        let got = anchor_start(&clips(&fx, c), &chosen(c), c["playhead"].as_f64().unwrap());
        assert!(
            (got - c["want"].as_f64().unwrap()).abs() < 1e-9,
            "{c}: {got}"
        );
    }
}

#[test]
fn clips_from_and_up_to_match() {
    let fx = fixture();
    for c in fx["clipsFrom"].as_array().unwrap() {
        let got = clips_from(&clips(&fx, c), c["at"].as_f64().unwrap());
        assert_eq!(got, ids(&c["want"]), "{c}");
    }
    for c in fx["clipsUpTo"].as_array().unwrap() {
        let got = clips_up_to(&clips(&fx, c), c["at"].as_f64().unwrap());
        assert_eq!(got, ids(&c["want"]), "{c}");
    }
}

#[test]
fn next_clip_id_matches() {
    let fx = fixture();
    for c in fx["nextClipId"].as_array().unwrap() {
        let got = next_clip_id(&clips(&fx, c), &chosen(c), c["playhead"].as_f64().unwrap());
        assert_eq!(got.as_deref(), c["want"].as_str(), "{c}");
    }
}
