// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust Match EQ against `shared-fixtures/match-eq.json`, which the
//! TypeScript test reads too.

use altered_audio_editor_lib::match_eq::*;
use serde_json::Value;

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/match-eq.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn near(a: f64, b: f64, what: &str) {
    let tol = 1e-6 * b.abs().max(1.0);
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b}");
}

fn list(v: &Value) -> Vec<f64> {
    v.as_array().unwrap().iter().map(f).collect()
}

fn spectrum(v: &Value) -> Spectrum {
    Spectrum {
        hz: list(&v["hz"]),
        db: list(&v["db"]),
    }
}

fn grid(n: usize, lo: f64, hi: f64) -> Vec<f64> {
    (0..n)
        .map(|i| lo * (hi / lo).powf(i as f64 / (n as f64 - 1.0)))
        .collect()
}

fn same_list(got: &[f64], want: &Value, what: &str) {
    let want = list(want);
    assert_eq!(got.len(), want.len(), "{what}");
    for (i, (g, w)) in got.iter().zip(&want).enumerate() {
        near(*g, *w, &format!("{what}[{i}]"));
    }
}

fn same_fit(got: &Option<MatchFit>, want: &Value, what: &str) {
    match (got, want) {
        (None, Value::Null) => {}
        (Some(g), w) if !w.is_null() => {
            let wp = w["params"].as_object().unwrap();
            assert_eq!(
                g.params.keys().collect::<Vec<_>>(),
                {
                    let mut k: Vec<_> = wp.keys().collect();
                    k.sort();
                    k
                },
                "{what}: params"
            );
            for (k, v) in &g.params {
                near(*v, f(&wp[k]), &format!("{what}: {k}"));
            }
            same_list(&g.target, &w["target"], &format!("{what}: target"));
            same_list(&g.fitted, &w["fitted"], &format!("{what}: fitted"));
            same_list(&g.hz, &w["hz"], &format!("{what}: hz"));
            near(g.error_before, f(&w["errorBefore"]), what);
            near(g.error_after, f(&w["errorAfter"]), what);
        }
        other => panic!("{what}: {other:?}"),
    }
}

#[test]
fn filter_responses() {
    for c in fixture()["response"].as_array().unwrap() {
        let shape = match c["shape"].as_str().unwrap() {
            "peak" => Shape::Peak,
            "lowShelf" => Shape::LowShelf,
            _ => Shape::HighShelf,
        };
        let got = response_db(
            shape,
            f(&c["rate"]),
            f(&c["f0"]),
            f(&c["q"]),
            f(&c["g"]),
            f(&c["f"]),
        );
        near(got, f(&c["out"]), &c.to_string());
    }
}

#[test]
fn matches() {
    for c in fixture()["fits"].as_array().unwrap() {
        let opts = MatchOptions {
            cap_db: c["opts"]["capDb"].as_f64(),
            strength: c["opts"]["strength"].as_f64(),
        };
        let got = match_eq(&spectrum(&c["src"]), &spectrum(&c["ref"]), opts);
        same_fit(&got, &c["fit"], &format!("{} {}", c["name"], c["oname"]));
    }
}

#[test]
fn voice_target_and_small_cases() {
    for c in fixture()["target"].as_array().unwrap() {
        if c.get("tgt").is_some() {
            let t = voice_target(c["points"].as_u64().unwrap() as usize);
            same_list(&t.hz, &c["tgt"]["hz"], "target hz");
            same_list(&t.db, &c["tgt"]["db"], "target db");
        } else {
            same_fit(
                &voice_target_eq(&spectrum(&c["src"])),
                &c["fit"],
                &c["name"].to_string(),
            );
        }
    }
    for c in fixture()["small"].as_array().unwrap() {
        if c.get("mismatch").is_some() {
            let s = Spectrum {
                hz: grid(10, 30.0, 20000.0),
                db: vec![1.0, 2.0, 3.0],
            };
            let r = Spectrum {
                hz: grid(10, 30.0, 20000.0),
                db: vec![0.0; 10],
            };
            assert!(match_eq(&s, &r, MatchOptions::default()).is_none());
            continue;
        }
        let n = c["n"].as_u64().unwrap() as usize;
        let s = Spectrum {
            hz: grid(n, 30.0, 20000.0),
            db: vec![-30.0; n],
        };
        let r = Spectrum {
            hz: grid(10, 30.0, 20000.0),
            db: vec![if n == 8 { -20.0 } else { -30.0 }; 10],
        };
        same_fit(
            &match_eq(&s, &r, MatchOptions::default()),
            &c["fit"],
            &format!("small {n}"),
        );
    }
}
