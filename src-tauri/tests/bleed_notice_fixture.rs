// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust Reduce Bleed notice rules must agree with
//! `shared-fixtures/bleed-notice.json`, the same file the TypeScript tests read.

use altered_audio_editor_lib::bleed_notice::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/bleed-notice.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn work_matches() {
    for c in fixture()["work"].as_array().unwrap() {
        let clips: Vec<ClipFlags> = c["clips"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| ClipFlags {
                locked: f["locked"].as_bool().unwrap(),
                looped: f["looped"].as_bool().unwrap(),
                stretched: f["stretched"].as_bool().unwrap(),
            })
            .collect();
        let got = bleed_work(&clips);
        let w = &c["want"];
        let want: Vec<usize> = w["work"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        assert_eq!(got.work, want, "{c}");
        assert_eq!(got.locked as u64, w["locked"].as_u64().unwrap(), "{c}");
        assert_eq!(
            got.unsuitable as u64,
            w["unsuitable"].as_u64().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn summary_matches() {
    for c in fixture()["summary"].as_array().unwrap() {
        let removed: Vec<f64> = c["removed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let got = bleed_summary(
            &removed,
            c["locked"].as_u64().unwrap() as usize,
            c["unsuitable"].as_u64().unwrap() as usize,
            c["changed"].as_u64().unwrap() as usize,
        );
        assert_eq!(got, c["want"].as_str().unwrap(), "{c}");
    }
}

#[test]
fn mixdown_matches() {
    for c in fixture()["mixdown"].as_array().unwrap() {
        let got = mixdown_bytes(
            c["samples"].as_f64().unwrap(),
            c["bits"].as_f64().unwrap(),
            c["channels"].as_f64().unwrap(),
        );
        assert!((got - c["want"].as_f64().unwrap()).abs() < 1e-9, "{c}");
    }
}
