// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust clip level and view rules must agree with
//! `shared-fixtures/clip-gain.json`, the same file the TypeScript tests read.

use altered_audio_editor_lib::clip_gain::*;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/clip-gain.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn gain_box_matches() {
    for c in fixture()["gainBox"].as_array().unwrap() {
        let got = gain_box(f(&c["x0"]), f(&c["x1"]), f(&c["bottom"]), f(&c["top"]));
        match (&c["want"], got) {
            (Value::Null, None) => {}
            (w, Some(g)) if !w.is_null() => {
                assert!(close(g.x, f(&w["x"])) && close(g.y, f(&w["y"])), "{c}");
                assert!(close(g.w, f(&w["w"])) && close(g.h, f(&w["h"])), "{c}");
            }
            _ => panic!("mismatch: {c}"),
        }
    }
}

#[test]
fn drag_gain_matches() {
    for c in fixture()["dragGain"].as_array().unwrap() {
        let got = drag_gain(
            f(&c["start"]),
            f(&c["dx"]),
            f(&c["dy"]),
            c["fine"].as_bool().unwrap(),
        );
        assert!(close(got, f(&c["want"])), "{c}: {got}");
    }
}

#[test]
fn gain_text_matches() {
    for c in fixture()["gainText"].as_array().unwrap() {
        assert_eq!(gain_text(f(&c["db"])), c["want"].as_str().unwrap(), "{c}");
    }
}

fn view(v: &Value) -> ViewDoc {
    ViewDoc {
        scroll: f(&v["scroll"]),
        samples_per_pixel: f(&v["samples_per_pixel"]),
    }
}

#[test]
fn view_range_matches() {
    for c in fixture()["viewRange"].as_array().unwrap() {
        let (s, e) = view_range(view(&c["v"]), f(&c["width"]));
        assert!(
            close(s, f(&c["want"]["start"])) && close(e, f(&c["want"]["end"])),
            "{c}"
        );
    }
}

#[test]
fn view_for_matches() {
    for c in fixture()["viewFor"].as_array().unwrap() {
        let got = view_for(f(&c["start"]), f(&c["end"]), f(&c["width"]));
        let w = view(&c["want"]);
        assert!(
            close(got.scroll, w.scroll) && close(got.samples_per_pixel, w.samples_per_pixel),
            "{c}"
        );
    }
}

#[test]
fn edit_selection_matches() {
    for c in fixture()["editSelection"].as_array().unwrap() {
        let part = match c["part"].as_str().unwrap() {
            "start" => SelectionPart::Start,
            "end" => SelectionPart::End,
            _ => SelectionPart::Duration,
        };
        let (a, b) = edit_selection(
            (f(&c["sel"]["start"]), f(&c["sel"]["end"])),
            part,
            f(&c["value"]),
        );
        assert!(
            close(a, f(&c["want"]["start"])) && close(b, f(&c["want"]["end"])),
            "{c}"
        );
    }
}
