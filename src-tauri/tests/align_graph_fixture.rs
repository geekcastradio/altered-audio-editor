// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust Phase Align notice and response graph maths must agree with
//! `shared-fixtures/align-graph.json`, the same file the TypeScript tests read.

use altered_audio_editor_lib::align::*;
use altered_audio_editor_lib::graph::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/align-graph.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9 * b.abs().max(1.0)
}

#[test]
fn align_matches() {
    let fx = fixture();
    for c in fx["describeAlign"].as_array().unwrap() {
        let a = Alignment {
            lag: f(&c["lag"]),
            score: f(&c["score"]),
            inverted: c["inverted"].as_bool().unwrap(),
        };
        let got = describe_align(a, f(&c["rate"]), "Guest.wav", "Track 2");
        assert_eq!(got, c["want"].as_str().unwrap(), "{c}");
    }
    for c in fx["alignedStart"].as_array().unwrap() {
        assert!(
            close(aligned_start(f(&c["clip"]), f(&c["lag"])), f(&c["want"])),
            "{c}"
        );
    }
}

#[test]
fn graph_matches() {
    let fx = fixture();
    for c in fx["xOf"].as_array().unwrap() {
        assert!(close(x_of(f(&c["freq"])), f(&c["want"])), "{c}");
    }
    for c in fx["freqAt"].as_array().unwrap() {
        assert!(close(freq_at(f(&c["x"])), f(&c["want"])), "{c}");
    }
    for c in fx["yOf"].as_array().unwrap() {
        assert!(
            close(
                y_of(f(&c["db"]), f(&c["top"]), f(&c["bottom"])),
                f(&c["want"])
            ),
            "{c}"
        );
    }
    for c in fx["dbAt"].as_array().unwrap() {
        assert!(
            close(
                db_at(f(&c["y"]), f(&c["top"]), f(&c["bottom"])),
                f(&c["want"])
            ),
            "{c}"
        );
    }
    for c in fx["range"].as_array().unwrap() {
        let points: Vec<f64> = c["points"].as_array().unwrap().iter().map(f).collect();
        let gains: Vec<(f64, f64)> = c["gains"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| (f(&g[0]), f(&g[1])))
            .collect();
        let (top, bottom) = range(&points, &gains);
        assert!(
            close(top, f(&c["want"][0])) && close(bottom, f(&c["want"][1])),
            "{c}"
        );
    }
}
