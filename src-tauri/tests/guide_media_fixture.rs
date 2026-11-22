// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust User Guide reader and session document rules against
//! `shared-fixtures/guide-media.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::guide::*;
use altered_audio_editor_lib::session_rules::*;
use serde_json::{json, Value};
use std::collections::HashSet;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/guide-media.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn pairs(v: &Value) -> Vec<(String, String)> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p[0].as_str().unwrap().to_string(),
                p[1].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn guide_reading() {
    let fx = fixture();
    for c in fx["parsed"].as_array().unwrap() {
        let got = serde_json::to_value(parse_guide(c["md"].as_str().unwrap())).unwrap();
        assert_eq!(
            got,
            c["out"],
            "parse of {:?}",
            &c["md"].as_str().unwrap()[..c["md"].as_str().unwrap().len().min(60)]
        );
    }
    for c in fx["spanCases"].as_array().unwrap() {
        assert_eq!(
            serde_json::to_value(spans(c["l"].as_str().unwrap())).unwrap(),
            c["out"],
            "spans {}",
            c["l"]
        );
    }
    let real = &fx["parsed"].as_array().unwrap().last().unwrap()["md"];
    let sections = parse_guide(real.as_str().unwrap());
    for c in fx["searches"].as_array().unwrap() {
        let got: Vec<&str> = search_guide(&sections, c["q"].as_str().unwrap())
            .iter()
            .map(|s| s.id.as_str())
            .collect();
        let want: Vec<&str> = c["out"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        assert_eq!(got, want, "search {}", c["q"]);
    }
}

#[test]
fn media_copy_rules() {
    let fx = fixture();
    let paths = [
        "C:\\Audio\\a.wav",
        "c:/audio/A.wav",
        "C:\\Session\\x\\b.wav",
        "C:\\Session\\x\\b.wav",
        "/home/u/lib/c.wav",
        "/home/u/lib2/d.wav",
        "D:\\Other\\e.wav",
        "C:\\Session\\xx\\f.wav",
        "/home/u/Lib/g.wav",
    ]
    .map(String::from);
    for c in fx["copies"].as_array().unwrap() {
        let f = c["f"].as_str().unwrap();
        assert_eq!(folder_of(f), c["home"].as_str().unwrap(), "folder {f}");
        let got = to_copy(&paths, f, &strings(&c["leave"]), c["ic"].as_bool().unwrap());
        assert_eq!(got, strings(&c["out"]), "copy {c}");
    }
    for c in fx["inside"].as_array().unwrap() {
        let d = strings(&c["d"]);
        let p = c["p"].as_str().unwrap();
        assert_eq!(
            vec![inside_any(p, &d, true), inside_any(p, &d, false)],
            c["ic"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| b.as_bool().unwrap())
                .collect::<Vec<_>>(),
            "inside {c}"
        );
    }
    for c in fx["repoints"].as_array().unwrap() {
        let media = c["media"].as_array().unwrap();
        let pool: Vec<(String, String)> = c["pool"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                (
                    p["id"].as_str().unwrap().to_string(),
                    p["path"].as_str().unwrap().to_string(),
                )
            })
            .collect();
        let (ops, files) = repoint_plan(
            media,
            &pool,
            &pairs(&c["moved"]),
            c["ic"].as_bool().unwrap(),
        );
        let want_files: Vec<(String, String)> = c["out"]["pool"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                (
                    p["id"].as_str().unwrap().to_string(),
                    p["path"].as_str().unwrap().to_string(),
                )
            })
            .collect();
        assert_eq!(Value::Array(ops), c["out"]["ops"], "repoint ops {c}");
        assert_eq!(files, want_files, "repoint files");
    }
}

#[test]
fn session_properties_and_lists() {
    let fx = fixture();
    for c in fx["props"].as_array().unwrap() {
        let got = properties_op(&c["doc"], &c["next"]).unwrap_or(Value::Null);
        assert_eq!(got, c["out"], "props {c}");
    }
    for c in fx["current"].as_array().unwrap() {
        assert_eq!(current_properties(&c["doc"]), c["out"]);
    }
    for c in fx["bleeds"].as_array().unwrap() {
        let known: HashSet<String> = strings(&c["known"]).into_iter().collect();
        let files: Vec<(Value, Value)> = c["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| (f["clip"].clone(), f["media"].clone()))
            .collect();
        assert_eq!(
            Value::Array(bleed_replace_ops("t1", &files, &known)),
            c["out"],
            "bleed {c}"
        );
    }
    let consts = &fx["consts"];
    assert_eq!(json!(SAMPLE_RATES), consts["rates"]);
    assert_eq!(
        Value::Array(
            BIT_DEPTHS
                .iter()
                .map(|(v, l)| json!({ "value": v, "label": l }))
                .collect()
        ),
        consts["depths"]
    );
    assert_eq!(
        Value::Array(
            PAN_LAWS
                .iter()
                .map(|(v, l, h)| json!({ "value": v, "label": l, "hint": h }))
                .collect()
        ),
        consts["pans"]
    );
    assert_eq!(
        Value::Array(
            LOUDNESS_CHOICES
                .iter()
                .map(|(i, l)| json!({ "id": i, "label": l }))
                .collect()
        ),
        consts["loud"]
    );
    let list = fx["profiles"]["list"].as_array().unwrap();
    assert_eq!(list.len(), HOST_PROFILES.len());
    for (p, w) in HOST_PROFILES.iter().zip(list) {
        assert_eq!(p.id, w["id"].as_str().unwrap());
        assert_eq!(p.name, w["name"].as_str().unwrap());
        assert_eq!(p.note, w["note"].as_str().unwrap());
        assert_eq!(p.gate_db, w["gateDb"].as_f64().unwrap());
        assert_eq!(
            if p.guest { "guest" } else { "standard" },
            w["role"].as_str().unwrap()
        );
    }
    for c in fx["profiles"]["ids"].as_array().unwrap() {
        assert_eq!(
            profile_by_id(c["id"].as_str()).map(|p| p.name),
            c["out"].as_str(),
            "profile {c}"
        );
    }
}
