// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust editing rules must agree with `shared-fixtures/editor-rules.json`,
//! the same file the TypeScript tests read. Numbers are compared to 1e-9.

use altered_audio_editor_lib::editor_rules::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/editor-rules.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn view(v: &Value) -> View {
    View {
        spp: v["spp"].as_f64().unwrap(),
        scroll: v["scroll"].as_f64().unwrap(),
    }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn follow_effect_matches() {
    for c in fixture()["follow"].as_array().unwrap() {
        let ch = &c["change"];
        let change = ViewChange {
            spp: ch["spp"].as_f64(),
            scroll: ch["scroll"].as_f64(),
            playhead: ch["playhead"].as_f64(),
        };
        let want = match c["want"].as_str().unwrap() {
            "release" => FollowEffect::Release,
            "hold" => FollowEffect::Hold,
            _ => FollowEffect::None,
        };
        assert_eq!(follow_effect(view(&c["now"]), change), want, "{c}");
    }
}

#[test]
fn max_scroll_matches() {
    for c in fixture()["maxScroll"].as_array().unwrap() {
        let got = max_scroll(
            c["total"].as_f64().unwrap(),
            c["width"].as_f64().unwrap(),
            c["spp"].as_f64().unwrap(),
            c["liveEnd"].as_f64(),
            c["following"].as_bool().unwrap(),
            c["lead"].as_f64(),
        );
        assert!(close(got, c["want"].as_f64().unwrap()), "{c}: {got}");
    }
}

fn edge_name(e: Option<Edge>) -> Option<&'static str> {
    e.map(|e| match e {
        Edge::Start => "start",
        Edge::End => "end",
    })
}

#[test]
fn edge_at_matches() {
    for c in fixture()["edge"].as_array().unwrap() {
        let sel = (!c["sel"].is_null()).then(|| {
            (
                c["sel"]["start"].as_f64().unwrap(),
                c["sel"]["end"].as_f64().unwrap(),
            )
        });
        let grab = c["grab"].as_f64().unwrap_or(EDGE_GRAB_PX);
        let got = edge_name(edge_at(
            sel,
            c["x"].as_f64().unwrap(),
            view(&c["view"]),
            grab,
        ));
        assert_eq!(got, c["want"].as_str(), "{c}");
    }
}

#[test]
fn anchor_matches() {
    for c in fixture()["anchor"].as_array().unwrap() {
        let edge = if c["edge"] == "start" {
            Edge::Start
        } else {
            Edge::End
        };
        let sel = (
            c["sel"]["start"].as_f64().unwrap(),
            c["sel"]["end"].as_f64().unwrap(),
        );
        assert!(
            close(anchor_for(sel, edge), c["want"].as_f64().unwrap()),
            "{c}"
        );
    }
}

fn points(v: &Value) -> Vec<EnvelopePoint> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| EnvelopePoint {
            position: p["position"].as_f64().unwrap(),
            value: p["value"].as_f64().unwrap(),
            curve: p["curve"].as_str().unwrap().into(),
        })
        .collect()
}

#[test]
fn silence_span_matches() {
    for c in fixture()["silence"].as_array().unwrap() {
        let got = silence_span(
            &points(&c["points"]),
            c["len"].as_f64().unwrap(),
            c["from"].as_f64().unwrap(),
            c["to"].as_f64().unwrap(),
            c["ramp"].as_f64().unwrap(),
        );
        if c["want"].is_null() {
            assert!(got.is_none(), "{c}");
            continue;
        }
        let want = points(&c["want"]);
        let got = got.unwrap_or_else(|| panic!("expected points: {c}"));
        assert_eq!(got.len(), want.len(), "{c}");
        for (g, w) in got.iter().zip(&want) {
            assert!(
                close(g.position, w.position) && close(g.value, w.value),
                "{c}"
            );
            assert_eq!(g.curve, w.curve, "{c}");
        }
    }
}
