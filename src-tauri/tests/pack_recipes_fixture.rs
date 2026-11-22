// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks preset packs and Batch Convert recipes in Rust against
//! `shared-fixtures/pack-recipes.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::batch_recipes::*;
use altered_audio_editor_lib::effect_presets::{Bank, EffectPreset};
use altered_audio_editor_lib::preset_pack::*;
use altered_audio_editor_lib::voice_chain::builtin_presets;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/pack-recipes.json");
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

fn read_json(r: &Result<ReadPack, String>) -> Value {
    match r {
        Ok(p) => json!({ "chains": p.chains, "bank": bank_json(&p.bank) }),
        Err(e) => json!({ "error": e }),
    }
}

/// The bank as the fixture wrote it, in its order (no sorting or limits).
fn raw_bank(v: &Value) -> Bank {
    let mut out = Bank::new();
    for (effect, list) in v.as_object().unwrap() {
        let items = list.as_array().unwrap().iter().map(|p| EffectPreset {
            name: p["name"].as_str().unwrap().into(),
            params: p["params"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.as_f64().unwrap()))
                .collect(),
            data: p["data"].as_str().map(String::from),
        });
        out.insert(effect.clone(), items.collect());
    }
    out
}

#[test]
fn reading_and_merging() {
    let fx = fixture();
    let mine_chains: Vec<Value> = fx["mine"]["chainsMine"].as_array().unwrap().clone();
    let mine_bank = raw_bank(&fx["mine"]["bankMine"]);
    for c in fx["reads"].as_array().unwrap() {
        let r = read_pack(c["text"].as_str().unwrap());
        ok(&read_json(&r), &c["out"], &format!("read {}", c["text"]));
    }
    for c in fx["merges"].as_array().unwrap() {
        let pack = read_pack(c["text"].as_str().unwrap()).unwrap();
        let (chains, bank) = if c["k"] == 0 {
            (mine_chains.clone(), mine_bank.clone())
        } else {
            (vec![], Default::default())
        };
        let m = merge_pack(&chains, &bank, &pack, &["Podcast Voice".to_string()]);
        let got = json!({
            "chains": m.chains, "bank": bank_json(&m.bank), "addedChains": m.added_chains,
            "addedEffects": m.added_effects, "renamed": m.renamed, "full": m.full,
        });
        ok(&got, &c["out"], &format!("merge {} {}", c["k"], c["text"]));
    }
}

#[test]
fn packing_and_names() {
    let fx = fixture();
    let mine_chains: Vec<Value> = fx["mine"]["chainsMine"].as_array().unwrap().clone();
    let mine_bank = raw_bank(&fx["mine"]["bankMine"]);
    for c in fx["packed"].as_array().unwrap() {
        let text = if c["k"] == 0 {
            pack_presets(&mine_chains, &mine_bank)
        } else {
            pack_presets(&[], &Default::default())
        };
        assert!(text.ends_with("}\n"));
        let got: Value = serde_json::from_str(&text).unwrap();
        let want: Value = serde_json::from_str(c["text"].as_str().unwrap()).unwrap();
        ok(&got, &want, "packed");
    }
    for c in fx["names"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap().to_string();
        let n = c["n"].as_u64().unwrap();
        let cut = |i: u64| {
            let suffix = format!(" ({i})");
            altered_audio_editor_lib::js_compat::js_slice_units(&name, 60 - suffix.len()) + &suffix
        };
        let taken: Vec<String> = std::iter::once(name.clone())
            .chain((0..n).map(|i| cut(i + 2)))
            .collect();
        let got = unique_name(&name, &|c| taken.iter().any(|t| t == c));
        assert_eq!(got, c["out"].as_str().unwrap(), "{c}");
    }
}

#[test]
fn recipes() {
    let fx = fixture();
    let r = &fx["recipes"];
    let groups: serde_json::Map<String, Value> = RECIPE_GROUPS
        .iter()
        .map(|(k, v)| (k.to_string(), json!(v)))
        .collect();
    ok(&Value::Object(groups), &r["groups"], "groups");
    ok(&json!(RECIPE_ONLY_GROUPS), &r["only"], "only");
    ok(&Value::Array(batch_recipes()), &r["all"], "all");
    ok(&Value::Array(build_recipes(&[])), &r["none"], "none");
    let mut taken: Vec<Value> = builtin_presets().into_iter().take(3).collect();
    taken.push(json!({ "name": "Soft", "group": "x", "chain": {} }));
    taken.extend(batch_recipes().into_iter().take(5));
    ok(&Value::Array(build_recipes(&taken)), &r["taken"], "taken");
}
