// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust take recorder, effect favorites and controls, and curve presets
//! against `shared-fixtures/auto-record.json`, `effect-ui.json` and
//! `curve-presets.json`, which the TypeScript tests read too.

use altered_audio_editor_lib::auto_record::*;
use altered_audio_editor_lib::curve_presets::*;
use altered_audio_editor_lib::effect_ui::*;
use altered_audio_editor_lib::envelope::Point;
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn near(a: f64, b: f64, what: &str) {
    let tol = 1e-9 * b.abs().max(1.0);
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b}");
}

fn points(v: &Value) -> Vec<Point> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| Point {
            position: f(&p["position"]),
            value: f(&p["value"]),
            curve: p["curve"].as_str().unwrap().to_string(),
        })
        .collect()
}

fn same_points(got: &[Point], want: &Value, what: &str) {
    let want = points(want);
    assert_eq!(got.len(), want.len(), "{what}: count");
    for (g, w) in got.iter().zip(&want) {
        near(g.position, w.position, what);
        near(g.value, w.value, what);
        assert_eq!(g.curve, w.curve, "{what}");
    }
}

#[test]
fn takes() {
    let fx = fixture("auto-record.json");
    for (n, take) in fx["takes"].as_array().unwrap().iter().enumerate() {
        let mut t = Take::new(f(&take["rate"]));
        let outs = take["out"].as_array().unwrap();
        for (i, s) in take["steps"].as_array().unwrap().iter().enumerate() {
            let what = format!("take {n} step {i}");
            let key = s["key"].as_str().unwrap_or("");
            let pos = s["pos"].as_f64().unwrap_or(0.0);
            let value = s["value"].as_f64().unwrap_or(0.0);
            match s["op"].as_str().unwrap() {
                "arm" => t.arm(
                    key,
                    RecordMode::parse(s["mode"].as_str().unwrap()).unwrap(),
                    &points(&s["base"]),
                    f(&s["rest"]),
                    f(&s["tol"]),
                    pos,
                    value,
                ),
                "grab" => t.grab(key, pos, value),
                "set" => t.set(key, pos, value),
                "release" => t.release(key, pos),
                "writing" => assert_eq!(t.writing(key), outs[i].as_bool().unwrap(), "{what}"),
                "has" => assert_eq!(t.has(key), outs[i].as_bool().unwrap(), "{what}"),
                "finish" => {
                    let got = t.finish(pos);
                    let want = outs[i].as_array().unwrap();
                    assert_eq!(got.len(), want.len(), "{what}: lanes");
                    for ((k, pts), w) in got.iter().zip(want) {
                        assert_eq!(k, w[0].as_str().unwrap(), "{what}");
                        same_points(pts, &w[1], &what);
                    }
                }
                other => panic!("unknown op {other}"),
            }
        }
    }
    for c in fx["tolerance"].as_array().unwrap() {
        near(
            tolerance_for(f(&c["top"]), f(&c["bottom"]), f(&c["share"])),
            f(&c["out"]),
            "tolerance",
        );
    }
}

fn param(v: &Value) -> Param {
    Param {
        key: v["key"].as_str().unwrap().to_string(),
        name: v["name"].as_str().unwrap().to_string(),
        unit: v["unit"].as_str().unwrap().to_string(),
        min: f(&v["min"]),
        max: f(&v["max"]),
        default: f(&v["default"]),
        step: f(&v["step"]),
        toggle: v["toggle"].as_bool().unwrap(),
        group: v["group"].as_str().map(str::to_string),
        hidden: v["hidden"].as_bool().unwrap_or(false),
    }
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn favorites() {
    let fx = fixture("effect-ui.json");
    assert_eq!(STARTER_FAVORITES.to_vec(), strings(&fx["starter"]));
    for c in fx["sanitize"].as_array().unwrap() {
        assert_eq!(
            sanitize_favorites(&c["raw"]),
            strings(&c["out"]),
            "{}",
            c["raw"]
        );
    }
    for c in fx["toggle"].as_array().unwrap() {
        assert_eq!(
            toggle_favorite(&strings(&c["list"]), c["id"].as_str().unwrap()),
            strings(&c["out"]),
            "{}",
            c["id"]
        );
    }
}

#[test]
fn controls() {
    let fx = fixture("effect-ui.json");
    for c in fx["columns"].as_array().unwrap() {
        let params: Vec<Param> = c["params"].as_array().unwrap().iter().map(param).collect();
        let got = control_columns(&params);
        let want = c["out"].as_array().unwrap();
        assert_eq!(got.len(), want.len());
        for (g, w) in got.iter().zip(want) {
            assert_eq!(g.name, w["name"].as_str().unwrap());
            assert_eq!(g.power.as_deref(), w["power"].as_str());
            assert_eq!(g.params, strings(&w["params"]));
        }
    }
    for c in fx["parse"].as_array().unwrap() {
        let got = parse_param(&param(&c["p"]), c["text"].as_str().unwrap());
        match c["out"].as_f64() {
            Some(w) => near(got.unwrap_or(f64::NAN), w, &c["text"].to_string()),
            None => assert!(got.is_none(), "{}", c["text"]),
        }
    }
    for c in fx["clamps"].as_array().unwrap() {
        let v = match c["v"].as_str() {
            Some("nan") => f64::NAN,
            Some("inf") => f64::INFINITY,
            _ => f(&c["v"]),
        };
        near(
            clamp_param(&param(&c["p"]), v),
            f(&c["out"]),
            &c["v"].to_string(),
        );
    }
}

fn segs(v: &Value) -> Vec<Seg> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| {
            let kind = match s[0].as_str().unwrap() {
                "c" => SegKind::C,
                "e" => SegKind::E,
                "n" => SegKind::N,
                _ => SegKind::X,
            };
            Seg(kind, f(&s[1]), f(&s[2]))
        })
        .collect()
}

fn same_params(got: &std::collections::BTreeMap<String, f64>, want: &Value, what: &str) {
    let want = want.as_object().unwrap();
    assert_eq!(got.len(), want.len(), "{what}: count");
    for (k, v) in want {
        near(
            *got.get(k).unwrap_or_else(|| panic!("{what}: {k}")),
            f(v),
            what,
        );
    }
}

#[test]
fn curves() {
    let fx = fixture("curve-presets.json");
    for c in fx["points"].as_array().unwrap() {
        let got = curve_points(&segs(&c["segs"]));
        let want = c["out"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{}", c["segs"]);
        for (g, w) in got.iter().zip(want) {
            near(g.0, f(&w[0]), "x");
            near(g.1, f(&w[1]), "y");
        }
    }
    for c in fx["custom"].as_array().unwrap() {
        let got = curve_params(f(&c["makeup"]), &segs(&c["segs"]), f(&c["knee"]));
        same_params(&got, &c["out"], "custom");
    }
    let table = builtin_curve_presets();
    let want = fx["presets"].as_array().unwrap();
    assert_eq!(table.len(), want.len());
    for ((name, makeup, s, knee), w) in table.iter().zip(want) {
        assert_eq!(*name, w["name"].as_str().unwrap());
        same_params(&curve_params(*makeup, s, *knee), &w["params"], name);
    }
}
