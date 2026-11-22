// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust automation lane rules against
//! `shared-fixtures/automation.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::automation::*;
use altered_audio_editor_lib::envelope::Point;
use serde_json::Value;
use std::collections::HashSet;

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/automation.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn near(a: f64, b: f64, what: &Value) {
    let tol = 1e-9 * b.abs().max(1.0);
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b}");
}

fn target(v: &Value) -> Target {
    match v["type"].as_str().unwrap() {
        "volume" => Target::Volume,
        "pan" => Target::Pan,
        "space" => Target::Space(v["param"].as_str().unwrap().to_string()),
        _ => Target::EffectParam {
            slot_id: v["slot_id"].as_str().unwrap().into(),
            param: v["param"].as_str().unwrap().into(),
        },
    }
}

fn range(v: &Value) -> AutoRange {
    AutoRange {
        top: f(&v["top"]),
        bottom: f(&v["bottom"]),
        step: f(&v["step"]),
        log: v["log"].as_bool().unwrap(),
        unit: v["unit"].as_str().unwrap().into(),
    }
}

fn points(v: &Value) -> Vec<Point> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| Point {
            position: f(&p["position"]),
            value: f(&p["value"]),
            curve: p["curve"].as_str().unwrap().into(),
        })
        .collect()
}

fn same_points(got: &[Point], want: &Value, what: &Value) {
    let want = points(want);
    assert_eq!(got.len(), want.len(), "{what}");
    for (g, w) in got.iter().zip(&want) {
        near(g.position, w.position, what);
        near(g.value, w.value, what);
        assert_eq!(g.curve, w.curve, "{what}");
    }
}

#[test]
fn lanes_ranges_and_keys() {
    let fx = fixture();
    for c in fx["lane"].as_array().unwrap() {
        assert_eq!(lane_height(c["h"].as_f64()), f(&c["out"]), "{c}");
    }
    for c in fx["keys"].as_array().unwrap() {
        assert_eq!(target_key(&target(&c["t"])), c["out"].as_str().unwrap());
    }
    assert_eq!(
        volume_range(),
        AutoRange {
            top: 12.0,
            bottom: -60.0,
            step: 0.1,
            log: false,
            unit: "dB".into()
        }
    );
    assert_eq!(
        pan_range(),
        AutoRange {
            top: -100.0,
            bottom: 100.0,
            step: 1.0,
            log: false,
            unit: String::new()
        }
    );
    for c in fx["ranges"].as_array().unwrap() {
        assert_eq!(
            space_range(c["p"].as_str().unwrap()),
            range(&c["out"]),
            "{c}"
        );
    }
}

#[test]
fn pixels_values_snapping_and_text() {
    let fx = fixture();
    for c in fx["ymap"].as_array().unwrap() {
        let r = range(&c["r"]);
        let (top, h) = (f(&c["top"]), f(&c["h"]));
        if c["dir"] == "toY" {
            near(auto_y(f(&c["v"]), &r, top, h), f(&c["out"]), &c["r"]);
        } else {
            near(auto_value(f(&c["y"]), &r, top, h), f(&c["out"]), &c["r"]);
        }
    }
    for c in fx["snap"].as_array().unwrap() {
        near(auto_snap(&target(&c["t"]), f(&c["v"])), f(&c["out"]), c);
    }
    for c in fx["format"].as_array().unwrap() {
        assert_eq!(
            auto_format(&target(&c["t"]), f(&c["v"]), &range(&c["r"])),
            c["out"].as_str().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn values_and_points() {
    let fx = fixture();
    for c in fx["at"].as_array().unwrap() {
        near(
            auto_at(&points(&c["pts"]), f(&c["pos"]), f(&c["rest"])),
            f(&c["out"]),
            c,
        );
    }
    for c in fx["add"].as_array().unwrap() {
        let (got, index) = auto_add(&points(&c["pts"]), f(&c["pos"]), f(&c["v"]));
        same_points(&got, &c["out"]["points"], c);
        assert_eq!(index as f64, f(&c["out"]["index"]), "{c}");
    }
    for c in fx["move"].as_array().unwrap() {
        same_points(
            &auto_move(
                &points(&c["pts"]),
                c["i"].as_u64().unwrap() as usize,
                f(&c["pos"]),
                f(&c["v"]),
            ),
            &c["out"],
            c,
        );
    }
    for c in fx["many"].as_array().unwrap() {
        let sel: HashSet<usize> = c["sel"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i.as_u64().unwrap() as usize)
            .collect();
        let dv = f(&c["dv"]);
        same_points(
            &auto_move_many(&points(&c["pts"]), &sel, f(&c["d"]), |p| p.value + dv),
            &c["out"],
            c,
        );
    }
    for c in fx["thin"].as_array().unwrap() {
        same_points(&thin(&points(&c["pts"]), f(&c["tol"])), &c["out"], c);
    }
}
