// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust Batch Convert naming and saved-chain rules against
//! `shared-fixtures/batch-rules.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::batch_rules::*;
use altered_audio_editor_lib::output_format::Kind;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/batch-rules.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn kind(s: &str) -> Kind {
    match s {
        "flac" => Kind::Flac,
        "mp3" => Kind::Mp3,
        "m4a" => Kind::M4a,
        "aac" => Kind::Aac,
        _ => Kind::Wav,
    }
}

fn ctx(v: &Value) -> NameContext {
    NameContext {
        number: v["number"].as_i64().unwrap(),
        count: v["count"].as_i64().unwrap(),
        year: v["y"].as_i64().unwrap(),
        month: v["m"].as_i64().unwrap(),
        day: v["d"].as_i64().unwrap(),
    }
}

/// A level as the fixture writes it.
fn level_json(l: &Level) -> Value {
    json!({
        "normalize": l.normalize, "normalizeDb": l.normalize_db, "fadeInMs": l.fade_in_ms, "fadeOutMs": l.fade_out_ms,
        "loudness": l.loudness, "loudnessLufs": l.loudness_lufs, "ceilingDb": l.ceiling_db,
        "trim": l.trim, "trimDb": l.trim_db, "trimKeepMs": l.trim_keep_ms,
    })
}

fn level_of(v: &Value) -> Level {
    Level {
        normalize: v["normalize"].as_bool().unwrap(),
        normalize_db: f(&v["normalizeDb"]),
        fade_in_ms: f(&v["fadeInMs"]),
        fade_out_ms: f(&v["fadeOutMs"]),
        loudness: v["loudness"].as_bool().unwrap(),
        loudness_lufs: f(&v["loudnessLufs"]),
        ceiling_db: f(&v["ceilingDb"]),
        trim: v["trim"].as_bool().unwrap(),
        trim_db: f(&v["trimDb"]),
        trim_keep_ms: f(&v["trimKeepMs"]),
    }
}

/// Equal as JSON, with every number compared as a float (0 and 0.0 are the same).
fn same(a: &Value, b: &Value, what: &str) {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => assert!(
            (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() < 1e-9,
            "{what}: {a} vs {b}"
        ),
        (Value::Array(x), Value::Array(y)) => {
            assert_eq!(x.len(), y.len(), "{what}: length");
            for (p, q) in x.iter().zip(y) {
                same(p, q, what);
            }
        }
        (Value::Object(x), Value::Object(y)) => {
            assert_eq!(
                x.keys().collect::<Vec<_>>(),
                y.keys().collect::<Vec<_>>(),
                "{what}: keys"
            );
            for (k, p) in x {
                same(p, &y[k], what);
            }
        }
        _ => assert_eq!(a, b, "{what}"),
    }
}

fn shaping_json(s: Option<Shaping>) -> Value {
    match s {
        None => Value::Null,
        Some(s) => json!({
            "normalizeDb": s.normalize_db, "fadeInMs": s.fade_in_ms, "fadeOutMs": s.fade_out_ms, "loudnessLufs": s.loudness_lufs,
            "ceilingDb": s.ceiling_db, "trimDb": s.trim_db, "trimKeepMs": s.trim_keep_ms,
        }),
    }
}

#[test]
fn levels_and_settings() {
    let fx = fixture();
    for c in fx["levels"].as_array().unwrap() {
        let l = sanitize_level(&c["raw"]);
        same(&level_json(&l), &c["out"], "level");
        same(&shaping_json(shaping_for(&l)), &c["shaping"], "shaping");
    }
    for c in fx["shapeIn"].as_array().unwrap() {
        same(
            &shaping_json(shaping_for(&level_of(&c["level"]))),
            &c["out"],
            "shaping in",
        );
    }
    for (i, c) in fx["batches"].as_array().unwrap().iter().enumerate() {
        let s = sanitize_batch(&c["raw"]);
        let got = json!({
            "naming": if s.naming == Naming::Template { "template" } else { "affix" },
            "prefix": s.prefix, "suffix": s.suffix, "process": s.process, "level": level_json(&s.level), "template": s.template,
            "folder": s.folder, "overwrite": s.overwrite, "removeDone": s.remove_done, "copyTags": s.copy_tags,
            "tags": { "title": s.tags[0], "artist": s.tags[1], "album": s.tags[2], "year": s.tags[3], "genre": s.tags[4], "comment": s.tags[5] },
        });
        same(&got, &c["out"], &format!("batch {i}"));
    }
    for c in fx["eff"].as_array().unwrap() {
        let naming = if c["naming"] == "template" {
            Naming::Template
        } else {
            Naming::Affix
        };
        let got = effective_template(
            naming,
            c["prefix"].as_str().unwrap(),
            c["suffix"].as_str().unwrap(),
            c["template"].as_str().unwrap(),
        );
        assert_eq!(got, c["out"].as_str().unwrap());
    }
}

#[test]
fn names() {
    let fx = fixture();
    for c in fx["expand"].as_array().unwrap() {
        let got = expand_template(
            c["template"].as_str().unwrap(),
            c["stem"].as_str().unwrap(),
            kind(c["kind"].as_str().unwrap()),
            &ctx(&c["ctx"]),
        );
        assert_eq!(
            got,
            c["out"].as_str().unwrap(),
            "{} / {}",
            c["template"],
            c["stem"]
        );
    }
    for c in fx["dests"].as_array().unwrap() {
        let got = dest_for(
            c["source"].as_str().unwrap(),
            c["template"].as_str().unwrap(),
            c["folder"].as_str(),
            kind(c["kind"].as_str().unwrap()),
            &ctx(&c["ctx"]),
        );
        assert_eq!(
            got,
            c["out"].as_str().unwrap(),
            "{} / {}",
            c["source"],
            c["folder"]
        );
    }
    for c in fx["fileNames"].as_array().unwrap() {
        assert_eq!(
            file_name(c["p"].as_str().unwrap()),
            c["out"].as_str().unwrap()
        );
    }
    for c in fx["tagCases"].as_array().unwrap() {
        let mut fields: TagFields = Default::default();
        for (i, k) in TAG_KEYS.iter().enumerate() {
            fields[i] = c["fields"][k].as_str().unwrap().to_string();
        }
        let got = tags_for(
            &fields,
            c["source"].as_str().unwrap(),
            kind(c["kind"].as_str().unwrap()),
            &ctx(&c["ctx"]),
        );
        match got {
            None => assert!(c["out"].is_null()),
            Some(list) => {
                let want = c["out"].as_object().unwrap();
                assert_eq!(list.len(), want.len());
                for (k, v) in list {
                    assert_eq!(v, want[k].as_str().unwrap(), "{k}");
                }
            }
        }
    }
}

fn rows(v: &Value) -> Vec<Row> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let (id, source) = if r.is_array() {
                (&r[0], &r[1])
            } else {
                (&r["id"], &r["source"])
            };
            Row {
                id: id.as_str().unwrap().to_string(),
                source: source.as_str().unwrap().to_string(),
            }
        })
        .collect()
}

#[test]
fn the_list() {
    let fx = fixture();
    for (i, c) in fx["conflictCases"].as_array().unwrap().iter().enumerate() {
        let dests: Vec<(String, String)> = c["dests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| {
                (
                    d[0].as_str().unwrap().to_string(),
                    d[1].as_str().unwrap().to_string(),
                )
            })
            .collect();
        let got = conflicts(&rows(&c["rows"]), &dests, c["overwrite"].as_bool().unwrap());
        let want: Vec<(String, String)> = c["out"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                (
                    e[0].as_str().unwrap().to_string(),
                    e[1].as_str().unwrap().to_string(),
                )
            })
            .collect();
        assert_eq!(got, want, "conflicts {i}");
    }
    for c in fx["addCases"].as_array().unwrap() {
        let mut n = 0;
        let paths: Vec<String> = c["paths"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_string())
            .collect();
        let (got, added) = add_rows(&rows(&c["rows"]), &paths, || {
            n += 1;
            format!("id{n}")
        });
        assert_eq!(got, rows(&c["out"]["rows"]));
        assert_eq!(added as u64, c["out"]["added"].as_u64().unwrap());
    }
}

fn chain_json(c: &Chain) -> Value {
    json!({ "name": c.name, "process": c.process, "level": level_json(&c.level), "formatKind": c.format.as_ref().map(|f| f.kind.as_str()) })
}

fn chains_json(list: &[Chain]) -> Value {
    Value::Array(list.iter().map(chain_json).collect())
}

fn chain_of(v: &Value) -> Chain {
    Chain {
        name: v["name"].as_str().unwrap().to_string(),
        process: v["process"].as_str().map(str::to_string),
        level: level_of(&v["level"]),
        format: v["formatKind"].as_str().map(|k| {
            altered_audio_editor_lib::output_format::sanitize_format(&json!({ "kind": k }))
        }),
    }
}

#[test]
fn chains() {
    let fx = fixture();
    for (i, c) in fx["chains"].as_array().unwrap().iter().enumerate() {
        same(
            &chains_json(&sanitize_chains(&c["raw"])),
            &c["out"],
            &format!("chains {i}"),
        );
    }
    let settings = sanitize_batch(&fx["settingsRaw"]);
    for c in fx["chainFrom"].as_array().unwrap() {
        let got = chain_from(
            c["name"].as_str().unwrap(),
            &settings,
            c["withFormat"].as_bool().unwrap(),
        );
        same(&chain_json(&got), &c["out"], "chain from");
    }
    let mut saved: Vec<Chain> = Vec::new();
    for c in fx["saves"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let next = save_chain(&saved, &chain_from(name, &settings, name == "c"));
        match (&next, c["out"].is_null()) {
            (None, true) => {}
            (Some(list), false) => {
                same(&chains_json(list), &c["out"], "save");
                saved = list.clone();
            }
            _ => panic!("save {name}"),
        }
    }
    let mut big: Vec<Chain> = Vec::new();
    for i in 0..45 {
        if let Some(n) = save_chain(
            &big,
            &chain_from(&format!("N {}", i % 43), &settings, false),
        ) {
            big = n;
        }
    }
    same(&chains_json(&big), &fx["bigRun"], "big run");
    for c in fx["removes"].as_array().unwrap() {
        same(
            &chains_json(&remove_chain(&saved, c["name"].as_str().unwrap())),
            &c["out"],
            "remove",
        );
    }
    for c in fx["applies"].as_array().unwrap() {
        let a = apply_chain(&settings, &chain_of(&c["chain"]));
        let got = json!({ "process": a.process, "level": level_json(&a.level), "formatKind": a.format.kind.as_str() });
        same(&got, &c["out"], "apply");
    }
    for c in fx["describes"].as_array().unwrap() {
        assert_eq!(
            describe_chain(&chain_of(&c["chain"])),
            c["out"].as_str().unwrap(),
            "{}",
            c["chain"]["name"]
        );
    }
    same(
        &chains_json(&builtin_chains()),
        &fx["builtin"],
        "builtin chains",
    );
}
