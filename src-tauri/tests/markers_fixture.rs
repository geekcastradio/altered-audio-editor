// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust marker and chapter export rules against
//! `shared-fixtures/markers.json`, which the TypeScript tests read too.

use altered_audio_editor_lib::markers::*;
use altered_audio_editor_lib::session::model::MarkerKind;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/markers.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn kind(v: &Value) -> MarkerKind {
    serde_json::from_value(v.clone()).unwrap()
}

fn sets(fx: &Value) -> HashMap<&'static str, Vec<Mark>> {
    let all: Vec<Mark> = serde_json::from_value(fx["markers"][0].clone()).unwrap();
    let pick = |ids: &[&str]| {
        ids.iter()
            .map(|i| all.iter().find(|m| m.id == *i).unwrap().clone())
            .collect::<Vec<_>>()
    };
    let mut extra = pick(&["d", "e"]);
    extra.push(Mark {
        id: "k".into(),
        name: "Marker 3".into(),
        position: 1_440_000,
        ..all[4].clone()
    });
    HashMap::from([
        ("all", all.clone()),
        ("none", vec![]),
        ("one", pick(&["a"])),
        ("few", pick(&["a", "b", "c"])),
        ("twoAtOnce", extra),
    ])
}

fn ids(v: &[Mark]) -> Vec<String> {
    v.iter().map(|m| m.id.clone()).collect()
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn near(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-9, "{what}: {a} vs {b}");
}

fn opt(v: &Value) -> Option<f64> {
    v.as_f64()
}

fn fmt_of(s: &str) -> ChapterFormat {
    match s {
        "timestamps" => ChapterFormat::Timestamps,
        "csv" => ChapterFormat::Csv,
        _ => ChapterFormat::PodcastJson,
    }
}

#[test]
fn kinds_and_new_markers() {
    let fx = fixture();
    for c in fx["kinds"].as_array().unwrap() {
        let k = kind(&c["k"]);
        assert_eq!(kind_label(k), c["label"].as_str().unwrap());
        assert_eq!(kind_color(k), c["color"].as_str().unwrap());
        assert_eq!(marker_css(k, None), c["css"].as_str().unwrap());
        assert_eq!(
            marker_css(k, Some("#123456")),
            c["cssOwn"].as_str().unwrap()
        );
        assert_eq!(marker_css(k, Some("")), c["cssEmpty"].as_str().unwrap());
    }
    for c in fx["news"].as_array().unwrap() {
        let a = c["args"].as_array().unwrap();
        let m = new_marker(
            a[0].as_str().unwrap(),
            a[1].as_str().unwrap(),
            a[2].as_f64().unwrap(),
            a[3].as_f64().unwrap(),
            kind(&a[4]),
        );
        assert_eq!(
            m.position as f64,
            c["m"]["position"].as_f64().unwrap(),
            "{c}"
        );
        assert_eq!(
            m.duration as f64,
            c["m"]["duration"].as_f64().unwrap(),
            "{c}"
        );
        assert_eq!(m.id, c["m"]["id"].as_str().unwrap());
        assert!(m.chapter.is_none() && m.contracted_duration.is_none() && m.description.is_empty());
    }
}

#[test]
fn picking_and_searching() {
    let fx = fixture();
    let all = sets(&fx);
    for c in fx["names"].as_array().unwrap() {
        assert_eq!(
            next_marker_name(
                &all[c["set"].as_str().unwrap()],
                c["base"].as_str().unwrap()
            ),
            c["out"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["picks"].as_array().unwrap() {
        let want: HashSet<String> = strs(&c["ids"]).into_iter().collect();
        assert_eq!(
            ids(&picked_markers(&all[c["set"].as_str().unwrap()], &want)),
            strs(&c["out"]),
            "{c}"
        );
    }
    for c in fx["between"].as_array().unwrap() {
        assert_eq!(
            markers_between(
                &all[c["set"].as_str().unwrap()],
                c["a"].as_str().unwrap(),
                c["b"].as_str().unwrap()
            ),
            strs(&c["out"]),
            "{c}"
        );
    }
    for c in fx["labels"].as_array().unwrap() {
        assert_eq!(
            delete_label(c["n"].as_u64().unwrap() as usize),
            c["out"].as_str().unwrap()
        );
    }
    for c in fx["filters"].as_array().unwrap() {
        assert_eq!(
            ids(&filter_markers(
                &all[c["set"].as_str().unwrap()],
                c["q"].as_str().unwrap()
            )),
            strs(&c["out"]),
            "{c}"
        );
    }
    for c in fx["hits"].as_array().unwrap() {
        let ms = &all[c["set"].as_str().unwrap()];
        let spp = c["spp"].as_f64().unwrap();
        let got = marker_at(
            ms,
            c["x"].as_f64().unwrap(),
            |v| v / spp,
            c["tol"].as_f64().unwrap(),
        );
        assert_eq!(got.map(|i| i as u64), c["out"].as_u64(), "{c}");
    }
    let m = &all["all"][0];
    assert_eq!(marker_end(m), m.position + m.duration);
    assert_eq!(marker_end(&all["all"][2]), 960_000 + 96_000);
}

#[test]
fn ranges_kind_changes_and_playlists() {
    let fx = fixture();
    for c in fx["ranges"].as_array().unwrap() {
        let sel = c["sel"]
            .as_object()
            .map(|s| (s["start"].as_f64().unwrap(), s["end"].as_f64().unwrap()));
        assert_eq!(
            range_length_for(c["pos"].as_f64().unwrap(), sel, c["rate"].as_f64().unwrap()),
            c["out"].as_f64().unwrap(),
            "{c}"
        );
    }
    for c in fx["kindChanges"].as_array().unwrap() {
        let got = kind_change(
            c["duration"].as_f64().unwrap(),
            opt(&c["contracted"]),
            kind(&c["kind"]),
            c["rate"].as_f64().unwrap(),
        );
        let want = &c["out"];
        assert_eq!(got.kind, kind(&want["kind"]), "{c}");
        assert_eq!(
            got.contracted_duration,
            opt(&want["contracted_duration"]),
            "{c}"
        );
        assert_eq!(
            got.clear_contracted,
            want["clear_contracted"].as_bool().unwrap_or(false),
            "{c}"
        );
    }
    for c in fx["playlists"].as_array().unwrap() {
        let pl: Vec<String> = if c["pl"].is_null() {
            vec![]
        } else {
            strs(&c["pl"])
        };
        assert_eq!(
            toggle_playlist(&pl, c["id"].as_str().unwrap(), c["d"].as_u64().unwrap()),
            strs(&c["out"]),
            "{c}"
        );
    }
}

#[test]
fn chapter_lists_and_files() {
    let fx = fixture();
    let all = sets(&fx);
    let rate = 48000.0;
    for c in fx["chapters"].as_array().unwrap() {
        let got = chapters_of(
            &all[c["set"].as_str().unwrap()],
            rate,
            c["which"] == "chapters",
        );
        let want = c["out"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{c}");
        for (g, w) in got.iter().zip(want) {
            assert_eq!(g.title, w["title"].as_str().unwrap());
            near(g.start, w["start"].as_f64().unwrap(), "start");
            assert_eq!(g.end, opt(&w["end"]));
            assert_eq!(g.kind, w["kind"].as_str().unwrap());
            assert_eq!(g.description, w["description"].as_str().unwrap());
            assert_eq!(g.url.as_deref(), w["url"].as_str());
        }
    }
    for c in fx["renders"].as_array().unwrap() {
        let list = chapters_of(
            &all[c["set"].as_str().unwrap()],
            rate,
            c["which"] == "chapters",
        );
        let f = fmt_of(c["fmt"].as_str().unwrap());
        assert_eq!(render(&list, f), c["text"].as_str().unwrap(), "{c}");
        assert_eq!(warnings(&list, f), strs(&c["warnings"]), "{c}");
    }
    for c in fx["embeds"].as_array().unwrap() {
        let ms = &all[c["set"].as_str().unwrap()];
        let range = c["range"]
            .as_array()
            .map(|r| (r[0].as_f64().unwrap(), r[1].as_f64().unwrap()));
        for (got, want, with_url) in [
            (embed_list(ms, rate, range), &c["chapters"], true),
            (marker_list(ms, rate, range), &c["all"], false),
        ] {
            let want = want.as_array().unwrap();
            assert_eq!(got.len(), want.len(), "{c}");
            for (g, w) in got.iter().zip(want) {
                assert_eq!(g.title, w["title"].as_str().unwrap());
                assert_eq!(g.start_ms, w["startMs"].as_f64().unwrap(), "{c}");
                assert_eq!(g.end_ms, opt(&w["endMs"]), "{c}");
                if with_url {
                    assert_eq!(g.url.as_deref(), w["url"].as_str());
                } else {
                    assert_eq!(g.kind, w["kind"].as_str().unwrap());
                }
            }
        }
    }
}

#[test]
fn urls_tips_and_clock() {
    let fx = fixture();
    let all = sets(&fx);
    for c in fx["urls"].as_array().unwrap() {
        if let Some(t) = c["t"].as_str() {
            assert_eq!(is_chapter_url(t), c["ok"].as_bool().unwrap(), "{c}");
        } else {
            let m = all["all"]
                .iter()
                .find(|m| m.id == c["marker"].as_str().unwrap())
                .unwrap();
            assert_eq!(chapter_url(m), c["url"].as_str().unwrap());
        }
    }
    for c in fx["tips"].as_array().unwrap() {
        if let Some(t) = c["t"].as_str() {
            assert_eq!(title_tips(t), strs(&c["out"]), "{c}");
        } else {
            assert_eq!(
                clock(c["clock"].as_f64().unwrap()),
                c["out"].as_str().unwrap(),
                "{c}"
            );
        }
    }
    let _ = json!(null);
}
