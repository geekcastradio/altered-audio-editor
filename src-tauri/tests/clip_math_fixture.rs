// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust clip number rules against `shared-fixtures/clip-math.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::clip_math::*;
use altered_audio_editor_lib::tempo_map::Tempo;
use serde_json::Value;

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/clip-math.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn list(v: &Value) -> Vec<f64> {
    v.as_array().unwrap().iter().map(f).collect()
}

#[test]
fn constants_and_shapes() {
    let fx = fixture();
    assert_eq!(MIN_CLIP, f(&fx["min"]));
    assert_eq!(TOP_LAYER, f(&fx["top"]));
    let shapes = fx["shapes"].as_array().unwrap();
    assert_eq!(shapes.len(), FADE_SHAPES.len());
    for (s, (id, label)) in shapes.iter().zip(FADE_SHAPES) {
        assert_eq!(s["id"].as_str().unwrap(), id);
        assert_eq!(s["label"].as_str().unwrap(), label);
    }
}

#[test]
fn lengths_and_ends() {
    for c in fixture()["lengths"].as_array().unwrap() {
        let span = ClipSpan {
            start: f(&c["start"]),
            source_in: f(&c["source_in"]),
            source_out: f(&c["source_out"]),
            looped: c["looped"].as_f64(),
            stretch: c["stretch"].as_f64(),
        };
        let rate = c["rate"].as_f64();
        let session = f(&c["session"]);
        assert_eq!(
            clip_length(&span, rate, session),
            f(&c["length"]),
            "length {c}"
        );
        assert_eq!(clip_end(&span, rate, session), f(&c["end"]), "end {c}");
        assert_eq!(
            stretch_of(span.looped.is_some(), span.stretch),
            f(&c["ratio"]),
            "ratio {c}"
        );
    }
}

#[test]
fn midi_spans() {
    for c in fixture()["spans"].as_array().unwrap() {
        let tempo: Option<Tempo> =
            (!c["tempo"].is_null()).then(|| serde_json::from_value(c["tempo"].clone()).unwrap());
        let (a, b) = midi_span(
            tempo.as_ref(),
            f(&c["rate"]),
            f(&c["start"]),
            f(&c["length"]),
        );
        assert_eq!(vec![a, b], list(&c["out"]), "span {c}");
    }
}

#[test]
fn snapping() {
    let fx = fixture();
    for c in fx["near"].as_array().unwrap() {
        let got = nearest(f(&c["v"]), &list(&c["targets"]), f(&c["tol"]));
        assert_eq!(got, c["out"].as_f64(), "nearest {c}");
    }
    for c in fx["snaps"].as_array().unwrap() {
        let markers: Vec<(f64, f64)> = c["markers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| (f(&m["position"]), f(&m["duration"])))
            .collect();
        assert_eq!(
            snap_to_markers(f(&c["pos"]), &markers, f(&c["tol"])),
            f(&c["out"]),
            "snap {c}"
        );
    }
}

#[test]
fn fade_shapes() {
    let fade = |v: &Value| Fade {
        length: f(&v["length"]),
        shape: v["shape"].as_str().map(String::from),
    };
    for c in fixture()["fades"].as_array().unwrap() {
        let (a, b) = fade_shape_change(
            &fade(&c["fin"]),
            &fade(&c["fout"]),
            c["shape"].as_str().unwrap(),
        );
        assert_eq!(
            vec![a, b],
            c["out"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_bool().unwrap())
                .collect::<Vec<_>>(),
            "fade {c}"
        );
    }
}
