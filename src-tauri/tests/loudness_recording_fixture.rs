// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust Check Loudness and recording rules must agree with
//! `shared-fixtures/loudness-recording.json`, the same file the TypeScript
//! tests read. Non-finite numbers are text in the file.

use altered_audio_editor_lib::loudness_check::*;
use altered_audio_editor_lib::recording::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/loudness-recording.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    match v.as_str() {
        Some("Infinity") => f64::INFINITY,
        Some("-Infinity") => f64::NEG_INFINITY,
        Some("NaN") => f64::NAN,
        _ => v.as_f64().unwrap(),
    }
}

fn same(a: f64, b: f64) -> bool {
    (a.is_nan() && b.is_nan()) || a == b || (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

fn report(v: &Value) -> Report {
    Report {
        integrated: f(&v["integrated"]),
        integrated_mono: f(&v["integratedMono"]),
        true_peak: f(&v["truePeak"]),
    }
}

fn target(v: &Value) -> Target {
    Target {
        id: v["id"].as_str().unwrap().to_string(),
        name: v["name"].as_str().unwrap().to_string(),
        lufs: f(&v["lufs"]),
        mono: v["mono"].as_bool().unwrap(),
        peak: f(&v["peak"]),
    }
}

#[test]
fn regions_and_targets_match() {
    let fx = fixture();
    for c in fx["regionForZone"].as_array().unwrap() {
        assert_eq!(
            region_for_zone(c["zone"].as_str()),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["targets"].as_array().unwrap() {
        let got = targets_for(
            c["region"].as_str().unwrap(),
            f(&c["custom"]["lufs"]),
            f(&c["custom"]["peak"]),
        );
        let want: Vec<Target> = c["want"].as_array().unwrap().iter().map(target).collect();
        assert_eq!(got, want, "{c}");
    }
}

#[test]
fn verdicts_and_predictions_match() {
    let fx = fixture();
    for c in fx["evaluate"].as_array().unwrap() {
        let got = evaluate(report(&c["report"]), &target(&c["target"]));
        let w = &c["want"];
        assert!(
            same(got.measured, f(&w["measured"])) && same(got.offset, f(&w["offset"])),
            "{c}"
        );
        assert!(same(got.gain_to_target, f(&w["gainToTarget"])), "{c}");
        assert_eq!(
            got.loudness.as_str(),
            w["loudness"].as_str().unwrap(),
            "{c}"
        );
        assert_eq!(got.peak.as_str(), w["peak"].as_str().unwrap(), "{c}");
        assert_eq!(got.gain_is_safe, w["gainIsSafe"].as_bool().unwrap(), "{c}");
        assert_eq!(got.advice, w["advice"].as_str().unwrap(), "{c}");
    }
    for c in fx["predict"].as_array().unwrap() {
        let t = if c["target"].is_null() {
            None
        } else {
            Some(target(&c["target"]))
        };
        let got = predict_export(
            report(&c["report"]),
            t.as_ref(),
            c["mono"].as_bool().unwrap(),
        );
        let w = &c["want"];
        assert!(
            same(got.gain, f(&w["gain"]))
                && same(got.loudness, f(&w["loudness"]))
                && same(got.true_peak, f(&w["truePeak"])),
            "{c}"
        );
        assert_eq!(
            got.limited_by_peak,
            w["limitedByPeak"].as_bool().unwrap(),
            "{c}"
        );
        match w["offset"].as_f64() {
            None => assert!(got.offset.is_none(), "{c}"),
            Some(o) => assert!(same(got.offset.unwrap(), o), "{c}"),
        }
        assert_eq!(got.grade.as_str(), w["grade"].as_str().unwrap(), "{c}");
        assert_eq!(got.summary, w["summary"].as_str().unwrap(), "{c}");
    }
}

fn plan(v: &Value) -> RecordPlan {
    let punch = v["punch"]
        .as_object()
        .map(|p| (f(&p["start"]), f(&p["end"])));
    RecordPlan {
        at: f(&v["at"]),
        punch,
    }
}

fn options(v: &Value) -> RecordOptions {
    RecordOptions {
        offset_ms: f(&v["offsetMs"]),
        punch: v["punch"].as_bool().unwrap(),
        preroll_sec: f(&v["prerollSec"]),
        replace: v["replace"].as_bool().unwrap(),
    }
}

#[test]
fn recording_matches() {
    let fx = fixture();
    for c in fx["sanitize"].as_array().unwrap() {
        assert_eq!(
            sanitize_record_options(&c["raw"]),
            options(&c["want"]),
            "{c}"
        );
    }
    for c in fx["recordPlan"].as_array().unwrap() {
        let sel = c["sel"].as_object().map(|s| (f(&s["start"]), f(&s["end"])));
        let got = record_plan(f(&c["playhead"]), sel, options(&c["o"]), f(&c["rate"]));
        assert_eq!(got, plan(&c["want"]), "{c}");
    }
    for c in fx["punchStop"].as_array().unwrap() {
        let got = punch_stop_after(plan(&c["plan"]), f(&c["rate"]));
        match c["want"].as_f64() {
            None => assert!(got.is_none(), "{c}"),
            Some(w) => assert!(same(got.unwrap(), w), "{c}"),
        }
    }
    for c in fx["placeTake"].as_array().unwrap() {
        let got = place_take(
            plan(&c["plan"]),
            f(&c["take"]["length"]),
            f(&c["take"]["sampleRate"]),
            f(&c["sessionRate"]),
            f(&c["offsetMs"]),
        );
        match c["want"].as_object() {
            None => assert!(got.is_none(), "{c}"),
            Some(w) => {
                let g = got.expect("a placement");
                assert!(
                    same(g.start, f(&w["start"]))
                        && same(g.source_in, f(&w["sourceIn"]))
                        && same(g.source_out, f(&w["sourceOut"])),
                    "{c}"
                );
            }
        }
    }
}
