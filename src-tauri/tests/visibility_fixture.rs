// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust track visibility rules must agree with
//! `shared-fixtures/visibility.json`, the same file the TypeScript tests read.

use altered_audio_editor_lib::visibility::*;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/visibility.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn presets(v: &Value) -> Vec<VisPreset> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| VisPreset {
            name: p["name"].as_str().unwrap().to_string(),
            hidden: strings(&p["hidden"]),
        })
        .collect()
}

#[test]
fn show_only_matches() {
    let fx = fixture();
    let tracks: Vec<(String, bool)> = fx["showOnly"]["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| (t[0].as_str().unwrap().to_string(), t[1].as_bool().unwrap()))
        .collect();
    for c in fx["showOnly"]["cases"].as_array().unwrap() {
        let keep: Vec<bool> = c["keep"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b.as_bool().unwrap())
            .collect();
        let want: Vec<(String, bool)> = c["want"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| (t[0].as_str().unwrap().to_string(), t[1].as_bool().unwrap()))
            .collect();
        assert_eq!(show_only(&tracks, &keep), want, "{c}");
    }
}

#[test]
fn names_and_presets_match() {
    let fx = fixture();
    let h = &fx["hiddenNames"];
    let tracks: Vec<(String, bool)> = h["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| (t[0].as_str().unwrap().to_string(), t[1].as_bool().unwrap()))
        .collect();
    assert_eq!(hidden_names(&tracks), strings(&h["want"]));
    for c in fx["keptByPreset"].as_array().unwrap() {
        let hidden = strings(&c["hidden"]);
        for r in c["results"].as_array().unwrap() {
            assert_eq!(
                kept_by_preset(&hidden, r[0].as_str().unwrap()),
                r[1].as_bool().unwrap(),
                "{c} {r}"
            );
        }
    }
    for c in fx["withPreset"].as_array().unwrap() {
        let p = VisPreset {
            name: c["preset"]["name"].as_str().unwrap().to_string(),
            hidden: strings(&c["preset"]["hidden"]),
        };
        assert_eq!(
            with_preset(&presets(&c["list"]), p),
            presets(&c["want"]),
            "{c}"
        );
    }
    for c in fx["sanitize"].as_array().unwrap() {
        assert_eq!(sanitize_presets(&c["raw"]), presets(&c["want"]), "{c}");
    }
}

#[test]
fn clip_overlap_matches() {
    for c in fixture()["hasClipIn"].as_array().unwrap() {
        let spans: Vec<(f64, f64)> = c["spans"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| (s[0].as_f64().unwrap(), s[1].as_f64().unwrap()))
            .collect();
        let got = has_clip_in(
            &spans,
            c["start"].as_f64().unwrap(),
            c["end"].as_f64().unwrap(),
        );
        assert_eq!(got, c["want"].as_bool().unwrap(), "{c}");
    }
}
