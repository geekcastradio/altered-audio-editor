// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust loudness matching rules must agree with
//! `shared-fixtures/loudness-match.json`, the same file the TypeScript tests
//! read. Numbers are compared to 1e-9; flags and labels exactly. Values JSON
//! cannot hold (NaN, infinities) are written as text.

use altered_audio_editor_lib::loudness_match::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/loudness-match.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn num(v: &Value) -> f64 {
    match v {
        Value::String(s) => match s.as_str() {
            "NaN" => f64::NAN,
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            other => panic!("unknown value {other}"),
        },
        v => v.as_f64().unwrap(),
    }
}

fn same(got: f64, want: &Value, what: &str, case: &Value) {
    let want = num(want);
    let ok = if want.is_nan() {
        got.is_nan()
    } else if want.is_infinite() {
        got == want
    } else {
        (got - want).abs() < 1e-9
    };
    assert!(ok, "{what}: got {got}, want {want}\n{case}");
}

fn item(fx: &Value) -> MatchItem {
    MatchItem {
        key: fx["item"]["key"].as_str().unwrap().into(),
        label: fx["item"]["label"].as_str().unwrap().into(),
        clip_ids: fx["item"]["clipIds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect(),
    }
}

fn reading(v: &Value) -> LoudnessReading {
    LoudnessReading {
        integrated: num(&v["integrated"]),
        integrated_mono: num(&v["integratedMono"]),
        true_peak: num(&v["truePeak"]),
    }
}

fn channel(v: &Value) -> ChannelLevels {
    ChannelLevels {
        peak_db: num(&v["peakDb"]),
        rms_db: num(&v["rmsDb"]),
    }
}

fn stereo(v: &Value) -> StereoLevels {
    StereoLevels {
        left: channel(&v["left"]),
        right: channel(&v["right"]),
    }
}

fn check_row(got: &MatchRow, want: &Value, case: &Value) {
    assert_eq!(got.item.key, want["key"].as_str().unwrap(), "{case}");
    assert_eq!(got.item.label, want["label"].as_str().unwrap(), "{case}");
    assert_eq!(
        got.item.clip_ids.len(),
        want["clipIds"].as_array().unwrap().len()
    );
    same(got.measured, &want["measured"], "measured", case);
    same(got.true_peak, &want["truePeak"], "truePeak", case);
    same(got.change, &want["change"], "change", case);
    same(got.peak_after, &want["peakAfter"], "peakAfter", case);
    assert_eq!(
        got.measurable,
        want["measurable"].as_bool().unwrap(),
        "{case}"
    );
    assert_eq!(
        got.peak_warning,
        want["peakWarning"].as_bool().unwrap(),
        "{case}"
    );
}

#[test]
fn constants_match() {
    let fx = fixture();
    let c = &fx["constants"];
    assert_eq!(GAIN_MIN, c["gainMin"].as_f64().unwrap());
    assert_eq!(GAIN_MAX, c["gainMax"].as_f64().unwrap());
    assert_eq!(DEFAULT_TOLERANCE, c["defaultTolerance"].as_f64().unwrap());
    assert_eq!(BASIS_LEVEL_PEAK, c["basisLevel"]["peak"].as_f64().unwrap());
    assert_eq!(
        BASIS_LEVEL_TRUE_PEAK,
        c["basisLevel"]["truepeak"].as_f64().unwrap()
    );
    assert_eq!(BASIS_LEVEL_RMS, c["basisLevel"]["rms"].as_f64().unwrap());
}

#[test]
fn change_for_matches() {
    for c in fixture()["changeFor"].as_array().unwrap() {
        let got = change_for(
            c["goal"].as_f64().unwrap(),
            c["measured"].as_f64().unwrap(),
            c["tolerance"].as_f64().unwrap(),
        );
        same(got, &c["want"], "change", c);
    }
}

#[test]
fn match_row_matches() {
    let fx = fixture();
    let it = item(&fx);
    for c in fx["matchRow"].as_array().unwrap() {
        let t = &c["target"];
        let target = LoudnessTarget {
            lufs: t["lufs"].as_f64().unwrap(),
            mono: t["mono"].as_bool().unwrap(),
            peak: t["peak"].as_f64().unwrap(),
        };
        let got = match_row(
            &it,
            &reading(&c["report"]),
            &target,
            c["tolerance"].as_f64().unwrap(),
        );
        check_row(&got, &c["want"], c);
    }
}

#[test]
fn levels_of_matches() {
    for c in fixture()["levelsOf"].as_array().unwrap() {
        let (peak, rms) = levels_of(&stereo(&c["report"]));
        same(peak, &c["want"]["peakDb"], "peakDb", c);
        same(rms, &c["want"]["rmsDb"], "rmsDb", c);
    }
}

#[test]
fn level_row_matches() {
    let fx = fixture();
    let it = item(&fx);
    for c in fx["levelRow"].as_array().unwrap() {
        let basis = match c["basis"].as_str().unwrap() {
            "peak" => LevelBasis::Peak,
            _ => LevelBasis::Rms,
        };
        let got = level_row(
            &it,
            basis,
            c["level"].as_f64().unwrap(),
            &stereo(&c["report"]),
            c["tolerance"].as_f64().unwrap(),
        );
        check_row(&got, &c["want"], c);
    }
}

#[test]
fn true_peak_row_matches() {
    let fx = fixture();
    let it = item(&fx);
    for c in fx["truePeakRow"].as_array().unwrap() {
        let got = true_peak_row(
            &it,
            c["level"].as_f64().unwrap(),
            &reading(&c["report"]),
            c["tolerance"].as_f64().unwrap(),
        );
        check_row(&got, &c["want"], c);
    }
}
