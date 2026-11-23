// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the effect envelope rules in Rust against `shared-fixtures/envelope-fx.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::envelope::EnvKind;
use altered_audio_editor_lib::envelope_fx::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/envelope-fx.json");
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

fn lane(v: &Value) -> Option<FxLane> {
    v.as_object().map(|_| FxLane {
        slot: v["slot"].as_str().unwrap().to_string(),
        param: v["param"].as_str().unwrap().to_string(),
    })
}

fn lane_json(l: &Option<FxLane>) -> Value {
    match l {
        Some(l) => json!({ "slot": l.slot, "param": l.param }),
        None => Value::Null,
    }
}

fn catalog(fx: &Value) -> Catalog {
    Catalog::new(fx["catalog"].as_array().unwrap())
}

#[test]
fn lanes_and_ranges() {
    let fx = fixture();
    let cat = catalog(&fx);
    let cases = fx["cases"].as_array().unwrap();
    assert!(cases.len() > 200);
    for c in cases {
        let kind = EnvKind::parse(c["kind"].as_str().unwrap()).unwrap();
        let clip = if c["clip"].is_null() {
            None
        } else {
            Some(&c["clip"])
        };
        let l = lane(&c["lane"]);
        let r = range_for(kind, &cat, clip, l.as_ref());
        let want = &c["range"];
        assert!(
            same(
                &json!({ "top": r.top, "bottom": r.bottom, "step": r.step }),
                want
            ),
            "range {c}"
        );
        let rest = rest_value(kind, &cat, clip, l.as_ref());
        assert!(same(&json!(rest), &c["rest"]), "rest {c}");
        assert_eq!(
            env_name(kind, &cat, clip, l.as_ref()),
            c["name"],
            "name {c}"
        );
        if let Some(clip) = clip {
            let pts = points_of(clip, kind, l.as_ref());
            assert!(same(&Value::Array(pts), &c["points"]), "points {c}");
            let change = env_change(
                clip,
                kind,
                &[json!({"position":1,"value":0.5,"curve":"linear"})],
                l.as_ref(),
            );
            assert!(
                same(&change.unwrap_or(Value::Null), &c["change"]),
                "change {c}"
            );
        }
    }
}

#[test]
fn choices_and_picked_lanes() {
    let fx = fixture();
    let cat = catalog(&fx);
    for c in fx["choices"].as_array().unwrap() {
        let got: Vec<Value> = fx_choices(&cat, &c["clip"])
            .into_iter()
            .map(|(l, label)| json!({ "lane": { "slot": l.slot, "param": l.param }, "label": label }))
            .collect();
        assert!(same(&Value::Array(got), &c["out"]), "choices {c}");
    }
    for c in fx["laneOf"].as_array().unwrap() {
        let picked = lane(&c["picked"]);
        let got = fx_lane_of(&cat, &c["clip"], picked.as_ref());
        assert!(same(&lane_json(&got), &c["out"]), "laneOf {c}");
    }
    for c in fx["fxp"].as_array().unwrap() {
        let l = lane(&c["lane"]);
        let got = match fx_param(&cat, &c["clip"], l.as_ref()) {
            Some(p) => {
                json!({ "effect": p.effect, "param": p.param, "min": p.min, "max": p.max, "step": p.step, "value": p.value })
            }
            None => Value::Null,
        };
        assert!(same(&got, &c["out"]), "fxParam {c}");
    }
}
