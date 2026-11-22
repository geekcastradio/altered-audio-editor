// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust timeline view rules must agree with `shared-fixtures/timeline.json`,
//! the same file the TypeScript tests read.

use altered_audio_editor_lib::timeline::*;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/timeline.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

fn vp(v: &Value) -> Viewport {
    Viewport {
        spp: f(&v["spp"]),
        scroll: f(&v["scroll"]),
    }
}

fn same_vp(a: Viewport, b: Viewport) -> bool {
    close(a.spp, b.spp) && close(a.scroll, b.scroll)
}

#[test]
fn view_maths_matches() {
    let fx = fixture();
    for c in fx["xOf"].as_array().unwrap() {
        assert!(close(x_of(f(&c["s"]), vp(&c["v"])), f(&c["want"])), "{c}");
    }
    for c in fx["sampleAt"].as_array().unwrap() {
        assert!(
            close(sample_at(f(&c["x"]), vp(&c["v"])), f(&c["want"])),
            "{c}"
        );
    }
    for c in fx["edgeScroll"].as_array().unwrap() {
        assert!(
            close(
                edge_scroll_px(f(&c["x"]), f(&c["left"]), f(&c["right"]), f(&c["band"])),
                f(&c["want"])
            ),
            "{c}"
        );
    }
    for c in fx["maxSpp"].as_array().unwrap() {
        assert!(
            close(
                max_spp_for(f(&c["w"]), f(&c["c"]), f(&c["r"])),
                f(&c["want"])
            ),
            "{c}"
        );
    }
    for c in fx["zoomAround"].as_array().unwrap() {
        let got = zoom_around(
            vp(&c["v"]),
            f(&c["factor"]),
            f(&c["ax"]),
            f(&c["maxSpp"]),
            f(&c["minSpp"]),
        );
        assert!(same_vp(got, vp(&c["want"])), "{c}");
    }
    for c in fx["fit"].as_array().unwrap() {
        assert!(
            same_vp(fit(f(&c["l"]), f(&c["w"]), f(&c["room"])), vp(&c["want"])),
            "{c}"
        );
    }
    for c in fx["overviewWindow"].as_array().unwrap() {
        let (x, w) = overview_window(vp(&c["v"]), f(&c["lw"]), f(&c["total"]), f(&c["sw"]));
        assert!(
            close(x, f(&c["want"]["x"])) && close(w, f(&c["want"]["w"])),
            "{c}"
        );
    }
    for c in fx["dragEdge"].as_array().unwrap() {
        let edge = if c["edge"] == "left" {
            Edge::Left
        } else {
            Edge::Right
        };
        let got = drag_overview_edge(
            vp(&c["v"]),
            edge,
            f(&c["x"]),
            f(&c["lw"]),
            f(&c["total"]),
            f(&c["sw"]),
        );
        assert!(same_vp(got, vp(&c["want"])), "{c}");
    }
    for c in fx["trackColor"].as_array().unwrap() {
        assert_eq!(
            default_track_color(f(&c["row"]) as i64),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["trackHeight"].as_array().unwrap() {
        assert!(
            close(
                track_height(f(&c["height"]), c["collapsed"].as_bool().unwrap()),
                f(&c["want"])
            ),
            "{c}"
        );
    }
}

#[test]
fn ruler_and_time_text_match() {
    let fx = fixture();
    for c in fx["rulerTicks"].as_array().unwrap() {
        let got = ruler_ticks(vp(&c["v"]), f(&c["w"]), f(&c["rate"]), f(&c["gap"]));
        let want = c["want"].as_array().unwrap();
        assert_eq!(got.len(), want.len(), "{c}");
        for (g, w) in got.iter().zip(want) {
            assert!(close(g.x, f(&w["x"])), "{c}");
            assert_eq!(g.label.as_deref(), w["label"].as_str(), "{c}");
        }
    }
    for c in fx["formatTime"].as_array().unwrap() {
        assert_eq!(
            format_time(f(&c["sec"]), f(&c["step"])),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["formatClock"].as_array().unwrap() {
        assert_eq!(
            format_clock(f(&c["s"]), f(&c["r"])),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["formatTimecode"].as_array().unwrap() {
        assert_eq!(
            format_timecode(f(&c["s"]), f(&c["r"]), f(&c["fps"])),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn peaks_match() {
    let fx = fixture();
    for c in fx["pickLevel"].as_array().unwrap() {
        let blocks: Vec<f64> = c["blocks"].as_array().unwrap().iter().map(f).collect();
        let got = pick_level(&blocks, f(&c["fpp"]));
        match c["want"].as_u64() {
            None => assert!(got.is_none(), "{c}"),
            // Equal blocks tie, so compare the block size, not the index.
            Some(w) => assert_eq!(blocks[got.expect("a level")], blocks[w as usize], "{c}"),
        }
    }
    for c in fx["peakRange"].as_array().unwrap() {
        let data: Vec<i16> = c["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| f(v) as i16)
            .collect();
        let (lo, hi) = peak_range(
            &data,
            f(&c["block"]),
            f(&c["channels"]) as usize,
            f(&c["ch"]) as usize,
            f(&c["from"]),
            f(&c["to"]),
        );
        assert!(
            close(lo, f(&c["want"][0])) && close(hi, f(&c["want"][1])),
            "{c}"
        );
    }
}
