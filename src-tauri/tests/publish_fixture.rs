// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the Rust Publish Package builder against
//! `shared-fixtures/publish.json` (and the markers in `markers.json`), which
//! the TypeScript test reads too.

use altered_audio_editor_lib::markers::{Chapter, Mark};
use altered_audio_editor_lib::publish::*;
use altered_audio_editor_lib::session::model::MarkerKind;
use serde_json::Value;

fn read(name: &str) -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn info(v: &Value) -> EpisodeInfo {
    EpisodeInfo {
        title: v["title"].as_str().unwrap().into(),
        description: v["description"].as_str().unwrap().into(),
        season: v["season"].as_f64(),
        episode: v["episode"].as_f64(),
        kind: match v["type"].as_str().unwrap() {
            "trailer" => EpisodeType::Trailer,
            "bonus" => EpisodeType::Bonus,
            _ => EpisodeType::Full,
        },
        explicit: v["explicit"].as_bool().unwrap(),
    }
}

fn set(all: &[Mark], name: &str) -> Vec<Mark> {
    let kind = |k: MarkerKind, keep: bool| {
        all.iter()
            .filter(|m| (m.kind == k) == keep)
            .cloned()
            .collect::<Vec<_>>()
    };
    match name {
        "all" => all.to_vec(),
        "none" => vec![],
        "chaptersOnly" => kind(MarkerKind::Chapter, true),
        "adsOnly" => kind(MarkerKind::AdSlot, true),
        "clipsOnly" => kind(MarkerKind::SocialClip, true),
        _ => kind(MarkerKind::Chapter, false),
    }
}

#[test]
fn episode_checks() {
    for c in read("publish.json")["problems"].as_array().unwrap() {
        let want: Vec<String> = c["out"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap().into())
            .collect();
        assert_eq!(episode_problems(&info(&c["info"])), want, "{c}");
    }
}

#[test]
fn package_files_match() {
    let all: Vec<Mark> =
        serde_json::from_value(read("markers.json")["markers"][0].clone()).unwrap();
    for c in read("publish.json")["pub"].as_array().unwrap() {
        let got = package_files(
            &info(&c["info"]),
            c["audio"].as_str().unwrap(),
            c["dur"].as_f64().unwrap(),
            &set(&all, c["set"].as_str().unwrap()),
            48000.0,
            c["checked"].as_str().unwrap(),
        );
        let want = c["files"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{} {}", c["set"], c["i"]);
        for (g, w) in got.iter().zip(want) {
            assert_eq!(g.name, w["name"].as_str().unwrap());
            assert_eq!(
                g.content,
                w["content"].as_str().unwrap(),
                "{} {} {}",
                c["set"],
                c["i"],
                g.name
            );
        }
    }
}

#[test]
fn vtt_matches() {
    let fx = read("publish.json");
    for c in fx["vtt"].as_array().unwrap() {
        let chapters: Vec<Chapter> = c["chaps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| Chapter {
                title: x["title"].as_str().unwrap().into(),
                start: x["start"].as_f64().unwrap(),
                end: x["end"].as_f64(),
                kind: String::new(),
                description: String::new(),
                url: None,
            })
            .collect();
        assert_eq!(
            chapters_vtt(&chapters, c["d"].as_f64().unwrap()),
            c["out"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["vttTime"].as_array().unwrap() {
        assert_eq!(
            vtt_time(c["t"].as_f64().unwrap()),
            c["out"].as_str().unwrap(),
            "{c}"
        );
    }
}
