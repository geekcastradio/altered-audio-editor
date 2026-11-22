// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust Source Channels and channel switch rules against
//! `shared-fixtures/channel-rules.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::channel_rules::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/channel-rules.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn u(v: &Value) -> usize {
    v.as_u64().unwrap() as usize
}

fn sends(v: &Value) -> Vec<Send> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| Send {
            left: s["left"].as_f64().unwrap(),
            right: s["right"].as_f64().unwrap(),
        })
        .collect()
}

fn same(a: &[Send], b: &[Send]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| (x.left - y.left).abs() < 1e-9 && (x.right - y.right).abs() < 1e-9)
}

fn flags(v: &Value) -> Vec<bool> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|b| b.as_bool().unwrap())
        .collect()
}

fn takes(t: Option<Vec<Source>>) -> Value {
    match t {
        None => Value::Null,
        Some(list) => Value::Array(
            list.iter()
                .map(|s| json!({ "after": s.after, "channel": s.channel }))
                .collect(),
        ),
    }
}

#[test]
fn labels_and_default_mixes() {
    let fx = fixture();
    for c in fx["labels"].as_array().unwrap() {
        let want: Vec<String> = c["out"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().to_string())
            .collect();
        assert_eq!(channel_labels(u(&c["n"])), want, "labels {}", c["n"]);
    }
    for c in fx["defaults"].as_array().unwrap() {
        assert!(
            same(&default_mix(u(&c["n"])), &sends(&c["out"])),
            "default {}",
            c["n"]
        );
    }
    for c in fx["owns"].as_array().unwrap() {
        let own = (!c["own"].is_null()).then(|| sends(&c["own"]["sends"]));
        let got = mix_of(own.as_deref(), u(&c["n"]));
        assert!(same(&got, &sends(&c["out"])), "mix_of {c}");
    }
}

#[test]
fn presets_and_matching() {
    let fx = fixture();
    for c in fx["pres"].as_array().unwrap() {
        let n = u(&c["n"]);
        let want = c["out"].as_array().unwrap();
        let got = presets(n);
        assert_eq!(got.len(), want.len(), "preset count {n}");
        for (g, w) in got.iter().zip(want) {
            assert_eq!(g.id, w["id"].as_str().unwrap());
            assert_eq!(g.label, w["label"].as_str().unwrap());
            assert!(same(&g.mix, &sends(&w["sends"])), "{n} {}", g.id);
        }
    }
    for c in fx["matches"].as_array().unwrap() {
        let got = preset_for(&sends(&c["sends"]), u(&c["n"]));
        assert_eq!(got.as_deref(), c["out"].as_str(), "match {}", c["n"]);
    }
}

#[test]
fn channel_switches() {
    let fx = fixture();
    for c in fx["allOn"].as_array().unwrap() {
        assert_eq!(all_on(u(&c["n"])), flags(&c["out"]));
    }
    for c in fx["channels"].as_array().unwrap() {
        let f = flags(&c["c"]);
        for (k, want) in c["toggles"].as_array().unwrap().iter().enumerate() {
            assert_eq!(toggle_channel(&f, k), flags(want), "toggle {f:?} {k}");
        }
        for (k, want) in c["solos"].as_array().unwrap().iter().enumerate() {
            assert_eq!(solo_channel(&f, k), flags(want), "solo {f:?} {k}");
        }
        assert_eq!(one_channel(&f).map(|k| k as u64), c["one"].as_u64());
        assert_eq!(some_off(&f), c["off"].as_bool().unwrap());
        assert_eq!(takes(edit_take(&f)), c["edit"], "edit {f:?}");
        let last = f.len() - 1;
        let want = c["paste"].as_array().unwrap();
        assert_eq!(takes(paste_take(&f, None)), want[0], "paste none {f:?}");
        assert_eq!(takes(paste_take(&f, Some(0))), want[1], "paste 0 {f:?}");
        assert_eq!(
            takes(paste_take(&f, Some(last))),
            want[2],
            "paste last {f:?}"
        );
        assert_eq!(channel_label(&f), c["label"].as_str().unwrap());
    }
}
