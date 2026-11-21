// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust Dynamics curve must agree with `shared-fixtures/dynamics-curve.json`,
//! the same file the TypeScript tests read. Values are compared to 1e-9.

use altered_audio_editor_lib::dynamics_curve::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/dynamics-curve.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn curve(c: &Value) -> Curve {
    Curve {
        threshold: c["threshold"].as_f64().unwrap(),
        ratio: c["ratio"].as_f64().unwrap(),
        knee: c["knee"].as_f64().unwrap(),
        makeup: c["makeup"].as_f64().unwrap(),
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn check_points(got: &[(f64, f64)], want: &Value, what: &Value) {
    let want = want.as_array().unwrap();
    assert_eq!(got.len(), want.len(), "{what}");
    for (g, w) in got.iter().zip(want) {
        let w = w.as_array().unwrap();
        assert!(
            close(g.0, w[0].as_f64().unwrap()) && close(g.1, w[1].as_f64().unwrap()),
            "{what}: {g:?} vs {w:?}"
        );
    }
}

#[test]
fn transfer_matches() {
    for c in fixture()["transfer"].as_array().unwrap() {
        let got = transfer(c["inDb"].as_f64().unwrap(), &curve(c));
        assert!(close(got, c["want"].as_f64().unwrap()), "{c}: {got}");
    }
}

#[test]
fn curve_points_matches() {
    for c in fixture()["curvePoints"].as_array().unwrap() {
        let got = curve_points(
            &curve(c),
            c["lo"].as_f64().unwrap(),
            c["hi"].as_f64().unwrap(),
            c["steps"].as_u64().unwrap() as usize,
        );
        check_points(&got, &c["want"], c);
    }
}

#[test]
fn curve_points_defaults_match() {
    for c in fixture()["curvePointsDefault"].as_array().unwrap() {
        let got = curve_points(&curve(c), DEFAULT_LO, DEFAULT_HI, DEFAULT_STEPS);
        check_points(&got, &c["want"], c);
    }
}

#[test]
fn ratio_from_slope_matches() {
    for c in fixture()["ratioFromSlope"].as_array().unwrap() {
        let min = c["min"].as_f64().unwrap_or(1.0);
        let max = c["max"].as_f64().unwrap_or(20.0);
        let got = ratio_from_slope(c["slope"].as_f64().unwrap(), min, max);
        assert!(close(got, c["want"].as_f64().unwrap()), "{c}: {got}");
    }
}
