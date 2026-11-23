// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the chain actions in Rust against `shared-fixtures/chain-actions.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::chain_actions::*;
use altered_audio_editor_lib::clip_edit::LockedError;
use altered_audio_editor_lib::loudness_check::Report;
use altered_audio_editor_lib::quickpass_scan::ClipLevels;
use serde_json::{json, Value};
use std::collections::HashSet;

const LOCKED: &str = "A locked clip is in the way. Unlock it first (Clip > Lock Position).";

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/chain-actions.json");
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

/// A number from the fixture, where JSON turned an infinity into null.
fn g(v: &Value) -> f64 {
    v.as_f64().unwrap_or(f64::NEG_INFINITY)
}

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("n{n}0")
    }
}

fn step(s: (Option<Value>, String)) -> Value {
    json!({ "op": s.0, "message": s.1 })
}

fn err(m: &str) -> Value {
    json!({ "error": m, "locked": false })
}

#[test]
fn loudness_and_mic() {
    let fx = fixture();
    for c in fx["lt"].as_array().unwrap() {
        let r = &c["report"];
        let report = Report {
            integrated: g(&r["integrated"]),
            integrated_mono: g(&r["integratedMono"]),
            true_peak: g(&r["truePeak"]),
        };
        let got = step(loudness_target_op(
            report,
            f(&c["master"]),
            c["target"].as_str().unwrap(),
            f(&c["tol"]),
            "europe",
            (-18.0, -1.0),
        ));
        ok(&got, &c["out"], &format!("loudness {c}"));
    }
    for c in fx["mic"].as_array().unwrap() {
        ok(&mic_options(&c["p"]), &c["out"], &format!("mic {}", c["p"]));
    }
}

#[test]
fn voice_chain_presets() {
    let fx = fixture();
    let saved = fx["saved"].as_array().unwrap().clone();
    for c in fx["vc"].as_array().unwrap() {
        let known: HashSet<String> = fx["knowns"][c["ki"].as_u64().unwrap() as usize]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect();
        let got = match voice_chain_preset_op(
            &fx["doc"],
            c["preset"].as_str().unwrap(),
            c["which"].as_str().unwrap(),
            &saved,
            &known,
            &mut counter(),
        ) {
            Ok(s) => step(s),
            Err(e) => err(&e),
        };
        ok(&got, &c["out"], &format!("voice chain {c}"));
    }
}

#[test]
fn quickpass_steps() {
    let fx = fixture();
    let doc = &fx["doc"];
    let (ids, info) = quick_pass_clips(doc);
    let want: Vec<String> = fx["clips"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c[0].as_str().unwrap().to_string())
        .collect();
    assert_eq!(ids, want);
    for c in fx["clips"].as_array().unwrap() {
        assert_eq!(
            info[c[0].as_str().unwrap()],
            (
                c[1].as_str().unwrap().to_string(),
                c[2].as_str().unwrap().to_string()
            )
        );
    }
    let levels: Vec<ClipLevels> = fx["levels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| ClipLevels {
            clip_id: l["clipId"].as_str().unwrap().to_string(),
            start: f(&l["start"]),
            window: f(&l["window"]),
            db: l["db"].as_array().unwrap().iter().map(f).collect(),
            peak: l["peak"].as_array().unwrap().iter().map(f).collect(),
        })
        .collect();
    for c in fx["qs"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        match quick_pass_settings(name, c["ripple"].as_str().unwrap()) {
            Err(e) => ok(&err(&e), &c["settings"], &format!("settings {c}")),
            Ok(s) => {
                ok(&s.to_json(), &c["settings"], &format!("settings {name}"));
                let got = match quick_pass_op(doc, &levels, &s, &mut counter()) {
                    Ok(x) => step(x),
                    Err(LockedError) => json!({ "error": LOCKED, "locked": true }),
                };
                ok(&got, &c["op"], &format!("quickpass {name} {}", c["ripple"]));
            }
        }
    }
}
