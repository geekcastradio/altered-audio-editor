// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust selection rules must agree with `shared-fixtures/selection.json`,
//! the same file the TypeScript tests read.

use altered_audio_editor_lib::selection::*;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/selection.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn sel(v: &Value) -> Option<Selection> {
    v.as_array().map(|a| Selection {
        start: a[0].as_i64().unwrap(),
        end: a[1].as_i64().unwrap(),
    })
}

fn pair(s: Option<Selection>) -> Value {
    s.map_or(Value::Null, |s| serde_json::json!([s.start, s.end]))
}

#[test]
fn adjust_selection_matches() {
    for c in fixture()["adjustSelection"].as_array().unwrap() {
        let mv = match c["move"].as_str().unwrap() {
            "start-left" => EdgeMove::StartLeft,
            "start-right" => EdgeMove::StartRight,
            "end-left" => EdgeMove::EndLeft,
            _ => EdgeMove::EndRight,
        };
        let got = adjust_selection(
            sel(&c["sel"]),
            mv,
            c["step"].as_f64().unwrap(),
            c["end"].as_f64().unwrap(),
        );
        assert_eq!(pair(got), c["want"], "{}", c["name"]);
    }
}

#[test]
fn head_target_matches() {
    for c in fixture()["headTarget"].as_array().unwrap() {
        let mv = match c["move"].as_str().unwrap() {
            "step-left" => HeadMove::StepLeft,
            "step-right" => HeadMove::StepRight,
            "page-left" => HeadMove::PageLeft,
            "page-right" => HeadMove::PageRight,
            "to-in" => HeadMove::ToIn,
            _ => HeadMove::ToOut,
        };
        let got = head_target(
            mv,
            c["pos"].as_i64().unwrap(),
            c["end"].as_i64().unwrap(),
            c["spp"].as_f64().unwrap(),
            c["width"].as_f64().unwrap(),
            sel(&c["sel"]),
        );
        let want = c["want"].as_i64();
        assert_eq!(got, want, "{}", c["name"]);
    }
}

#[test]
fn play_start_matches() {
    for c in fixture()["playStart"].as_array().unwrap() {
        let got = play_start(
            c["playhead"].as_i64().unwrap(),
            sel(&c["sel"]),
            c["skips"].as_bool().unwrap(),
        );
        assert_eq!(got, c["want"].as_i64().unwrap(), "{}", c["name"]);
    }
}

#[test]
fn stop_at_selection_matches() {
    for c in fixture()["stopAtSelectionDue"].as_array().unwrap() {
        let got = stop_at_selection_due(StopAtSelection {
            enabled: c["enabled"].as_bool().unwrap(),
            backend_stops_at_end: c["backendStopsAtEnd"].as_bool().unwrap(),
            skips: c["skips"].as_bool().unwrap(),
            started_at: c["startedAt"].as_i64().unwrap(),
            position: c["position"].as_i64().unwrap(),
            sel: sel(&c["sel"]).unwrap(),
        });
        assert_eq!(got, c["want"].as_bool().unwrap(), "{}", c["name"]);
    }
}
