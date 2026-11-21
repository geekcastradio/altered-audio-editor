// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust time helpers must agree with `shared-fixtures/time.json`, the same
//! file the TypeScript tests read. Seconds are compared to 1e-9; labels exactly.

use altered_audio_editor_lib::time::*;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/time.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn format_decimal_matches() {
    for c in fixture()["formatDecimal"].as_array().unwrap() {
        let got = format_decimal(c["samples"].as_f64().unwrap(), c["rate"].as_f64().unwrap());
        assert_eq!(got.unwrap(), c["want"].as_str().unwrap(), "{c}");
    }
}

#[test]
fn samples_to_seconds_matches() {
    for c in fixture()["samplesToSeconds"].as_array().unwrap() {
        let got = samples_to_seconds(c["samples"].as_f64().unwrap(), c["rate"].as_f64().unwrap());
        assert!(
            (got.unwrap() - c["want"].as_f64().unwrap()).abs() < 1e-9,
            "{c}"
        );
    }
}

#[test]
fn bad_rates_match() {
    for c in fixture()["badRates"].as_array().unwrap() {
        let rate = match &c["rate"] {
            Value::String(s) => match s.as_str() {
                "NaN" => f64::NAN,
                "Infinity" => f64::INFINITY,
                _ => f64::NEG_INFINITY,
            },
            v => v.as_f64().unwrap(),
        };
        assert_eq!(
            samples_to_seconds(10.0, rate).is_err(),
            c["threw"].as_bool().unwrap(),
            "{c}"
        );
    }
}
