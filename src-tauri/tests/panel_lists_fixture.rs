// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust Metadata panel, soundboard pad and library bin rules against
//! `shared-fixtures/panel-lists.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::metadata_rules::*;
use altered_audio_editor_lib::pads_bins::*;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

fn fixture() -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/panel-lists.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

fn tag_set(v: &Value) -> TagSet {
    TagSet {
        name: v["name"].as_str().unwrap().to_string(),
        tags: v["tags"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, x)| (k.clone(), x.as_str().unwrap().to_string()))
            .collect(),
    }
}

fn same_sets(got: &[TagSet], want: &Value, what: &str) {
    let want: Vec<TagSet> = want.as_array().unwrap().iter().map(tag_set).collect();
    assert_eq!(got, want.as_slice(), "{what}");
}

#[test]
fn tag_sets() {
    let fx = fixture();
    for (i, c) in fx["sanitize"].as_array().unwrap().iter().enumerate() {
        same_sets(
            &sanitize_sets(&c["raw"]),
            &c["out"],
            &format!("sanitize {i}"),
        );
    }
    let mut sets: Vec<TagSet> = Vec::new();
    for (i, c) in fx["upserts"].as_array().unwrap().iter().enumerate() {
        let values: BTreeMap<String, Option<String>> = c["values"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().map(str::to_string)))
            .collect();
        sets = upsert_set(&sets, c["name"].as_str().unwrap(), &values);
        same_sets(&sets, &c["out"], &format!("upsert {i}"));
    }
    for c in fx["removes"].as_array().unwrap() {
        same_sets(
            &remove_set(&sets, c["name"].as_str().unwrap()),
            &c["out"],
            "remove",
        );
    }
    let mut big: Vec<TagSet> = Vec::new();
    for i in 0..105 {
        let values = BTreeMap::from([("title".to_string(), Some("t".to_string()))]);
        big = upsert_set(&big, &format!("Item {}", (i * 13) % 105), &values);
    }
    same_sets(&big, &fx["bigRun"], "big run");
}

#[test]
fn checks_and_lists() {
    let fx = fixture();
    for c in fx["checks"]["year"].as_array().unwrap() {
        assert_eq!(
            check_year(c["v"].as_str().unwrap()),
            c["out"].as_str(),
            "{}",
            c["v"]
        );
    }
    for c in fx["checks"]["track"].as_array().unwrap() {
        assert_eq!(
            check_track(c["v"].as_str().unwrap()),
            c["out"].as_str(),
            "{}",
            c["v"]
        );
    }
    for c in fx["checks"]["problems"].as_array().unwrap() {
        let got = problems(c["y"].as_str().unwrap(), c["t"].as_str().unwrap());
        let want: Vec<&str> = c["out"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m.as_str().unwrap())
            .collect();
        assert_eq!(got, want);
    }
    for c in fx["baseNames"].as_array().unwrap() {
        assert_eq!(
            base_name(c["p"].as_str().unwrap()),
            c["out"].as_str().unwrap()
        );
    }
    let entries: Vec<Entry> = fx["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let t = |k: &str| e["tags"][k].as_str().map(str::to_string);
            Entry {
                path: e["path"].as_str().unwrap().to_string(),
                title: t("title"),
                artist: t("artist"),
                album: t("album"),
                genre: t("genre"),
            }
        })
        .collect();
    for c in fx["filter"].as_array().unwrap() {
        let got: Vec<&str> = filter_entries(&entries, c["q"].as_str().unwrap())
            .iter()
            .map(|e| e.path.as_str())
            .collect();
        let want: Vec<&str> = c["out"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap())
            .collect();
        assert_eq!(got, want, "{}", c["q"]);
    }
    for c in fx["overlays"].as_array().unwrap() {
        let tags: Map<String, Value> = c["tags"].as_object().unwrap().clone();
        assert_eq!(Value::Object(overlay(&tags, &tag_set(&c["set"]))), c["out"]);
    }
    for c in fx["adds"].as_array().unwrap() {
        assert_eq!(
            add_paths(&strings(&c["have"]), &strings(&c["more"])),
            strings(&c["out"])
        );
    }
}

fn pad(v: &Value) -> Pad {
    Pad {
        path: v["path"].as_str().map(str::to_string),
        label: v["label"].as_str().unwrap().to_string(),
    }
}

#[test]
fn pads() {
    let fx = fixture();
    for (i, c) in fx["pads"].as_array().unwrap().iter().enumerate() {
        let want: Vec<Pad> = c["out"].as_array().unwrap().iter().map(pad).collect();
        assert_eq!(sanitize_pads(&c["raw"]), want, "pads {i}");
    }
    for c in fx["labels"].as_array().unwrap() {
        assert_eq!(
            pad_label(c["p"].as_str().unwrap()),
            c["out"].as_str().unwrap(),
            "{}",
            c["p"]
        );
    }
    let start: Vec<Pad> = (0..PAD_COUNT).map(|_| empty_pad()).collect();
    for c in fx["withs"].as_array().unwrap() {
        let want: Vec<Pad> = c["out"].as_array().unwrap().iter().map(pad).collect();
        assert_eq!(
            with_pad(&start, c["i"].as_u64().unwrap() as usize, c["p"].as_str()),
            want
        );
    }
}

#[test]
fn bins() {
    let fx = fixture();
    assert_eq!(PALETTE.to_vec(), strings(&fx["palette"]));
    let starter: Vec<(String, Option<String>)> = fx["starterBins"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["name"].as_str().unwrap().to_string(),
                b["color"].as_str().map(str::to_string),
            )
        })
        .collect();
    let ours: Vec<(String, Option<String>)> = STARTER_BINS
        .iter()
        .map(|(n, c)| (n.to_string(), c.map(str::to_string)))
        .collect();
    assert_eq!(ours, starter);
    for c in fx["initialsOf"].as_array().unwrap() {
        assert_eq!(
            initials(c["n"].as_str().unwrap()),
            c["out"].as_str().unwrap(),
            "{}",
            c["n"]
        );
    }
    for c in fx["colors"].as_array().unwrap() {
        assert_eq!(
            default_color(c["n"].as_str().unwrap()),
            c["out"].as_str().unwrap(),
            "{}",
            c["n"]
        );
    }
    for c in fx["starter"].as_array().unwrap() {
        let got: Vec<(String, Option<String>)> = missing_starter_bins(&strings(&c["have"]))
            .iter()
            .map(|(n, c)| (n.to_string(), c.map(str::to_string)))
            .collect();
        let want: Vec<(String, Option<String>)> = c["out"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                (
                    b["name"].as_str().unwrap().to_string(),
                    b["color"].as_str().map(str::to_string),
                )
            })
            .collect();
        assert_eq!(got, want);
    }
}
