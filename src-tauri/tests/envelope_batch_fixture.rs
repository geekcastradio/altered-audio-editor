// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust envelope, silence, stretch input and session tab rules must agree
//! with `shared-fixtures/envelope-batch.json`, the same file the TypeScript
//! tests read.

use altered_audio_editor_lib::envelope::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/envelope-batch.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

fn range(v: &Value) -> EnvRange {
    EnvRange {
        top: f(&v["top"]),
        bottom: f(&v["bottom"]),
        step: f(&v["step"]),
    }
}

fn points(v: &Value) -> Vec<Point> {
    serde_json::from_value(v.clone()).unwrap()
}

fn same_points(a: &[Point], b: &[Point]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(x, y)| {
            close(x.position, y.position) && close(x.value, y.value) && x.curve == y.curve
        })
}

fn kind(v: &Value) -> EnvKind {
    EnvKind::parse(v.as_str().unwrap()).unwrap()
}

#[test]
fn value_maths_matches() {
    let fx = fixture();
    for c in fx["valueToY"].as_array().unwrap() {
        assert!(
            close(
                value_to_y(f(&c["v"]), f(&c["top"]), f(&c["h"]), range(&c["range"])),
                f(&c["want"])
            ),
            "{c}"
        );
    }
    for c in fx["yToValue"].as_array().unwrap() {
        assert!(
            close(
                y_to_value(f(&c["y"]), f(&c["top"]), f(&c["h"]), range(&c["range"])),
                f(&c["want"])
            ),
            "{c}"
        );
    }
    for c in fx["snap"].as_array().unwrap() {
        assert!(
            close(snap_value(kind(&c["kind"]), f(&c["v"])), f(&c["want"])),
            "{c}"
        );
    }
    for c in fx["clamp"].as_array().unwrap() {
        assert!(
            close(clamp_value(f(&c["v"]), range(&c["range"])), f(&c["want"])),
            "{c}"
        );
    }
    for c in fx["format"].as_array().unwrap() {
        assert_eq!(
            format_value(kind(&c["kind"]), f(&c["v"]), c["unit"].as_str().unwrap()),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["dbToY"].as_array().unwrap() {
        assert!(
            close(
                db_to_y(f(&c["db"]), f(&c["top"]), f(&c["h"])),
                f(&c["want"])
            ),
            "{c}"
        );
    }
    for c in fx["yToDb"].as_array().unwrap() {
        assert!(
            close(y_to_db(f(&c["y"]), f(&c["top"]), f(&c["h"])), f(&c["want"])),
            "{c}"
        );
    }
    for c in fx["chipLabel"].as_array().unwrap() {
        let got = chip_label(
            c["v"].as_bool().unwrap(),
            c["p"].as_bool().unwrap(),
            c["f"].as_bool().unwrap(),
        );
        assert_eq!(got, c["want"].as_str().unwrap(), "{c}");
    }
    for c in fx["chipBox"].as_array().unwrap() {
        let got = chip_box(f(&c["x0"]), f(&c["x1"]), c["label"].as_str().unwrap());
        match c["want"].as_object() {
            None => assert!(got.is_none(), "{c}"),
            Some(w) => {
                let (x, width) = got.expect("a box");
                assert!(close(x, f(&w["x"])) && close(width, f(&w["w"])), "{c}");
            }
        }
    }
}

#[test]
fn point_edits_match() {
    let fx = fixture();
    for c in fx["levelAt"].as_array().unwrap() {
        assert!(
            close(level_at(&points(&c["points"]), f(&c["pos"])), f(&c["want"])),
            "{c}"
        );
    }
    for c in fx["slice"].as_array().unwrap() {
        let got = slice_volume(&points(&c["points"]), f(&c["from"]), f(&c["to"]));
        assert!(same_points(&got, &points(&c["want"])), "{c}");
    }
    for c in fx["addPoint"].as_array().unwrap() {
        let (got, i) = add_point(&points(&c["points"]), f(&c["pos"]), f(&c["value"]));
        assert!(same_points(&got, &points(&c["want"]["points"])), "{c}");
        assert_eq!(i as f64, f(&c["want"]["index"]), "{c}");
    }
    for c in fx["movePoint"].as_array().unwrap() {
        let got = move_point(
            &points(&c["points"]),
            f(&c["i"]) as usize,
            f(&c["pos"]),
            f(&c["value"]),
            f(&c["len"]),
            range(&c["range"]),
        );
        assert!(same_points(&got, &points(&c["want"])), "{c}");
    }
    for c in fx["movePoints"].as_array().unwrap() {
        let sel: Vec<usize> = c["sel"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| f(k) as usize)
            .collect();
        let how = &c["how"];
        let value_for = |p: &Point| match how.get("set") {
            Some(s) => f(s),
            None => p.value + f(&how["add"]),
        };
        let got = move_points(
            &points(&c["points"]),
            &sel,
            f(&c["dPos"]),
            value_for,
            f(&c["len"]),
            range(&c["range"]),
        );
        assert!(same_points(&got, &points(&c["want"])), "{c}");
    }
    for c in fx["silence"].as_array().unwrap() {
        let got = silence_span(
            &points(&c["points"]),
            f(&c["len"]),
            f(&c["from"]),
            f(&c["to"]),
            f(&c["ramp"]),
        );
        match c["want"].as_array() {
            None => assert!(got.is_none(), "{c}"),
            Some(_) => assert!(
                same_points(&got.expect("points"), &points(&c["want"])),
                "{c}"
            ),
        }
    }
}

#[test]
fn stretch_input_and_tabs_match() {
    let fx = fixture();
    for c in fx["percent"].as_array().unwrap() {
        assert_eq!(percent(f(&c["ratio"])), c["want"].as_str().unwrap(), "{c}");
    }
    for c in fx["parseSeconds"].as_array().unwrap() {
        let got = parse_seconds(c["text"].as_str().unwrap());
        match c["want"].as_f64() {
            None => assert!(got.is_none(), "{c}"),
            Some(w) => assert!(close(got.expect("seconds"), w), "{c}"),
        }
    }
    for c in fx["tabs"].as_array().unwrap() {
        let ids: Vec<String> = c["ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        assert_eq!(show_tabs(ids.len()), c["show"].as_bool().unwrap(), "{c}");
        for n in c["neighbors"].as_array().unwrap() {
            let want = n[1].as_str().map(str::to_string);
            assert_eq!(neighbor_of(&ids, n[0].as_str().unwrap()), want, "{c} {n}");
        }
    }
    for c in fx["tabLabels"].as_array().unwrap() {
        assert_eq!(
            tab_label(c["name"].as_str().unwrap(), c["dirty"].as_bool().unwrap()),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
}
