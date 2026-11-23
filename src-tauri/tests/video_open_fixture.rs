// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Checks the video helpers and the open-file rules in Rust against
//! `shared-fixtures/video-open.json`, which the TypeScript test reads too.

use altered_audio_editor_lib::open_rules::*;
use altered_audio_editor_lib::video_view::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let p =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/video-open.json");
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

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn opt(v: &Value) -> Option<f64> {
    v.as_f64()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn video() {
    let fx = fixture();
    let v = &fx["video"];
    for c in v["describe"].as_array().unwrap() {
        let m = &c["meta"];
        let stats = (!c["st"].is_null()).then(|| {
            let s = &c["st"];
            (
                f(&s["frames"]),
                f(&s["slow"]),
                f(&s["average_ms"]),
                f(&s["max_ms"]),
                s["hardware"].as_bool().unwrap(),
            )
        });
        let got = describe_video(
            f(&m["width"]),
            f(&m["height"]),
            f(&m["fps"]),
            m["codec"].as_str(),
            stats,
        );
        ok(&json!(got), &c["out"], &format!("describe {c}"));
    }
    for c in v["pics"].as_array().unwrap() {
        let bytes: Vec<u8> = c["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b.as_u64().unwrap() as u8)
            .collect();
        let got = decode_picture(&bytes).map_or(
            Value::Null,
            |p| json!({ "width": p.width, "height": p.height, "time": p.time, "rgba": p.rgba }),
        );
        ok(&got, &c["out"], "picture");
    }
    for c in v["time"].as_array().unwrap() {
        let s = c["s"].as_f64().unwrap_or(f64::INFINITY);
        ok(
            &json!(video_time(s, f(&c["o"]), f(&c["d"]))),
            &c["out"],
            &format!("time {c}"),
        );
    }
    for c in v["tc"].as_array().unwrap() {
        ok(
            &json!(timecode(f(&c["s"]), f(&c["f"]))),
            &c["out"],
            &format!("timecode {c}"),
        );
    }
    for c in v["frame"].as_array().unwrap() {
        ok(
            &json!(needs_frame(opt(&c["a"]), opt(&c["b"]), f(&c["fps"]))),
            &c["out"],
            &format!("frame {c}"),
        );
    }
    for c in v["width"].as_array().unwrap() {
        ok(
            &json!(ask_width(f(&c["p"]), f(&c["r"]), f(&c["w"]))),
            &c["out"],
            &format!("width {c}"),
        );
    }
    for c in v["err"].as_array().unwrap() {
        ok(
            &json!(error_text(&c["e"], c["fb"].as_str().unwrap())),
            &c["out"],
            &format!("error {c}"),
        );
    }
}

#[test]
fn open_files_and_summaries() {
    let fx = fixture();
    let o = &fx["open"];
    for c in o["plans"].as_array().unwrap() {
        ok(
            &plan_open(&strings(&c["paths"])),
            &c["out"],
            &format!("plan {c}"),
        );
    }
    for c in o["kinds"].as_array().unwrap() {
        let p = c["p"].as_str().unwrap();
        assert_eq!(is_audio_file(p), c["audio"].as_bool().unwrap(), "audio {p}");
        assert_eq!(is_video_file(p), c["video"].as_bool().unwrap(), "video {p}");
        assert_eq!(
            is_session_file(p),
            c["session"].as_bool().unwrap(),
            "session {p}"
        );
    }
    for c in o["names"].as_array().unwrap() {
        let got = display_name(c["p"].as_str(), c["d"].as_str());
        ok(&json!(got), &c["out"], &format!("name {c}"));
    }
    for c in o["sesx"].as_array().unwrap() {
        ok(
            &json!(sesx_summary(&c["r"])),
            &c["out"],
            &format!("sesx {c}"),
        );
    }
    for c in o["rpp"].as_array().unwrap() {
        ok(&json!(rpp_summary(&c["r"])), &c["out"], &format!("rpp {c}"));
    }
    for c in o["otio"].as_array().unwrap() {
        ok(
            &json!(otio_summary(&c["r"])),
            &c["out"],
            &format!("otio {c}"),
        );
    }
}

#[test]
fn screen_rules() {
    let fx = fixture();
    let o = &fx["open"];
    for c in o["drop"].as_array().unwrap() {
        ok(
            &json!(drop_scale(opt(&c["p"]), f(&c["c"]), f(&c["d"]))),
            &c["out"],
            &format!("drop {c}"),
        );
    }
    for c in o["wheel"].as_array().unwrap() {
        let got = wheel_anchor_x(
            c["a"].as_str().unwrap(),
            f(&c["m"]),
            f(&c["w"]),
            f(&c["p"]),
            c["ex"].as_bool().unwrap(),
        );
        ok(&json!(got), &c["out"], &format!("wheel {c}"));
    }
    for c in o["zoom"].as_array().unwrap() {
        let (zin, zout) = zoom_ratios(f(&c["step"]));
        ok(
            &json!({ "zoomIn": zin, "zoomOut": zout }),
            &c["out"],
            &format!("zoom {c}"),
        );
    }
    for c in o["colors"].as_array().unwrap() {
        let s = |k: &str| c[k].as_str().unwrap().to_string();
        ok(
            &color_vars(&s("wave"), &s("playhead"), &s("accent"), &s("laneSelected")),
            &c["out"],
            &format!("colors {c}"),
        );
    }
    for c in o["meter"].as_array().unwrap() {
        ok(
            &json!(meter_percent(f(&c["db"]), f(&c["floor"]))),
            &c["out"],
            &format!("meter {c}"),
        );
    }
}
