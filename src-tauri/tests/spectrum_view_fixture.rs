// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the spectrum view axis, zoom, scale marks, colors and selection
//! masks in Rust against `shared-fixtures/spectrum-view.json`, which the
//! TypeScript test reads too.

use altered_audio_editor_lib::spectrum_view::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/spectrum-view.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
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

fn ok(a: &Value, b: &Value, what: &str) {
    assert!(same(a, b), "{what}\n got {a}\nwant {b}");
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn view(fx: &Value, c: &Value) -> FreqView {
    serde_json::from_value(fx["views"][c["vi"].as_u64().unwrap() as usize].clone()).unwrap()
}

#[test]
fn axis_and_view() {
    let fx = fixture();
    for c in fx["axis"].as_array().unwrap() {
        let (ny, x) = (f(&c["ny"]), f(&c["x"]));
        ok(&json!(freq_at(x, ny)), &c["at"], &format!("freq at {c}"));
        ok(&json!(frac_of(x, ny)), &c["of"], &format!("frac of {c}"));
    }
    for c in fx["range"].as_array().unwrap() {
        let (lo, hi) = view_range(&view(&fx, c), f(&c["ny"]));
        ok(&json!([lo, hi]), &c["out"], &format!("range {c}"));
    }
    for c in fx["vf"].as_array().unwrap() {
        ok(
            &json!(view_freq(f(&c["x"]), &view(&fx, c), f(&c["ny"]))),
            &c["out"],
            &format!("view freq {c}"),
        );
    }
    for c in fx["vfrac"].as_array().unwrap() {
        ok(
            &json!(view_frac(f(&c["x"]), &view(&fx, c), f(&c["ny"]))),
            &c["out"],
            &format!("view frac {c}"),
        );
    }
    for c in fx["edges"].as_array().unwrap() {
        let rows = c["rows"].as_u64().unwrap() as usize;
        ok(
            &json!(row_edges(rows, &view(&fx, c), f(&c["ny"]))),
            &c["out"],
            &format!("edges {c}"),
        );
    }
}

#[test]
fn zoom_ticks_and_colors() {
    let fx = fixture();
    for c in fx["zoom"].as_array().unwrap() {
        let got = zoom_freq(&view(&fx, c), f(&c["hz"]), f(&c["f"]), f(&c["ny"]));
        ok(&json!(got), &c["out"], &format!("zoom {c}"));
    }
    for c in fx["ticks"].as_array().unwrap() {
        let got = freq_ticks(&view(&fx, c), f(&c["ny"]), f(&c["h"]), f(&c["gap"]));
        ok(&json!(got), &c["out"], &format!("ticks {c}"));
    }
    ok(&json!(spectrum_lut()), &fx["lut"], "color table");
}

#[test]
fn selection_masks() {
    let fx = fixture();
    let sets = fx["shapes"].as_array().unwrap();
    for (n, set) in sets.iter().enumerate() {
        let shapes: Vec<SpectralShape> = serde_json::from_value(set.clone()).unwrap();
        ok(
            &json!(shapes_bounds(&shapes)),
            &fx["bounds"][n],
            &format!("bounds {n}"),
        );
        ok(
            &json!(rasterize(&shapes, 800, 300)),
            &fx["raster"][n],
            &format!("mask {n}"),
        );
        ok(
            &json!(rasterize(&shapes, 8, 6)),
            &fx["rasterSmall"][n],
            &format!("small mask {n}"),
        );
    }
}
