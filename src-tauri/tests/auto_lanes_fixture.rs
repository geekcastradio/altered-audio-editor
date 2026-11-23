// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the automation lane helpers in Rust against
//! `shared-fixtures/auto-lanes.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::auto_lanes::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/auto-lanes.json");
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

fn at(v: &Value, k: &str) -> usize {
    v[k].as_u64().unwrap() as usize
}

#[test]
fn choices_and_per_target() {
    let fx = fixture();
    let catalog = fx["catalog"].as_array().unwrap();
    let owners = fx["owners"].as_array().unwrap();
    let targets = fx["targets"].as_array().unwrap();
    for c in fx["choices"].as_array().unwrap() {
        let got = Value::Array(auto_choices(
            &owners[at(c, "oi")],
            catalog,
            c["in3d"].as_bool().unwrap(),
        ));
        ok(
            &got,
            &c["out"],
            &format!("choices {} {}", c["oi"], c["in3d"]),
        );
    }
    for c in fx["per"].as_array().unwrap() {
        let (o, t) = (&owners[at(c, "oi")], &targets[at(c, "ti")]);
        let what = format!("per {} {}", c["oi"], c["ti"]);
        assert_eq!(
            auto_name(o, catalog, t),
            c["name"].as_str().unwrap(),
            "{what}"
        );
        assert_eq!(
            is_orphan(o, catalog, t),
            c["orphan"].as_bool().unwrap(),
            "{what}"
        );
        let r = auto_range(o, catalog, t);
        ok(
            &json!({ "top": r.top, "bottom": r.bottom, "step": r.step, "log": r.log, "unit": r.unit }),
            &c["range"],
            &what,
        );
        ok(&json!(auto_rest(o, catalog, t)), &c["rest"], &what);
    }
    for (t, want) in targets.iter().zip(fx["keys"].as_array().unwrap()) {
        assert_eq!(target_key(t), want.as_str().unwrap());
    }
    for c in fx["next"].as_array().unwrap() {
        let got = next_target(&owners[at(c, "oi")], catalog, c["in3d"].as_bool().unwrap())
            .unwrap_or(Value::Null);
        ok(&got, &c["out"], &format!("next {c}"));
    }
}

#[test]
fn lanes_and_modes() {
    let fx = fixture();
    let owners = fx["owners"].as_array().unwrap();
    let targets = fx["targets"].as_array().unwrap();
    for (o, want) in owners.iter().zip(fx["shown"].as_array().unwrap()) {
        ok(&Value::Array(shown_lanes(o)), want, "shown");
    }
    for c in fx["drop"].as_array().unwrap() {
        ok(
            &Value::Array(drop_lane(&owners[at(c, "oi")], c["id"].as_str().unwrap())),
            &c["out"],
            "drop",
        );
    }
    for (o, want) in owners.iter().zip(fx["mode"].as_array().unwrap()) {
        assert_eq!(auto_mode(o), want.as_str().unwrap());
    }
    for c in fx["modeChange"].as_array().unwrap() {
        let (lanes, record) = mode_change(&owners[at(c, "oi")], c["mode"].as_str().unwrap());
        ok(
            &json!({ "envelopes": lanes, "record": record }),
            &c["out"],
            &format!("modeChange {c}"),
        );
    }
    for c in fx["setMode"].as_array().unwrap() {
        ok(
            &Value::Array(set_mode(&owners[at(c, "oi")], c["mode"].as_str().unwrap())),
            &c["out"],
            "setMode",
        );
    }
    let mut n = 0;
    for (i, t) in targets.iter().take(3).enumerate() {
        n += 1;
        ok(
            &new_lane(t, &format!("id-{n}")),
            &fx["newLane"][i],
            "newLane",
        );
    }
    let lanes = fx["lanes"].as_array().unwrap();
    for c in fx["put"].as_array().unwrap() {
        let mut gen = || {
            n += 1;
            format!("id-{n}")
        };
        let got = Value::Array(put_lane(
            &owners[at(c, "oi")],
            &lanes[at(c, "li")],
            &mut gen,
        ));
        ok(&got, &c["out"], &format!("put {} {}", c["oi"], c["li"]));
    }
    assert_eq!(n, fx["idsUsed"].as_u64().unwrap());
}
