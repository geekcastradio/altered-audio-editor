// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust color rules must agree with `shared-fixtures/colors.json`, the same
//! file the TypeScript tests read. Hex strings and palette bytes must match
//! exactly; blended colors are compared to 1e-9.

use altered_audio_editor_lib::colors::*;
use serde_json::Value;

fn fixture() -> Value {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../shared-fixtures/colors.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn rgb(v: &Value) -> Rgb {
    let a = v.as_array().unwrap();
    [
        a[0].as_f64().unwrap(),
        a[1].as_f64().unwrap(),
        a[2].as_f64().unwrap(),
    ]
}

fn assert_close(got: [f64; 3], want: Rgb, what: &str) {
    for i in 0..3 {
        assert!(
            (got[i] - want[i]).abs() < 1e-9,
            "{what}: {got:?} vs {want:?}"
        );
    }
}

#[test]
fn hex_to_rgb_matches() {
    for c in fixture()["hexToRgb"].as_array().unwrap() {
        let got = hex_to_rgb(c["in"].as_str().unwrap());
        let want = (!c["want"].is_null()).then(|| rgb(&c["want"]));
        assert_eq!(got, want, "{}", c["in"]);
    }
}

#[test]
fn rgb_to_hex_matches() {
    for c in fixture()["rgbToHex"].as_array().unwrap() {
        assert_eq!(
            rgb_to_hex(rgb(&c["rgb"])),
            c["want"].as_str().unwrap(),
            "{}",
            c["rgb"]
        );
    }
}

#[test]
fn rgb_to_hsl_matches() {
    for c in fixture()["rgbToHsl"].as_array().unwrap() {
        assert_close(
            rgb_to_hsl(rgb(&c["rgb"])),
            rgb(&c["want"]),
            &c["rgb"].to_string(),
        );
    }
}

#[test]
fn hsl_to_rgb_matches() {
    for c in fixture()["hslToRgb"].as_array().unwrap() {
        let got = hsl_to_rgb(
            c["h"].as_f64().unwrap(),
            c["s"].as_f64().unwrap(),
            c["l"].as_f64().unwrap(),
        );
        assert_close(got, rgb(&c["want"]), &c.to_string());
    }
}

#[test]
fn ramp_matches() {
    for c in fixture()["ramp"].as_array().unwrap() {
        let low = hex_to_rgb(c["low"].as_str().unwrap()).unwrap();
        let high = hex_to_rgb(c["high"].as_str().unwrap()).unwrap();
        let got = ramp(low, high, c["t"].as_f64().unwrap());
        assert_close(got, rgb(&c["want"]), &c.to_string());
    }
}

#[test]
fn spectrum_lut_matches() {
    for c in fixture()["spectrumLut"].as_array().unwrap() {
        let want: Vec<u8> = c["want"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();
        let got = spectrum_lut(c["low"].as_str().unwrap(), c["high"].as_str().unwrap());
        assert_eq!(got.len(), 768);
        assert_eq!(got, want, "{} to {}", c["low"], c["high"]);
    }
}
