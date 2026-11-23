// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the QuickPass scanning rules in Rust against
//! `shared-fixtures/quickpass-scan.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::quickpass_scan::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/quickpass-scan.json");
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

fn nums(v: &Value) -> Vec<f64> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect()
}

fn levels(fx: &Value) -> Vec<ClipLevels> {
    fx["lv"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| ClipLevels {
            clip_id: l["clipId"].as_str().unwrap().into(),
            start: l["start"].as_f64().unwrap(),
            window: l["window"].as_f64().unwrap(),
            db: nums(&l["db"]),
            peak: nums(&l["peak"]),
        })
        .collect()
}

fn tracks(fx: &Value) -> ClipInfo {
    fx["tracks"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                (
                    v["trackId"].as_str().unwrap().into(),
                    v["name"].as_str().unwrap().into(),
                ),
            )
        })
        .collect()
}

fn runs(r: Vec<(usize, usize)>) -> Value {
    json!(r.iter().map(|(a, b)| json!([a, b])).collect::<Vec<_>>())
}

fn findings(f: Vec<Finding>) -> Value {
    Value::Array(f.iter().map(Finding::to_json).collect())
}

#[test]
fn settings() {
    let fx = fixture();
    for c in fx["sanitize"].as_array().unwrap() {
        ok(
            &sanitize_settings(&c["raw"]).to_json(),
            &c["out"],
            &format!("sanitize {}", c["raw"]),
        );
    }
    ok(&defaults().to_json(), &data()["defaults"], "defaults");
}

#[test]
fn scans() {
    let fx = fixture();
    let (lv, map) = (levels(&fx), tracks(&fx));
    for c in fx["scans"].as_array().unwrap() {
        let s = Settings::from_json(&c["s"]);
        ok(
            &findings(find_all(&lv, &map, &s, 44100.0)),
            &c["out"],
            &format!("find_all {}", c["s"]),
        );
        ok(
            &runs(silent_runs(&lv[0].db, 0.01, &s)),
            &c["runs"]["silent"],
            "silent",
        );
        ok(
            &runs(noise_runs(&lv[1].db, 0.02, &s)),
            &c["runs"]["noise"],
            "noise",
        );
        ok(
            &runs(sound_runs(&lv[2].db, 0.01, &s)),
            &c["runs"]["sound"],
            "sound",
        );
        ok(
            &runs(clip_runs(&lv[0].peak, &s)),
            &c["runs"]["clip"],
            "clip",
        );
        ok(&spots_request(&s), &c["req"], "request");
    }
}

#[test]
fn spots_levels_and_labels() {
    let fx = fixture();
    let map = tracks(&fx);
    let spots: Vec<ClipSpots> = fx["spots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| ClipSpots {
            clip_id: c["clipId"].as_str().unwrap().into(),
            spans: c["spans"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| (p[0].as_f64().unwrap(), p[1].as_f64().unwrap()))
                .collect(),
        })
        .collect();
    for c in fx["spotOut"].as_array().unwrap() {
        ok(
            &findings(spot_findings(&spots, &map, c["kind"].as_str().unwrap())),
            &c["out"],
            "spots",
        );
    }
    for c in fx["suggest"].as_array().unwrap() {
        let got = suggest_levels(&nums(&c["db"])).map_or(
            Value::Null,
            |(q, k)| json!({ "quietDb": q, "contentDb": k }),
        );
        ok(&got, &c["out"], "suggest");
    }
    for c in fx["labels"].as_array().unwrap() {
        let p = &c["p"];
        let progress = (!p.is_null()).then(|| {
            (
                p["done"].as_i64().unwrap(),
                p["total"].as_i64().unwrap(),
                p["found"].as_i64(),
            )
        });
        assert_eq!(scan_label(progress), c["out"].as_str().unwrap(), "{c}");
    }
}
