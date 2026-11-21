// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust time input rules must agree with `shared-fixtures/time-input.json`,
//! the same file the TypeScript tests read.

use altered_audio_editor_lib::time_input::{edit_selection, parse_time, TimeFormat};
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/time-input.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn parse_time_matches() {
    for c in fixture()["parseTime"].as_array().unwrap() {
        let format = match c["format"].as_str().unwrap() {
            "time" => TimeFormat::Time,
            "samples" => TimeFormat::Samples,
            "timecode" => TimeFormat::Timecode,
            "seconds" => TimeFormat::Seconds,
            other => panic!("unknown format {other}"),
        };
        let got = parse_time(
            c["in"].as_str().unwrap(),
            c["rate"].as_f64().unwrap(),
            format,
            c["fps"].as_f64().unwrap(),
        );
        assert_eq!(got, c["want"].as_f64(), "{}", c["name"]);
    }
}

#[test]
fn edit_selection_matches() {
    for c in fixture()["editSelection"].as_array().unwrap() {
        let s = c["sel"].as_array().unwrap();
        let got = edit_selection(
            (s[0].as_f64().unwrap(), s[1].as_f64().unwrap()),
            c["part"].as_str().unwrap(),
            c["value"].as_f64().unwrap(),
        );
        let want = c["want"].as_array().unwrap();
        assert_eq!(
            [got.0, got.1],
            [want[0].as_f64().unwrap(), want[1].as_f64().unwrap()],
            "{}",
            c["name"]
        );
    }
}
