// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust Speaker Balance arithmetic must agree with
//! `shared-fixtures/speaker-balance.json`, the same file the TypeScript tests
//! read. Numbers are compared to 1e-9; flags and text exactly. NaN and the
//! infinities are written as text.

use altered_audio_editor_lib::loudness_match::MatchItem;
use altered_audio_editor_lib::speaker_balance::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/speaker-balance.json");
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

#[test]
fn balance_matches() {
    for c in fixture()["balance"].as_array().unwrap() {
        let items: Vec<MatchItem> = c["labels"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, l)| MatchItem {
                key: format!("t{i}"),
                label: l.as_str().unwrap().into(),
                clip_ids: vec![format!("c{i}")],
            })
            .collect();
        let reports: Vec<Option<BalanceReading>> = c["reports"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (!r.is_null()).then(|| BalanceReading {
                    integrated: num(&r["integrated"]),
                    true_peak: num(&r["truePeak"]),
                })
            })
            .collect();
        let got = balance(&items, &reports);
        let want = &c["want"];
        let rows = want["rows"].as_array().unwrap();
        assert_eq!(got.rows.len(), rows.len(), "{c}");
        for (g, w) in got.rows.iter().zip(rows) {
            same(g.measured, &w["measured"], "measured", c);
            same(g.true_peak, &w["truePeak"], "truePeak", c);
            same(g.change, &w["change"], "change", c);
            same(g.peak_after, &w["peakAfter"], "peakAfter", c);
            assert_eq!(g.measurable, w["measurable"].as_bool().unwrap(), "{c}");
        }
        assert_eq!(
            got.reference.map(|i| i as u64),
            want["reference"].as_u64(),
            "{c}"
        );
        assert_eq!(
            describe_balance(&got),
            want["text"].as_str().unwrap(),
            "{c}"
        );
    }
}
