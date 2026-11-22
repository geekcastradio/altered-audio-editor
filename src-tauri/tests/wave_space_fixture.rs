// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks Waveform file rows and the 3D Space room maths in Rust against
//! `shared-fixtures/wave-space.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::space_ui::*;
use altered_audio_editor_lib::wave_rows::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/wave-space.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn near(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() <= 1e-9 * (1.0 + a.abs().max(b.abs())),
        "{what}: got {a}, want {b}"
    );
}

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

fn pair(v: &Value) -> (f64, f64) {
    (f(&v[0]), f(&v[1]))
}

#[test]
fn file_rows() {
    for c in fixture()["rows"].as_array().unwrap() {
        let a = &c["args"];
        let current = (!a[0].is_null()).then(|| CurrentFile {
            path: a[0]["path"].as_str().unwrap().into(),
            format: a[0]["format"].as_str().unwrap().into(),
            untitled: a[0]["untitled"].as_bool().unwrap(),
        });
        let parked: Vec<ParkedFile> = a[2]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| ParkedFile {
                path: p["path"].as_str().unwrap().into(),
                format: p["format"].as_str().unwrap().into(),
                untitled: p["untitled"].as_bool().unwrap(),
                dirty: p["dirty"].as_bool().unwrap(),
            })
            .collect();
        let listed: Vec<String> = a[3]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().into())
            .collect();
        let rows = wave_rows(current.as_ref(), a[1].as_bool().unwrap(), &parked, &listed);
        let got: Vec<Value> = rows
            .iter()
            .map(|r| json!({ "path": r.path, "name": r.name, "format": r.format, "untitled": r.untitled, "dirty": r.dirty, "front": r.front }))
            .collect();
        assert_eq!(Value::Array(got), c["out"], "{a}");
    }
}

#[test]
fn layouts_and_positions() {
    let fx = fixture();
    for c in fx["layouts"].as_array().unwrap() {
        let l = c["l"].as_str().unwrap();
        let spots: Vec<Value> = speaker_spots(l)
            .iter()
            .map(|s| json!({ "name": s.name, "x": s.x, "y": s.y, "z": s.z }))
            .collect();
        assert!(same(&Value::Array(spots), &c["spots"]), "{l}");
        assert_eq!(layout_name(l).unwrap(), c["name"].as_str().unwrap());
        assert_eq!(json!(channel_names(l)), c["channels"]);
    }
    assert_eq!(LAYOUTS.len(), fx["layouts"].as_array().unwrap().len());
    for c in fx["tidy"].as_array().unwrap() {
        let p = (!c["p"].is_null()).then_some(&c["p"]);
        assert!(same(&tidy_pos(p), &c["out"]), "tidy {c}");
    }
}

#[test]
fn room_maths() {
    let fx = fixture();
    for c in fx["views"].as_array().unwrap() {
        let (size, x, y) = (f(&c["size"]), f(&c["x"]), f(&c["y"]));
        let (tx, ty) = to_view(x, y, size);
        near(tx, f(&c["to"][0]), "to x");
        near(ty, f(&c["to"][1]), "to y");
        let (bx, by) = from_view(tx, ty, size);
        near(bx, f(&c["back"][0]), "back x");
        near(by, f(&c["back"][1]), "back y");
        let (ox, oy) = from_view(x * 300.0, y * 300.0 - 20.0, size);
        near(ox, f(&c["out"][0]), "out x");
        near(oy, f(&c["out"][1]), "out y");
    }
    for c in fx["sides"].as_array().unwrap() {
        let got = sides(f(&c["x"]), f(&c["y"]), f(&c["width"]), f(&c["rotation"]));
        for (g, w) in got.iter().zip(c["out"].as_array().unwrap()) {
            near(g.0, f(&w[0]), "side x");
            near(g.1, f(&w[1]), "side y");
        }
    }
    for c in fx["deg"].as_array().unwrap() {
        near(wrap_deg(f(&c["d"])), f(&c["wrap"]), &format!("wrap {c}"));
        near(snap_angle(f(&c["d"])), f(&c["snap"]), &format!("snap {c}"));
    }
    for c in fx["snapDegs"].as_array().unwrap() {
        near(snap_angle(f(&c["d"])), f(&c["snap"]), "snap angle");
    }
    for c in fx["angles"].as_array().unwrap() {
        near(
            angle_of(f(&c["cx"]), f(&c["cy"]), f(&c["x"]), f(&c["y"])),
            f(&c["out"]),
            "angle",
        );
    }
    let spk: Vec<(f64, f64)> = fx["spk"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (f(&s["x"]), f(&s["y"])))
        .collect();
    for c in fx["snaps"].as_array().unwrap() {
        let tol = c["tol"].as_f64().unwrap_or(SNAP_TOL);
        let (x, y) = snap_spot(f(&c["x"]), f(&c["y"]), &spk, tol);
        let want = pair(&c["out"]);
        near(x, want.0, "snap x");
        near(y, want.1, "snap y");
    }
}
