// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust copies of the pitch edit choices, the welcome plan and the
//! bug report text against `shared-fixtures/app-rules.json`, which the
//! TypeScript tests read too.

use altered_audio_editor_lib::bug_info::{self, Context, RecordedError};
use altered_audio_editor_lib::pitch_edit::{self, Edit};
use altered_audio_editor_lib::welcome::{self, PlanKind};
use serde_json::Value;

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/app-rules.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn near(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-9, "{what}: {a} vs {b}");
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn pitch_edit_matches() {
    for case in fixture()["pitch"].as_array().unwrap() {
        let c = pitch_edit::sanitize_choices(&case["raw"]);
        let e = &case["c"];
        let what = case["raw"].to_string();
        assert_eq!(c.bend, e["mode"] == "bend", "{what}");
        near(c.key, f(&e["key"]), &what);
        assert_eq!(c.scale, e["scale"].as_str().unwrap(), "{what}");
        near(c.strength, f(&e["strength"]), &what);
        near(c.speed_ms, f(&e["speedMs"]), &what);
        near(c.max_cents, f(&e["maxCents"]), &what);
        near(c.bend_start, f(&e["bendStart"]), &what);
        near(c.bend_end, f(&e["bendEnd"]), &what);
        match (&e["bendMiddle"], c.bend_middle) {
            (Value::Null, None) => {}
            (v, Some(m)) => near(m, f(v), &what),
            other => panic!("{what}: middle {other:?}"),
        }
        assert_eq!(
            pitch_edit::is_no_change(&c),
            case["noChange"].as_bool().unwrap(),
            "{what}"
        );
        let ex = &case["edit"];
        match pitch_edit::edit_for(&c) {
            Edit::Tune {
                key,
                scale,
                strength,
                speed_ms,
                max_cents,
            } => {
                assert_eq!(ex["kind"], "tune", "{what}");
                near(key, f(&ex["key"]), &what);
                assert_eq!(scale, ex["scale"].as_str().unwrap());
                near(strength, f(&ex["strength"]), &what);
                near(speed_ms, f(&ex["speedMs"]), &what);
                near(max_cents, f(&ex["maxCents"]), &what);
            }
            Edit::Bend { points } => {
                assert_eq!(ex["kind"], "bend", "{what}");
                let want = ex["points"].as_array().unwrap();
                assert_eq!(points.len(), want.len(), "{what}");
                for (p, w) in points.iter().zip(want) {
                    near(p.0, f(&w[0]), &what);
                    near(p.1, f(&w[1]), &what);
                }
            }
        }
    }
}

#[test]
fn report_text_matches() {
    for case in fixture()["reports"].as_array().unwrap() {
        let got =
            pitch_edit::describe_report(f(&case["frames"]), f(&case["voiced"]), f(&case["moved"]));
        assert_eq!(got, case["text"].as_str().unwrap(), "{case}");
    }
}

#[test]
fn welcome_plan_matches() {
    for case in fixture()["welcome"].as_array().unwrap() {
        let revs: Vec<f64> = case["revs"].as_array().unwrap().iter().map(f).collect();
        let got = welcome::welcome_plan(Some(&case["saved"]), &revs);
        match (&case["expect"], got) {
            (Value::Null, None) => {}
            (e, Some((kind, idx))) => {
                let k = if kind == PlanKind::Install {
                    "install"
                } else {
                    "upgrade"
                };
                assert_eq!(e["kind"], k, "{case}");
                let want: Vec<usize> = e["idx"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_u64().unwrap() as usize)
                    .collect();
                assert_eq!(idx, want, "{case}");
            }
            other => panic!("{case}: {other:?}"),
        }
    }
}

#[test]
fn scrub_matches() {
    for case in fixture()["scrubs"].as_array().unwrap() {
        assert_eq!(
            bug_info::scrub(case["in"].as_str().unwrap()),
            case["out"].as_str().unwrap(),
            "{case}"
        );
    }
}

#[test]
fn recorded_errors_match() {
    let mut list = Vec::new();
    for case in fixture()["records"].as_array().unwrap() {
        bug_info::record_error(
            &mut list,
            "t",
            case["message"].as_str().unwrap(),
            case["stack"].as_str().unwrap(),
        );
        let last = list.last().unwrap();
        assert_eq!(last.message, case["msg"].as_str().unwrap());
        assert_eq!(last.stack, case["stk"].as_str().unwrap());
    }
    for i in 0..30 {
        bug_info::record_error(&mut list, &i.to_string(), "m", "");
    }
    assert_eq!(list.len(), bug_info::KEEP);
    assert_eq!(list.last().unwrap().at, "29");
}

#[test]
fn bug_text_matches() {
    for case in fixture()["bug"].as_array().unwrap() {
        let c = &case["ctx"];
        let (state, plugins, audio, startup) = (
            strings(&c["state"]),
            strings(&c["plugins"]),
            strings(&c["audio"]),
            strings(&c["startup"]),
        );
        let ctx = Context {
            app_version: c["appVersion"].as_str().unwrap(),
            system: c["system"].as_str().unwrap(),
            state: &state,
            plugins: &plugins,
            audio: &audio,
            startup: &startup,
        };
        let list: Vec<RecordedError> = case["list"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| RecordedError {
                at: e["at"].as_str().unwrap().into(),
                message: e["message"].as_str().unwrap().into(),
                stack: e["stack"].as_str().unwrap().into(),
            })
            .collect();
        assert_eq!(
            bug_info::bug_info_text(&ctx, &list),
            case["out"].as_str().unwrap()
        );
    }
}
