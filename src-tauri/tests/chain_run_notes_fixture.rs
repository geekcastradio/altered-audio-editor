// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the chain run notices in Rust against `shared-fixtures/chain-run-notes.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::chain_run_notes::*;
use serde_json::Value;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/chain-run-notes.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn preflight_messages() {
    let fx = fixture();
    for c in fx["pre"].as_array().unwrap() {
        let problems: Vec<Problem> = c["problems"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| Problem {
                step: p["step"].as_u64().map(|n| n as usize),
                message: p["message"].as_str().unwrap().to_string(),
            })
            .collect();
        let got = preflight(&problems, &strs(&c["steps"]), &strs(&c["handlers"]));
        assert_eq!(got.as_deref(), c["out"].as_str(), "case {c}");
    }
}

#[test]
fn summaries() {
    let fx = fixture();
    for c in fx["sum"].as_array().unwrap() {
        let reports: Vec<Report> = c["reports"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| Report {
                title: r["title"].as_str().unwrap().to_string(),
                status: match r["status"].as_str().unwrap() {
                    "done" => Status::Done,
                    "nothing" => Status::Nothing,
                    _ => Status::Skipped,
                },
                message: r["message"].as_str().unwrap().to_string(),
            })
            .collect();
        let got = chain_summary(c["name"].as_str().unwrap(), &reports);
        assert_eq!(got, c["out"].as_str().unwrap(), "case {c}");
    }
}
