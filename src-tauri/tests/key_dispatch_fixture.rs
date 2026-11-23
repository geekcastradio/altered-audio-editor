// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks what a key press does, in Rust, against `shared-fixtures/key-dispatch.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::chord::KeyEvent;
use altered_audio_editor_lib::key_dispatch::*;
use serde_json::Value;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/key-dispatch.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn decisions() {
    let fx = fixture();
    let map = fx["map"].as_object().unwrap();
    let lookup = |c: &str| map.get(c).and_then(|v| v.as_str()).map(str::to_string);
    let cases = fx["cases"].as_array().unwrap();
    assert!(cases.len() > 500);
    for c in cases {
        let e = &c["ev"];
        let ev = KeyEvent {
            key: e["key"].as_str().unwrap().to_string(),
            ctrl: e["ctrlKey"].as_bool().unwrap(),
            alt: e["altKey"].as_bool().unwrap(),
            shift: e["shiftKey"].as_bool().unwrap(),
            meta: e["metaKey"].as_bool().unwrap(),
            code: e["code"].as_str().map(str::to_string),
        };
        let ctx = Context {
            in_text_field: c["inTextField"].as_bool().unwrap(),
            modal_open: c["modalOpen"].as_bool().unwrap(),
            release: c["release"].as_bool().unwrap(),
        };
        let got = decide(
            &ev,
            c["repeat"].as_bool().unwrap(),
            c["dp"].as_bool().unwrap(),
            ctx,
            &lookup,
        );
        let want = Decision {
            action: c["out"]["action"].as_str().map(str::to_string),
            prevent: c["out"]["prevent"].as_bool().unwrap(),
        };
        assert_eq!(got, want, "case {c}");
    }
}

#[test]
fn arrow_owners() {
    let fx = fixture();
    for c in fx["owns"].as_array().unwrap() {
        let got = owns_arrow_keys(c["tag"].as_str(), c["type"].as_str(), c["role"].as_str());
        assert_eq!(got, c["out"].as_bool().unwrap(), "case {c}");
    }
}
