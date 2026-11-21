// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust audiobook delivery check must agree with `shared-fixtures/delivery.json`,
//! the same file the TypeScript tests read. The expected values were produced by
//! the TypeScript check, so the advice text must match word for word.

use altered_audio_editor_lib::delivery::{acx_check, Report};
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/delivery.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// JSON numbers 3 and 3.0 are different values to serde; make them all floats.
fn floats(v: Value) -> Value {
    match v {
        Value::Number(n) => serde_json::json!(n.as_f64().unwrap()),
        Value::Array(a) => Value::Array(a.into_iter().map(floats).collect()),
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, v)| (k, floats(v))).collect()),
        other => other,
    }
}

#[test]
fn acx_check_matches() {
    for c in fixture()["acxCheck"].as_array().unwrap() {
        let report = Report {
            rms: c["rms"].as_f64(),
            noise_floor: c["noiseFloor"].as_f64(),
            sample_peak: c["samplePeak"].as_f64(),
            silent_seconds: c["silentSeconds"].as_f64(),
        };
        let got = floats(serde_json::to_value(acx_check(&report)).unwrap());
        assert_eq!(got, floats(c["want"].clone()), "{}", c["name"]);
    }
}
