// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust playlist and review rules must agree with
//! `shared-fixtures/playlist-review.json`, the same file the TypeScript tests read.

use altered_audio_editor_lib::playlist::*;
use altered_audio_editor_lib::review::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/playlist-review.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn markers(v: &Value) -> Vec<Marker> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|m| Marker {
            id: m["id"].as_str().unwrap().into(),
            name: m["name"].as_str().unwrap().into(),
            position: m["position"].as_f64().unwrap(),
            duration: m["duration"].as_f64().unwrap(),
        })
        .collect()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn ids(items: &[&Marker]) -> Vec<String> {
    items.iter().map(|m| m.id.clone()).collect()
}

#[test]
fn playlist_matches() {
    let f = fixture();
    let ms = markers(&f["markers"]);
    for c in f["playlist"].as_array().unwrap() {
        let pl = if c["playlist"].is_null() {
            Vec::new()
        } else {
            strings(&c["playlist"])
        };
        let items = playlist_items(&ms, &pl);
        assert_eq!(ids(&items), strings(&c["items"]), "{c}");
        assert_eq!(ids(&available(&ms, &pl)), strings(&c["available"]), "{c}");
        let want_added = c["added"].as_array().unwrap();
        let a1: Vec<String> = ["a", "e", "e", "zzz"].map(String::from).to_vec();
        assert_eq!(with_added(&ms, &pl, &a1), strings(&want_added[0]), "{c}");
        assert_eq!(with_added(&ms, &pl, &[]), strings(&want_added[1]), "{c}");
        for (id, want) in ["a", "b", "nope"]
            .iter()
            .zip(c["removed"].as_array().unwrap())
        {
            assert_eq!(with_removed(&ms, &pl, id), strings(want), "{c}");
        }
        for m in c["moved"].as_array().unwrap() {
            let got = with_moved(
                &ms,
                &pl,
                m["id"].as_str().unwrap(),
                m["by"].as_i64().unwrap(),
            );
            assert_eq!(got, strings(&m["want"]), "{c}");
        }
        for (i, want) in c["next"].as_array().unwrap().iter().enumerate() {
            let got = next_start(&items, i);
            match want {
                Value::Null => assert!(got.is_none(), "{c}"),
                w => {
                    let (idx, at) = got.unwrap();
                    assert_eq!(idx as u64, w["index"].as_u64().unwrap(), "{c}");
                    assert_eq!(at, w["at"].as_f64().unwrap(), "{c}");
                }
            }
        }
        assert_eq!(export_names(&items), strings(&c["names"]), "{c}");
    }
}

#[test]
fn export_names_match() {
    for c in fixture()["exportNames"].as_array().unwrap() {
        let ms: Vec<Marker> = strings(&c["names"])
            .into_iter()
            .enumerate()
            .map(|(i, name)| Marker {
                id: format!("x{i}"),
                name,
                position: i as f64,
                duration: 1.0,
            })
            .collect();
        let refs: Vec<&Marker> = ms.iter().collect();
        assert_eq!(export_names(&refs), strings(&c["want"]), "{c}");
    }
}

fn kind_name(k: Option<ReviewKind>) -> Option<&'static str> {
    k.map(|k| match k {
        ReviewKind::Cut => "cut",
        ReviewKind::Retake => "retake",
        ReviewKind::Great => "great",
    })
}

#[test]
fn review_matches() {
    let f = fixture();
    let ms = markers(&f["markers"]);
    for c in f["reviewKinds"].as_array().unwrap() {
        assert_eq!(
            kind_name(review_kind(c["name"].as_str().unwrap())),
            c["want"].as_str(),
            "{c}"
        );
    }
    for c in f["reviewMarks"].as_array().unwrap() {
        let filter = match c["filter"].as_str().unwrap() {
            "cut" => Some(ReviewKind::Cut),
            "retake" => Some(ReviewKind::Retake),
            "great" => Some(ReviewKind::Great),
            _ => None,
        };
        assert_eq!(ids(&review_marks(&ms, filter)), strings(&c["want"]), "{c}");
    }
    let want = &f["reviewCounts"];
    let (cut, retake, great) = review_counts(&ms);
    assert_eq!(cut as u64, want["cut"].as_u64().unwrap());
    assert_eq!(retake as u64, want["retake"].as_u64().unwrap());
    assert_eq!(great as u64, want["great"].as_u64().unwrap());
    let sorted = review_marks(&ms, None);
    for c in f["stepMark"].as_array().unwrap() {
        let got = step_mark(
            &sorted,
            c["at"].as_f64().unwrap(),
            c["dir"].as_i64().unwrap() as i32,
        );
        assert_eq!(got.map(|i| i as u64), c["want"].as_u64(), "{c}");
    }
}
