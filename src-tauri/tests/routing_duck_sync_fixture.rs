// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust mix routing, Duck Music Under Voice and Sync Tracks rules
//! against `shared-fixtures/routing-duck-sync.json`, which the TypeScript test
//! reads too.

use altered_audio_editor_lib::mix_routing::*;
use serde_json::{json, Map, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/routing-duck-sync.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

/// Equal, with numbers compared as numbers (1 and 1.0 are the same).
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() < 1e-9
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

fn ids(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn track<'a>(doc: &'a Value, id: &str) -> &'a Value {
    doc["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["id"] == id)
        .unwrap()
}

fn id_list(v: &[Value]) -> Vec<String> {
    v.iter()
        .map(|t| t["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn routing() {
    let fx = fixture();
    let docs = fx["docs"].as_array().unwrap();
    for c in fx["routing"].as_array().unwrap() {
        let doc = &docs[c["di"].as_u64().unwrap() as usize];
        for p in c["pairs"].as_array().unwrap() {
            assert_eq!(
                reaches(doc, p["a"].as_str().unwrap(), p["b"].as_str().unwrap()),
                p["out"].as_bool().unwrap(),
                "reaches {p}"
            );
        }
        for b in c["bus"].as_array().unwrap() {
            assert_eq!(
                id_list(&bus_choices(doc, b["o"].as_str().unwrap())),
                ids(&b["out"]),
                "bus choices {b}"
            );
        }
        for k in c["keys"].as_array().unwrap() {
            ok(
                &Value::Array(key_choices(doc, k["o"].as_str().unwrap())),
                &k["out"],
                &format!("key choices {k}"),
            );
        }
        for o in c["outs"].as_array().unwrap() {
            let out = (!o["o"].is_null()).then_some(&o["o"]);
            assert_eq!(
                output_name(doc, out),
                o["name"].as_str().unwrap(),
                "name {o}"
            );
            assert_eq!(output_value(out), o["value"].as_str().unwrap(), "value {o}");
        }
        for o in c["ops"].as_array().unwrap() {
            let got = bus_op(
                Some(doc),
                o["id"].as_str().unwrap(),
                &o["changes"],
                o["ex"].as_bool().unwrap(),
            );
            ok(&got, &o["out"], &format!("bus op {o}"));
        }
        assert_eq!(is_routed(Some(doc)), c["routed"].as_bool().unwrap());
        for (t, want) in doc["tracks"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c["sources"].as_array().unwrap())
        {
            assert_eq!(key_sources(t.get("effects")), ids(want));
        }
    }
    assert_eq!(is_routed(None), fx["noDoc"]["routed"].as_bool().unwrap());
    ok(
        &bus_op(None, "b", &json!({ "solo": true }), true),
        &fx["noDoc"]["op"],
        "no doc op",
    );
    for c in fx["picks"].as_array().unwrap() {
        ok(&output_of(c["v"].as_str().unwrap()), &c["out"], "output_of");
    }
    for c in fx["patches"].as_array().unwrap() {
        let sends = vec![new_send("b1", "s1"), new_send("b2", "s2")];
        let patch: Map<String, Value> = c["p"].as_object().unwrap().clone();
        ok(
            &Value::Array(set_send(&sends, "s2", &patch)),
            &c["out"],
            "set_send",
        );
    }
    for c in fx["removes"].as_array().unwrap() {
        let sends = vec![new_send("b1", "s1"), new_send("b2", "s2")];
        ok(
            &Value::Array(remove_send(&sends, c["id"].as_str().unwrap())),
            &c["out"],
            "remove_send",
        );
    }
}

#[test]
fn duck() {
    let fx = fixture();
    let docs = fx["docs"].as_array().unwrap();
    let k = &fx["consts"];
    assert_eq!(DUCK_PRESET, k["preset"].as_str().unwrap());
    assert_eq!(DUCK_THRESHOLD_DB, k["thr"].as_f64().unwrap());
    for (d, w) in DUCK_DEPTHS.iter().zip(k["depths"].as_array().unwrap()) {
        assert_eq!(d.0, w["id"].as_str().unwrap());
        assert_eq!(d.1, w["label"].as_str().unwrap());
        assert_eq!(d.2, w["db"].as_f64().unwrap());
    }
    for c in fx["params"].as_array().unwrap() {
        let db = c["db"].as_f64().unwrap();
        ok(&duck_params(db), &c["out"], "duck params");
        ok(&duck_slot("id", "v1", db), &c["slot"], "duck slot");
    }
    for c in fx["slotTests"].as_array().unwrap() {
        assert_eq!(is_duck_slot(&c["sl"]), c["out"].as_bool().unwrap(), "{c}");
    }
    for c in fx["duck"].as_array().unwrap() {
        let doc = &docs[c["di"].as_u64().unwrap() as usize];
        let tracks = doc["tracks"].as_array().unwrap();
        for (t, w) in tracks.iter().zip(c["isVoice"].as_array().unwrap()) {
            assert_eq!(is_voice_track(t), w.as_bool().unwrap(), "voice {t}");
        }
        for p in c["plans"].as_array().unwrap() {
            let sel = p["sel"].as_str().map(|id| track(doc, id));
            let got = duck_plan(doc, sel);
            match (&got, &p["out"]) {
                (None, Value::Null) => {}
                (Some(g), w) => {
                    assert_eq!(id_list(&g.targets), ids(&w["targets"]), "targets {p}");
                    assert_eq!(id_list(&g.keys), ids(&w["keys"]), "keys {p}");
                    assert_eq!(g.by_role, w["byRole"].as_bool().unwrap());
                }
                _ => panic!("plan mismatch {p}"),
            }
        }
        for kc in c["keys"].as_array().unwrap() {
            assert_eq!(
                id_list(&duck_keys(doc, track(doc, kc["id"].as_str().unwrap()))),
                ids(&kc["out"])
            );
        }
        for (t, w) in tracks.iter().zip(c["has"].as_array().unwrap()) {
            assert_eq!(has_duck(t), w.as_bool().unwrap());
        }
        for w in c["withs"].as_array().unwrap() {
            let t = track(doc, w["t"].as_str().unwrap());
            let chain = t.get("effects").cloned().unwrap_or(
                json!({ "enabled": true, "input_gain_db": 0, "output_gain_db": 0, "slots": [] }),
            );
            let mut n = 0;
            let mut new_id = || {
                n += 1;
                format!("n{n}")
            };
            let got = with_duck(
                &chain,
                &ids(&w["v"]),
                10.0,
                &mut new_id,
                w["max"].as_u64().unwrap() as usize,
            )
            .unwrap_or(Value::Null);
            ok(&got, &w["out"], "with_duck");
            ok(&without_duck(&chain), &w["without"], "without_duck");
        }
    }
}

#[test]
fn sync() {
    let fx = fixture();
    let docs = fx["docs"].as_array().unwrap();
    assert_eq!(DRIFT_QUIET_PPM, fx["consts"]["quiet"].as_f64().unwrap());
    for c in fx["moves"].as_array().unwrap() {
        let doc = &docs[c["di"].as_u64().unwrap() as usize];
        let got = match move_track(doc, c["id"].as_str().unwrap(), c["off"].as_f64().unwrap()) {
            Ok(Move::Op(v)) => v,
            Ok(Move::BeforeStart) => json!("before-start"),
            Ok(Move::Blocked) => json!("blocked"),
            Ok(Move::Nothing) => Value::Null,
            Err(Locked) => json!({ "throws": "locked" }),
        };
        ok(&got, &c["out"], &format!("move {} {}", c["id"], c["off"]));
    }
    for c in fx["describes"].as_array().unwrap() {
        let r = &c["r"];
        let sync = TrackSync {
            offset: r["offset"].as_f64().unwrap(),
            drift_ppm: r["driftPpm"].as_f64().unwrap(),
            confidence: r["confidence"].as_f64().unwrap(),
            inverted: r["inverted"].as_bool().unwrap(),
        };
        assert_eq!(
            describe_sync(&sync, c["rate"].as_f64().unwrap()),
            ids(&c["out"]),
            "describe {c}"
        );
    }
    for c in fx["trusts"].as_array().unwrap() {
        let sync = TrackSync {
            offset: 0.0,
            drift_ppm: 0.0,
            confidence: c["c"].as_f64().unwrap(),
            inverted: false,
        };
        assert_eq!(trust(&sync), c["out"].as_str().unwrap());
    }
}
