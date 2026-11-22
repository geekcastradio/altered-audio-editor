// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust Voice Chain rules against `shared-fixtures/voice-chain.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::voice_chain::*;
use serde_json::{json, Map, Value};
use std::collections::HashSet;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/voice-chain.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

/// Equal, with numbers compared as numbers (1 and 1.0 are the same).
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

fn opt(v: Option<Value>) -> Value {
    v.unwrap_or(Value::Null)
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn counter() -> impl FnMut() -> String {
    let mut n = 0;
    move || {
        n += 1;
        format!("n{n}")
    }
}

fn slot() -> Value {
    json!({ "id": "new", "effect": "altaudio.gain", "enabled": true, "params": {}, "preset": null })
}

#[test]
fn chain_and_slot_edits() {
    let fx = fixture();
    for c in fx["chainOf"].as_array().unwrap() {
        let input = (!c["c"].is_null()).then_some(&c["c"]);
        ok(&chain_of(input), &c["out"], &format!("chain_of {c}"));
    }
    for c in fx["newSlots"].as_array().unwrap() {
        ok(&new_slot(&c["info"], "id1"), &c["out"], "new_slot");
    }
    for o in fx["ops"].as_array().unwrap() {
        let chain = &o["c"];
        let layout: Vec<Value> = layout_slots(chain)
            .into_iter()
            .map(|s| s.unwrap_or(Value::Null))
            .collect();
        ok(&Value::Array(layout), &o["layout"], "layout");
        for a in o["add"].as_array().unwrap() {
            let idx = (!a["index"].is_null()).then(|| f(&a["index"]));
            ok(
                &opt(add_slot(chain, &slot(), idx)),
                &a["out"],
                &format!("add {a}"),
            );
        }
        for a in o["addAt"].as_array().unwrap() {
            ok(
                &opt(add_slot_at(chain, &slot(), f(&a["at"]))),
                &a["out"],
                &format!("add_at {}", a["at"]),
            );
        }
        for a in o["toIdx"].as_array().unwrap() {
            ok(
                &move_slot_to_index(chain, a["id"].as_str().unwrap(), f(&a["at"])),
                &a["out"],
                &format!("to_idx {} {}", a["id"], a["at"]),
            );
        }
        for a in o["remove"].as_array().unwrap() {
            ok(
                &remove_slot(chain, a["id"].as_str().unwrap()),
                &a["out"],
                "remove",
            );
        }
        for a in o["move"].as_array().unwrap() {
            ok(
                &move_slot(chain, a["id"].as_str().unwrap(), a["by"].as_i64().unwrap()),
                &a["out"],
                &format!("move {a}"),
            );
        }
        for a in o["moveTo"].as_array().unwrap() {
            ok(
                &move_slot_to(chain, a["id"].as_str().unwrap(), a["to"].as_i64().unwrap()),
                &a["out"],
                &format!("move_to {a}"),
            );
        }
        for a in o["replace"].as_array().unwrap() {
            ok(
                &replace_slot(chain, a["id"].as_str().unwrap(), &slot()),
                &a["out"],
                "replace",
            );
        }
        for a in o["set"].as_array().unwrap() {
            let change: Map<String, Value> = a["change"].as_object().unwrap().clone();
            ok(
                &set_slot(chain, a["id"].as_str().unwrap(), &change),
                &a["out"],
                "set",
            );
        }
    }
}

#[test]
fn params_and_sliders() {
    let fx = fixture();
    let chain = &fx["ops"][3]["c"];
    for c in fx["params"].as_array().unwrap() {
        let p = ParamInfo::from_json(&c["info"]);
        for a in c["clamp"].as_array().unwrap() {
            assert_eq!(
                clamp_param(&p, f(&a["v"])),
                f(&a["out"]),
                "clamp {a} {}",
                c["info"]
            );
        }
        for a in c["slider"].as_array().unwrap() {
            let got = to_slider(&p, f(&a["v"]));
            // A log slider of a value at or below zero is not a number; JSON keeps that as null.
            if a["out"].is_null() {
                assert!(got.is_nan() || got.is_infinite(), "slider {a}: got {got}");
            } else {
                assert!(
                    (got - f(&a["out"])).abs() < 1e-9,
                    "slider {a} {}: got {got}",
                    c["info"]
                );
            }
        }
        for a in c["from"].as_array().unwrap() {
            let got = from_slider(&p, f(&a["t"]));
            assert!(
                (got - f(&a["out"])).abs() < 1e-9,
                "from {a} {}: got {got}",
                c["info"]
            );
        }
        for a in c["fmt"].as_array().unwrap() {
            assert_eq!(
                format_param(&p, f(&a["v"])),
                a["out"].as_str().unwrap(),
                "fmt {a} {}",
                c["info"]
            );
        }
        for a in c["setp"].as_array().unwrap() {
            ok(
                &set_param(chain, "s0", &p, f(&a["v"])),
                &a["out"],
                "set_param",
            );
        }
        for a in c["pv"].as_array().unwrap() {
            assert_eq!(param_value(&a["slot"], &p), f(&a["out"]), "param_value {a}");
        }
    }
}

#[test]
fn presets() {
    let fx = fixture();
    let all = builtin_presets();
    let names: Vec<&str> = all.iter().map(|p| p["name"].as_str().unwrap()).collect();
    let want_names: Vec<&str> = fx["flags"]["names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap())
        .collect();
    assert_eq!(names, want_names);
    for (p, w) in all.iter().zip(fx["flags"]["addsOne"].as_array().unwrap()) {
        assert_eq!(adds_one(p), w.as_bool().unwrap(), "adds_one {}", p["name"]);
    }
    let batch: Vec<String> = batch_presets(&all)
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect();
    let want_batch: Vec<String> = fx["flags"]["batch"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().to_string())
        .collect();
    assert_eq!(batch, want_batch);

    let known_all: HashSet<String> = all
        .iter()
        .flat_map(|p| {
            p["chain"]["slots"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s["effect"].as_str().unwrap().to_string())
        })
        .chain(
            [
                "altaudio.limiter",
                "altaudio.dynamics",
                "altaudio.parametric_eq",
                "altaudio.noise_reducer",
            ]
            .map(String::from),
        )
        .collect();
    let known_some: HashSet<String> = ["altaudio.limiter", "altaudio.dynamics"]
        .map(String::from)
        .into_iter()
        .collect();
    for c in fx["apply"].as_array().unwrap() {
        let p = all.iter().find(|p| p["name"] == c["name"]).unwrap();
        let known = if c["kn"] == "all" {
            &known_all
        } else {
            &known_some
        };
        let cur = if c["ci"] == 0 {
            &fx["ops"][0]["c"]
        } else {
            &fx["ops"][3]["c"]
        };
        let (mut a, mut b) = (counter(), counter());
        ok(
            &preset_chain(p, known, &mut a),
            &c["chain"],
            &format!("preset_chain {}", c["name"]),
        );
        ok(
            &opt(apply_preset(cur, p, known, &mut b)),
            &c["apply"],
            &format!("apply {}", c["name"]),
        );
    }
    for c in fx["from"].as_array().unwrap() {
        ok(
            &preset_from(c["name"].as_str().unwrap(), &c["chain"]),
            &c["out"],
            "preset_from",
        );
    }
    for c in fx["sanitized"].as_array().unwrap() {
        ok(
            &Value::Array(sanitize_presets(&c["raw"])),
            &c["out"],
            &format!("sanitize {}", c["raw"]),
        );
    }
}

#[test]
fn builtin_data_is_well_formed() {
    let data = preset_data();
    for p in data["builtin"].as_array().unwrap() {
        for s in p["chain"]["slots"].as_array().unwrap() {
            assert!(
                s["effect"].as_str().unwrap().starts_with("altaudio."),
                "{}",
                p["name"]
            );
        }
    }
    assert!(data["colors"].as_object().is_some_and(|c| !c.is_empty()));
}
