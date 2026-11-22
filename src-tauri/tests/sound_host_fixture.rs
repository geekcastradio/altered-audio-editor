// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks host profiles and Sound Types in Rust against
//! `shared-fixtures/sound-host.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::host_profiles::*;
use altered_audio_editor_lib::sound_types::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/sound-host.json");
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

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("n{n}")
    }
}

fn setup_json(s: &Option<Setup>) -> Value {
    s.as_ref().map_or(Value::Null, |s| json!({ "type": s.kind, "sections": s.sections, "preset": s.preset, "amount": s.amount }))
}

fn setup_of(v: &Value) -> Setup {
    Setup {
        kind: v["type"].as_str().unwrap().into(),
        sections: v["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().into())
            .collect(),
        preset: v["preset"].as_str().unwrap().into(),
        amount: v["amount"].as_f64().unwrap(),
    }
}

#[test]
fn host_profiles() {
    let fx = fixture();
    for c in fx["profiles"].as_array().unwrap() {
        let p = profile_by_id(c["p"]["id"].as_str()).unwrap();
        assert_eq!(p.name, c["p"]["name"].as_str().unwrap());
        assert_eq!(p.gate_db, c["p"]["gateDb"].as_f64().unwrap());
        assert_eq!(p.role(), c["p"]["role"].as_str().unwrap());
        assert_eq!(p.note, c["p"]["note"].as_str().unwrap());
        ok(&profile_chain(&p, &mut counter()), &c["chain"], "chain");
        ok(&profile_op("t1", Some(&p), &mut counter()), &c["op"], "op");
    }
    ok(
        &profile_op("t1", None, &mut counter()),
        &fx["clear"],
        "clear",
    );
    for c in fx["byId"].as_array().unwrap() {
        let got = profile_by_id(c["id"].as_str()).map(|p| p.id);
        assert_eq!(got, c["out"]["id"].as_str(), "{c}");
    }
}

#[test]
fn sections_and_setups() {
    let fx = fixture();
    for c in fx["sections"].as_array().unwrap() {
        for o in c["outs"].as_array().unwrap() {
            let got: Vec<Value> = section_effects(
                c["type"].as_str().unwrap(),
                c["sec"].as_str().unwrap(),
                o["a"].as_f64().unwrap(),
            )
            .into_iter()
            .map(|m| json!({ "effect": m.effect, "params": m.params }))
            .collect();
            ok(
                &Value::Array(got),
                &o["out"],
                &format!("{} {} {}", c["type"], c["sec"], o["a"]),
            );
        }
    }
    for c in fx["setups"].as_array().unwrap() {
        let k = c["type"].as_str().unwrap();
        ok(
            &setup_json(&preset_setup(k, c["id"].as_str().unwrap())),
            &c["out"],
            "preset_setup",
        );
        ok(
            &setup_json(&preset_setup(k, "zzz")),
            &c["fallback"],
            "fallback",
        );
    }
    for c in fx["made"].as_array().unwrap() {
        ok(
            &Value::Array(make_slots(&setup_of(&c["setup"]), &mut counter())),
            &c["slots"],
            "make_slots",
        );
    }
}

#[test]
fn chains() {
    let fx = fixture();
    for c in fx["chains"].as_array().unwrap() {
        let chain = &c["chain"];
        let setup = read_setup(Some(chain));
        ok(&setup_json(&setup), &c["setup"], "read_setup");
        assert_eq!(
            is_custom(Some(chain), setup.as_ref()),
            c["custom"].as_bool().unwrap(),
            "custom {c}"
        );
        let made = &fx["made"][c["i"].as_u64().unwrap() as usize]["setup"];
        let m = setup_of(made);
        assert_eq!(
            is_custom(Some(chain), Some(&m)),
            c["customOther"].as_bool().unwrap(),
            "custom other"
        );
        ok(
            &with_setup(chain, &m, &mut counter(), 16).unwrap_or(Value::Null),
            &c["with"],
            "with",
        );
        ok(
            &with_setup(chain, &m, &mut counter(), 3).unwrap_or(Value::Null),
            &c["withMax"],
            "with max",
        );
        ok(&without_setup(chain), &c["without"], "without");
    }
    let n = &fx["nulls"];
    ok(&setup_json(&read_setup(None)), &n["readNull"], "none");
    assert_eq!(is_custom(None, None), n["customNull"].as_bool().unwrap());
    assert_eq!(
        is_custom(Some(&json!({ "slots": [] })), None),
        n["customNoSetup"].as_bool().unwrap()
    );
}
