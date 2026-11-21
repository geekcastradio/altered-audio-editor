// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust analysis formats and scales must agree with
//! `shared-fixtures/analysis.json`, the same file the TypeScript tests read.
//! Numbers are compared to 1e-9; text exactly.

use altered_audio_editor_lib::analysis::*;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/analysis.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn close(got: f64, want: &Value, case: &Value) {
    assert!((got - want.as_f64().unwrap()).abs() < 1e-9, "{case}: {got}");
}

/// A value the fixture wrote as a number, or as the text of one that JSON cannot hold.
fn number(v: &Value) -> Option<f64> {
    match v {
        Value::Null => None,
        Value::String(s) => match s.as_str() {
            "NaN" => Some(f64::NAN),
            "Infinity" => Some(f64::INFINITY),
            "-Infinity" => Some(f64::NEG_INFINITY),
            "undefined" => None,
            other => panic!("unknown value {other}"),
        },
        v => Some(v.as_f64().unwrap()),
    }
}

#[test]
fn fmt_db_matches() {
    for c in fixture()["fmtDb"].as_array().unwrap() {
        let got = fmt_db(number(&c["v"]), c["floor"].as_f64().unwrap());
        assert_eq!(got, c["want"].as_str().unwrap(), "{c}");
    }
}

#[test]
fn text_formats_match() {
    let fx = fixture();
    for c in fx["fmtSample"].as_array().unwrap() {
        assert_eq!(
            fmt_sample(c["v"].as_f64().unwrap()),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["fmtHz"].as_array().unwrap() {
        assert_eq!(
            fmt_hz(c["hz"].as_f64().unwrap()),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["fmtAt"].as_array().unwrap() {
        assert_eq!(
            fmt_at(c["seconds"].as_f64().unwrap()),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn scales_match() {
    let fx = fixture();
    for c in fx["logX"].as_array().unwrap() {
        let got = log_x(
            c["hz"].as_f64().unwrap(),
            c["low"].as_f64().unwrap(),
            c["high"].as_f64().unwrap(),
        );
        close(got, &c["want"], c);
    }
    for c in fx["dbY"].as_array().unwrap() {
        let got = db_y(
            c["db"].as_f64().unwrap(),
            c["top"].as_f64().unwrap(),
            c["bottom"].as_f64().unwrap(),
        );
        close(got, &c["want"], c);
    }
    for c in fx["binDb"].as_array().unwrap() {
        close(bin_db(c["i"].as_u64().unwrap() as usize), &c["want"], c);
    }
    let ticks: Vec<f64> = fx["freqTicks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    assert_eq!(ticks, FREQ_TICKS.to_vec());
}

#[test]
fn histogram_bars_match() {
    for c in fixture()["histogramBars"].as_array().unwrap() {
        let bins: Vec<f64> = c["bins"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let got = histogram_bars(&bins);
        let want = c["want"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{c}");
        for (g, w) in got.iter().zip(want) {
            close(*g, w, c);
        }
    }
}

#[test]
fn hum_cut_fix_matches() {
    for c in fixture()["humCutFix"].as_array().unwrap() {
        let h = &c["hum"];
        let hum = Hum {
            base_hz: h["baseHz"].as_f64().unwrap(),
            strength_db: h["strengthDb"].as_f64().unwrap(),
            tones: h["tones"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| HumTone {
                    hz: t["hz"].as_f64().unwrap(),
                    excess_db: t["excessDb"].as_f64().unwrap(),
                })
                .collect(),
        };
        let got = hum_cut_fix(&hum);
        let want = &c["want"];
        assert_eq!(got.label, want["label"].as_str().unwrap(), "{c}");
        assert_eq!(got.effect, want["effect"].as_str().unwrap(), "{c}");
        close(got.mains, &want["params"]["mains"], c);
        close(got.harmonics, &want["params"]["harmonics"], c);
    }
}
