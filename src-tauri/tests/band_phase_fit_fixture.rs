// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust band split, Phase Curve editor and Fit Music to Length rules
//! against `shared-fixtures/band-phase-fit.json`, which the TypeScript test
//! reads too.

use altered_audio_editor_lib::band_split::*;
use altered_audio_editor_lib::music_fit::*;
use altered_audio_editor_lib::phase_curve_ui::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/band-phase-fit.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

/// Equal, with numbers compared as numbers (1 and 1.0 are the same).
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            (x - y).abs() <= 1e-9 * (1.0 + x.abs().max(y.abs()))
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        _ => a == b,
    }
}

fn ok(a: &Value, b: &Value, what: &str) {
    assert!(same(a, b), "{what}\n got {a}\nwant {b}");
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("u{n}")
    }
}

fn nums(v: &Value) -> Vec<f64> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| if x.is_null() { f64::NAN } else { f(x) })
        .collect()
}

#[test]
fn band_split() {
    let fx = fixture();
    assert_eq!(BAND_MIN_HZ, f(&fx["consts"]["min"]));
    assert_eq!(BAND_MAX_HZ, f(&fx["consts"]["max"]));
    assert_eq!(BAND_MIN_RATIO, f(&fx["consts"]["ratio"]));
    for c in fx["parse"].as_array().unwrap() {
        let got = parse_edges(c["t"].as_str().unwrap())
            .map(|v| json!(v))
            .unwrap_or(Value::Null);
        ok(&got, &c["out"], &format!("parse {c}"));
    }
    for c in fx["check"].as_array().unwrap() {
        let got = check_edges(&nums(&c["e"]))
            .map(Value::from)
            .unwrap_or(Value::Null);
        assert_eq!(got, c["out"], "check {c}");
    }
    for c in fx["slots"].as_array().unwrap() {
        let mut id = counter();
        let got = band_slots(c["k"].as_u64().unwrap() as usize, &nums(&c["e"]), &mut id);
        ok(&Value::Array(got), &c["out"], &format!("slots {c}"));
    }
    for c in fx["splitCases"].as_array().unwrap() {
        let tracks = c["tracks"].as_array().unwrap();
        let found: Vec<Located> = c["sel"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                let ti = s["ti"].as_u64().unwrap() as usize;
                let clip = tracks[ti]["clips"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|k| k["id"] == s["id"])
                    .unwrap()
                    .clone();
                Located {
                    track: tracks[ti].clone(),
                    clip,
                    track_index: ti,
                }
            })
            .collect();
        let mut id = counter();
        let got = band_split_op(&found, &nums(&c["edges"]), &mut id).unwrap_or(Value::Null);
        ok(&got, &c["out"], &format!("split {}", c["edges"]));
    }
}

#[test]
fn phase_curve() {
    let fx = fixture();
    for c in fx["phase"].as_array().unwrap() {
        let slot = &c["slot"];
        assert_eq!(point_count(slot) as f64, f(&c["count"]));
        assert_eq!(range_of(slot), f(&c["range"]));
        let pts = curve_points(slot);
        let want = c["pts"].as_array().unwrap();
        assert_eq!(pts.len(), want.len());
        for (p, w) in pts.iter().zip(want) {
            assert_eq!(p.index as f64, f(&w["index"]));
            assert!(
                (p.hz - f(&w["hz"])).abs() < 1e-9 && (p.deg - f(&w["deg"])).abs() < 1e-9,
                "point {w}"
            );
        }
        for a in c["at"].as_array().unwrap() {
            let got = curve_at(&pts, a["sp"].as_bool().unwrap(), f(&a["hz"]));
            assert!((got - f(&a["out"])).abs() < 1e-9, "curve_at {a}: got {got}");
        }
        let add = add_point(slot, 123.0, -45.0)
            .map(Value::Object)
            .unwrap_or(Value::Null);
        ok(&add, &c["add"], "add");
        for (i, w) in c["rem"].as_array().unwrap().iter().enumerate() {
            ok(
                &remove_point(slot, i)
                    .map(Value::Object)
                    .unwrap_or(Value::Null),
                w,
                &format!("remove {i}"),
            );
        }
        for s in c["shown"].as_array().unwrap() {
            assert_eq!(
                group_shown(
                    s["effect"].as_str().unwrap(),
                    slot,
                    s["g"].as_str().unwrap()
                ),
                s["out"].as_bool().unwrap(),
                "{s}"
            );
        }
    }
    assert_eq!(curve_at(&[], true, 100.0), f(&fx["emptyAt"]));
    assert_eq!(PHASE_CURVE, "altaudio.phase_curve");
}

fn plan_of(p: &Value) -> FitPlan {
    FitPlan {
        segments: p["segments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| (f(&s["start"]), f(&s["end"])))
            .collect(),
        length: f(&p["length"]),
        stretch: f(&p["stretch"]),
        joins: p["joins"].as_u64().unwrap() as u32,
        quality: f(&p["quality"]),
        tempo: f(&p["tempo"]),
    }
}

#[test]
fn fit_music() {
    let fx = fixture();
    assert_eq!(FIT_XFADE, f(&fx["consts"]["xfade"]));
    let doc = json!({ "sample_rate": 48000 });
    for c in fx["fit"].as_array().unwrap() {
        let mut id = counter();
        let got = fit_ops(
            &doc,
            "tr",
            &c["clip"],
            &c["media"],
            &plan_of(&c["plan"]),
            &mut id,
        );
        ok(
            &got,
            &c["out"],
            &format!("fit {} {}", c["clip"], c["media"]),
        );
    }
    for c in fx["desc"].as_array().unwrap() {
        let p = plan_of(&c["p"]);
        assert_eq!(describe_plan(&p), c["plan"].as_str().unwrap(), "{c}");
        assert_eq!(
            describe_joins(&p, f(&c["origin"])),
            c["joins"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["same"].as_array().unwrap() {
        assert_eq!(
            same_length(f(&c["a"]), f(&c["b"])),
            c["out"].as_bool().unwrap()
        );
    }
}
