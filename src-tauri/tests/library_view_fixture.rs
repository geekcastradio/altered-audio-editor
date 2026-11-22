// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust library view and Files list rules against
//! `shared-fixtures/library-view.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::library_view::*;
use serde_json::Value;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/library-view.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn view(v: &Value) -> ViewState {
    ViewState {
        group_by: v["groupBy"].as_str().unwrap().into(),
        sort_by: v["sortBy"].as_str().unwrap().into(),
        filter: v["filter"].as_str().unwrap().into(),
        view: v["view"].as_str().unwrap().into(),
        open: strs(&v["open"]),
    }
}

fn items(fx: &Value) -> Vec<Item> {
    fx["inputs"][0]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| Item {
            id: i["id"].as_str().unwrap().into(),
            name: i["name"].as_str().unwrap().into(),
            kind: i["kind"].as_str().unwrap().into(),
            added: i["added"].as_str().unwrap().into(),
        })
        .collect()
}

fn bins(fx: &Value, bi: usize) -> Vec<Bin> {
    fx["inputs"][0]["bins"][bi]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| Bin {
            id: b["id"].as_str().unwrap().into(),
            name: b["name"].as_str().unwrap().into(),
            items: strs(&b["items"]),
            color: b["color"].as_str().map(String::from),
        })
        .collect()
}

fn rows(v: &Value) -> Vec<FileRow> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|r| FileRow {
            path: r["path"].as_str().unwrap().into(),
            name: r["name"].as_str().unwrap().into(),
            status: r["status"].as_str().unwrap().into(),
            info: (!r["info"].is_null()).then(|| FileInfo {
                sample_rate: r["info"]["sampleRate"].as_f64().unwrap(),
                channels: r["info"]["channels"].as_f64().unwrap(),
                length: r["info"]["length"].as_f64().unwrap(),
                format: r["info"]["format"].as_str().unwrap().into(),
                codec: r["info"]["codec"].as_str().unwrap().into(),
            }),
        })
        .collect()
}

#[test]
fn saved_views() {
    for c in fixture()["view"].as_array().unwrap() {
        assert_eq!(sanitize_view(&c["raw"]), view(&c["out"]), "{c}");
    }
}

#[test]
fn sections_and_folding() {
    let fx = fixture();
    let all = items(&fx);
    for c in fx["arrange"].as_array().unwrap() {
        let v = view(&c["v"]);
        let got = arrange(&bins(&fx, c["bi"].as_u64().unwrap() as usize), &all, &v);
        let want = c["out"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{c}");
        for (g, w) in got.iter().zip(want) {
            assert_eq!(g.id, w["id"].as_str().unwrap(), "{c}");
            assert_eq!(g.name, w["name"].as_str().unwrap());
            assert_eq!(g.color.as_deref(), w["color"].as_str());
            assert_eq!(g.bin.as_deref(), w["bin"].as_str());
            assert_eq!(
                g.items.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
                strs(&w["items"]),
                "{} {}",
                c["v"],
                g.id
            );
        }
    }
    for c in fx["fold"].as_array().unwrap() {
        let v = view(&c["v"]);
        let sections = arrange(&bins(&fx, c["bi"].as_u64().unwrap() as usize), &all, &v);
        assert_eq!(
            flat_ids(&sections, &strs(&c["collapsed"])),
            strs(&c["flat"]),
            "{c}"
        );
        assert_eq!(
            folded_ids(&sections, &v, false),
            strs(&c["folded"][0]),
            "{c}"
        );
        assert_eq!(
            folded_ids(&sections, &v, true),
            strs(&c["folded"][1]),
            "{c}"
        );
        for (s, t) in sections.iter().zip(c["toggles"].as_array().unwrap()) {
            assert_eq!(toggle_section(&v, &s.id, true), view(&t[0]), "{c}");
            assert_eq!(toggle_section(&v, &s.id, false), view(&t[1]), "{c}");
        }
    }
}

#[test]
fn files_list() {
    let fx = fixture();
    for c in fx["rows"].as_array().unwrap() {
        let got: Vec<String> = sort_items(
            &rows(&c["rows"]),
            c["key"].as_str(),
            c["desc"].as_bool().unwrap(),
        )
        .into_iter()
        .map(|r| r.path)
        .collect();
        assert_eq!(got, strs(&c["out"]), "{} {}", c["key"], c["desc"]);
    }
    for c in fx["match"].as_array().unwrap() {
        let all = rows(&fx["rows"][0]["rows"]);
        let got: Vec<String> = all
            .iter()
            .filter(|r| matches(r, c["q"].as_str().unwrap()))
            .map(|r| r.path.clone())
            .collect();
        assert_eq!(got, strs(&c["out"]), "{c}");
    }
}
