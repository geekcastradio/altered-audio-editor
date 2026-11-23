// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the shortcut import in Rust against `shared-fixtures/shortcut-import.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::shortcut_import::*;
use serde_json::Value;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/shortcut-import.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn bindings(v: &Value) -> Bindings {
    v.as_object()
        .unwrap()
        .iter()
        .map(|(k, list)| {
            (
                k.clone(),
                list.as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c.as_str().unwrap().to_string())
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn reads_exports() {
    let fx = fixture();
    let cases = fx["parse"].as_array().unwrap();
    assert!(cases.len() > 150);
    let mut errors = 0;
    for c in cases {
        let xml = c["xml"].as_str().unwrap();
        let out = &c["out"];
        match parse_export(xml) {
            Err(_) => {
                errors += 1;
                assert_eq!(out["error"], Value::Bool(true), "{xml}");
            }
            Ok(r) => {
                assert!(out.get("error").is_none(), "{xml}");
                assert_eq!(r.bindings, bindings(&out["bindings"]), "{xml}");
                let unmapped: Vec<String> = out["unmapped"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|s| s.as_str().unwrap().to_string())
                    .collect();
                assert_eq!(r.unmapped, unmapped, "{xml}");
                assert_eq!(
                    r.unreadable as u64,
                    out["unreadable"].as_u64().unwrap(),
                    "{xml}"
                );
            }
        }
    }
    assert!(errors >= 3);
}

#[test]
fn applies_imports() {
    let fx = fixture();
    let defaults = bindings(&fx["defaults"]);
    let cases = fx["apply"].as_array().unwrap();
    assert!(cases.len() > 100);
    let mut taken = 0;
    for c in cases {
        let got = apply_import(
            &bindings(&c["imported"]),
            &bindings(&c["current"]),
            &defaults,
        );
        assert_eq!(got.overrides, bindings(&c["out"]["overrides"]), "{c}");
        let mut want: Vec<(String, String)> = c["out"]["takenFrom"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| {
                (
                    t["action"].as_str().unwrap().to_string(),
                    t["chord"].as_str().unwrap().to_string(),
                )
            })
            .collect();
        want.sort();
        let mut have = got.taken_from.clone();
        have.sort();
        assert_eq!(have, want, "{c}");
        taken += want.len();
    }
    assert!(taken > 50);
}
