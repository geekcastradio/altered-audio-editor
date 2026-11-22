// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust playhead key rules, update notice rules and insert-session
//! wording against `shared-fixtures/playhead-update.json`, which the
//! TypeScript test reads too.

use altered_audio_editor_lib::playhead_keys::*;
use altered_audio_editor_lib::session_rules::{insert_notice, mix_name};
use altered_audio_editor_lib::time_input::TimeFormat;
use altered_audio_editor_lib::update_notes::*;
use serde_json::Value;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/playhead-update.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn same(got: Option<f64>, want: &Value, what: &str) {
    match (got, want) {
        (None, Value::Null) => {}
        (Some(g), w) if w.is_number() => {
            assert!((g - f(w)).abs() < 1e-9, "{what}: got {g}, want {w}")
        }
        _ => panic!("{what}: got {got:?}, want {want}"),
    }
}

fn pair(v: &Value) -> Option<(f64, f64)> {
    (!v.is_null()).then(|| (f(&v["start"]), f(&v["end"])))
}

#[test]
fn head_and_selection_keys() {
    let fx = fixture();
    for c in fx["head"].as_array().unwrap() {
        let k = &c["c"];
        let ctx = HeadContext {
            pos: f(&k["pos"]),
            end: f(&k["end"]),
            spp: f(&k["spp"]),
            width: f(&k["width"]),
            sel: pair(&k["sel"]),
        };
        let mv = HeadMove::parse(c["m"].as_str().unwrap()).unwrap();
        same(head_target(mv, &ctx), &c["out"], &format!("head {c}"));
    }
    for c in fx["jumps"].as_array().unwrap() {
        let fmt = match c["format"].as_str().unwrap() {
            "time" => TimeFormat::Time,
            "samples" => TimeFormat::Samples,
            "timecode" => TimeFormat::Timecode,
            _ => TimeFormat::Seconds,
        };
        let got = jump_target(
            c["text"].as_str().unwrap(),
            f(&c["rate"]),
            fmt,
            f(&c["fps"]),
            f(&c["end"]),
        );
        same(got, &c["out"], &format!("jump {c}"));
    }
    for c in fx["adj"].as_array().unwrap() {
        let mv = EdgeMove::parse(c["m"].as_str().unwrap()).unwrap();
        let got = adjust_selection(pair(&c["sel"]), mv, f(&c["step"]), f(&c["end"]));
        match (got, pair(&c["out"])) {
            (None, None) => {}
            (Some(g), Some(w)) => assert!(
                (g.0 - w.0).abs() < 1e-9 && (g.1 - w.1).abs() < 1e-9,
                "adjust {c}: got {g:?}"
            ),
            (g, _) => panic!("adjust {c}: got {g:?}"),
        }
    }
}

#[test]
fn update_rules() {
    let fx = fixture();
    for c in fx["due"].as_array().unwrap() {
        let every = if c["every"] == "launch" {
            UpdateEvery::Launch
        } else {
            UpdateEvery::Week
        };
        let last = if c["last"].is_null() {
            f64::NAN
        } else {
            f(&c["last"])
        };
        assert_eq!(
            update_due(c["enabled"].as_bool().unwrap(), last, f(&c["now"]), every),
            c["out"].as_bool().unwrap(),
            "{c}"
        );
    }
    for c in fx["plain"].as_array().unwrap() {
        assert_eq!(
            plain_notes(c["md"].as_str().unwrap()),
            c["out"].as_str().unwrap(),
            "plain {c}"
        );
    }
    for c in fx["hl"].as_array().unwrap() {
        let got = highlights(
            c["md"].as_str(),
            c["max"].as_u64().unwrap() as usize,
            c["width"].as_u64().unwrap() as usize,
        );
        let want: Vec<String> = c["out"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        assert_eq!(got, want, "highlights {c}");
    }
}

#[test]
fn insert_session_wording() {
    let fx = fixture();
    for c in fx["mix"].as_array().unwrap() {
        assert_eq!(
            mix_name(c["p"].as_str().unwrap()),
            c["out"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["notices"].as_array().unwrap() {
        assert_eq!(
            insert_notice(c["name"].as_str().unwrap(), f(&c["missing"]), f(&c["peak"])),
            c["out"].as_str().unwrap()
        );
    }
}
