// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust output format rules must agree with
//! `shared-fixtures/output-format.json`, the same file the TypeScript tests
//! read.

use altered_audio_editor_lib::output_format::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/output-format.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

fn src(v: &Value) -> Src {
    Src {
        sample_rate: f(&v["sampleRate"]),
        channels: f(&v["channels"]),
        bit_depth: f(&v["bitDepth"]),
    }
}

fn opt(v: &Value) -> Option<f64> {
    v.as_f64()
}

/// Checks every field of a format against its JSON form.
fn same_format(got: &OutputFormat, want: &Value) -> bool {
    got.kind.as_str() == want["kind"].as_str().unwrap()
        && got.bit_depth.map(f64::from) == opt(&want["bitDepth"])
        && got.sample_rate.map(f64::from) == opt(&want["sampleRate"])
        && got.channels.map(f64::from) == opt(&want["channels"])
        && (if got.mp3_vbr { "vbr" } else { "cbr" }) == want["mp3Mode"].as_str().unwrap()
        && f64::from(got.mp3_bitrate) == f(&want["mp3Bitrate"])
        && f64::from(got.mp3_quality) == f(&want["mp3Quality"])
        && f64::from(got.aac_bitrate) == f(&want["aacBitrate"])
        && got.dither == want["dither"].as_bool().unwrap()
        && got.dither_kind.as_str() == want["ditherKind"].as_str().unwrap()
        && close(got.dither_amount, f(&want["ditherAmount"]))
        && got.noise_shaping.as_str() == want["noiseShaping"].as_str().unwrap()
        && got.resample_quality.as_str() == want["resampleQuality"].as_str().unwrap()
        && close(got.mix_left, f(&want["mixLeft"]))
        && close(got.mix_right, f(&want["mixRight"]))
}

fn kind(v: &Value) -> Kind {
    KINDS
        .iter()
        .copied()
        .find(|k| k.as_str() == v.as_str().unwrap())
        .unwrap()
}

#[test]
fn reading_and_lookup_match() {
    let fx = fixture();
    for c in fx["sanitize"].as_array().unwrap() {
        assert!(same_format(&sanitize_format(&c["raw"]), &c["want"]), "{c}");
    }
    for c in fx["depths"].as_array().unwrap() {
        let got: Vec<Value> = depths_for(kind(&c["kind"]))
            .iter()
            .map(|(v, l)| serde_json::json!({"value": v, "label": l}))
            .collect();
        assert_eq!(Value::Array(got), c["want"], "{c}");
    }
    for c in fx["paths"].as_array().unwrap() {
        let got = kind_of_path(c["path"].as_str().unwrap()).map(|k| k.as_str());
        assert_eq!(got, c["want"].as_str(), "{c}");
    }
    for c in fx["avail"].as_array().unwrap() {
        let windows = c["platform"] == "windows";
        assert_eq!(
            format_available(kind(&c["kind"]), windows),
            c["want"].as_bool().unwrap(),
            "{c}"
        );
    }
}

#[test]
fn text_and_size_match() {
    let fx = fixture();
    for c in fx["describe"].as_array().unwrap() {
        assert_eq!(
            describe_format(&sanitize_format(&c["f"]), src(&c["src"])),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["sample"].as_array().unwrap() {
        assert_eq!(
            describe_sample_type(&sanitize_format(&c["f"]), src(&c["src"])),
            c["want"].as_str().unwrap(),
            "{c}"
        );
    }
    for c in fx["bytes"].as_array().unwrap() {
        assert!(
            close(
                estimate_bytes(&sanitize_format(&c["f"]), f(&c["seconds"]), src(&c["src"])),
                f(&c["want"])
            ),
            "{c}"
        );
    }
    for c in fx["settings"].as_array().unwrap() {
        let fmt = sanitize_format(&c["f"]);
        assert_eq!(
            describe_settings(&fmt).as_deref(),
            c["want"].as_str(),
            "{c}"
        );
        assert_eq!(dither_note(&fmt).as_deref(), c["note"].as_str(), "{c}");
    }
}

fn memory_of(v: &Value) -> Memory {
    let mut m: Memory = Default::default();
    for (i, k) in KINDS.iter().enumerate() {
        if let Some(x) = v.get(k.as_str()) {
            m[i] = Some(sanitize_format(x));
        }
    }
    m
}

fn same_memory(got: &Memory, want: &Value) -> bool {
    KINDS
        .iter()
        .enumerate()
        .all(|(i, k)| match (&got[i], want.get(k.as_str())) {
            (None, None) => true,
            (Some(g), Some(w)) => same_format(g, w),
            _ => false,
        })
}

#[test]
fn memory_matches() {
    let fx = fixture();
    for c in fx["memory"].as_array().unwrap() {
        assert!(same_memory(&sanitize_memory(&c["raw"]), &c["want"]), "{c}");
    }
    for c in fx["switch"].as_array().unwrap() {
        let (mem, format) = switch_format(
            &memory_of(&c["memory"]),
            &sanitize_format(&c["from"]),
            kind(&c["to"]),
        );
        assert!(same_memory(&mem, &c["want"]["memory"]), "{c}");
        assert!(same_format(&format, &c["want"]["format"]), "{c}");
    }
}
