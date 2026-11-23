// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the bars and beats ruler in Rust against `shared-fixtures/bar-ruler.json`,
//! which the TypeScript test reads too.

use altered_audio_editor_lib::bar_ruler::*;
use altered_audio_editor_lib::tempo_map::Tempo;
use altered_audio_editor_lib::timeline::Viewport;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/bar-ruler.json");
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

fn tempos(fx: &Value) -> Vec<Tempo> {
    fx["tempos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| serde_json::from_value(t.clone()).unwrap())
        .collect()
}

fn f(v: &Value, k: &str) -> f64 {
    v[k].as_f64().unwrap()
}

#[test]
fn format_and_parse() {
    let fx = fixture();
    let ts = tempos(&fx);
    for c in fx["fmt"].as_array().unwrap() {
        let got = format_bars(f(c, "s"), 48000.0, &ts[c["ti"].as_u64().unwrap() as usize]);
        assert_eq!(got, c["out"].as_str().unwrap(), "fmt {c}");
    }
    for c in fx["parse"].as_array().unwrap() {
        let got = parse_bars(
            c["text"].as_str().unwrap(),
            48000.0,
            &ts[c["ti"].as_u64().unwrap() as usize],
        )
        .map_or(Value::Null, |x| json!(x));
        ok(&got, &c["out"], &format!("parse {c}"));
    }
}

#[test]
fn ticks_and_grid() {
    let fx = fixture();
    let ts = tempos(&fx);
    for c in fx["ticks"].as_array().unwrap() {
        let v = Viewport {
            spp: f(c, "spp"),
            scroll: f(c, "scroll"),
        };
        let gap = c.get("gap").and_then(Value::as_f64).unwrap_or(70.0);
        let got: Vec<Value> = bar_ticks(
            v,
            900.0,
            48000.0,
            &ts[c["ti"].as_u64().unwrap() as usize],
            gap,
        )
        .into_iter()
        .map(|t| json!({ "x": t.x, "label": t.label }))
        .collect();
        ok(
            &Value::Array(got),
            &c["out"],
            &format!("ticks {} {} {}", c["ti"], c["spp"], c["scroll"]),
        );
    }
    for c in fx["grid"].as_array().unwrap() {
        let v = Viewport {
            spp: f(c, "spp"),
            scroll: f(c, "scroll"),
        };
        let got: Vec<Value> = grid_lines(
            v,
            900.0,
            48000.0,
            &ts[c["ti"].as_u64().unwrap() as usize],
            6.0,
        )
        .into_iter()
        .map(|(x, bar)| json!({ "x": x, "bar": bar }))
        .collect();
        ok(
            &Value::Array(got),
            &c["out"],
            &format!("grid {} {} {}", c["ti"], c["spp"], c["scroll"]),
        );
    }
}
