// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Color ramps for the analysis views, in Rust.
//!
//! These mirror `src/app/colors.ts`. Both are tested against
//! `shared-fixtures/colors.json`, so the two copies cannot drift. When the
//! interface moves to Rust, the TypeScript copy goes away.
//!
//! A ramp blends two colors around the color wheel, so a red-to-magenta pair
//! passes through every hue in between.

use crate::js_compat::js_round;

pub type Rgb = [f64; 3];

/// Defaults: the Phase Wheel runs red (low notes) to magenta (high).
pub const PHASE_LOW: &str = "#ff2a2a";
pub const PHASE_HIGH: &str = "#e02aff";
/// The Default spectrogram: deep purple (quiet) through red and orange to yellow (loud).
pub const SPECTRUM_LOW: &str = "#4a1470";
pub const SPECTRUM_HIGH: &str = "#ffd23f";

/// "#rrggbb" or "rrggbb", in either case. None for anything else.
pub fn hex_to_rgb(hex: &str) -> Option<Rgb> {
    let t = hex.trim();
    let digits = t.strip_prefix('#').unwrap_or(t);
    if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(digits, 16).ok()?;
    Some([
        f64::from((n >> 16) & 255),
        f64::from((n >> 8) & 255),
        f64::from(n & 255),
    ])
}

fn byte(v: f64) -> f64 {
    js_round(v.clamp(0.0, 255.0))
}

pub fn rgb_to_hex(rgb: Rgb) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        byte(rgb[0]) as u8,
        byte(rgb[1]) as u8,
        byte(rgb[2]) as u8
    )
}

/// Hue 0 to 360, saturation and lightness 0 to 1.
pub fn rgb_to_hsl([r, g, b]: Rgb) -> [f64; 3] {
    let (r, g, b) = (r / 255.0, g / 255.0, b / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if max == min {
        return [0.0, 0.0, l];
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    [h * 60.0, s, l]
}

pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Rgb {
    let hue = (((h % 360.0) + 360.0) % 360.0) / 360.0;
    if s == 0.0 {
        return [l * 255.0, l * 255.0, l * 255.0];
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let f = |t: f64| {
        let u = ((t % 1.0) + 1.0) % 1.0;
        if u < 1.0 / 6.0 {
            p + (q - p) * 6.0 * u
        } else if u < 1.0 / 2.0 {
            q
        } else if u < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - u) * 6.0
        } else {
            p
        }
    };
    [
        f(hue + 1.0 / 3.0) * 255.0,
        f(hue) * 255.0,
        f(hue - 1.0 / 3.0) * 255.0,
    ]
}

/// The color `t` (0 to 1) of the way from `low` to `high`, going up the color
/// wheel from low's hue to high's, with saturation and lightness blended.
pub fn ramp(low: Rgb, high: Rgb, t: f64) -> Rgb {
    let [h0, s0, l0] = rgb_to_hsl(low);
    let [h1_raw, s1, l1] = rgb_to_hsl(high);
    let h1 = if h1_raw < h0 { h1_raw + 360.0 } else { h1_raw };
    let u = t.clamp(0.0, 1.0);
    hsl_to_rgb(h0 + (h1 - h0) * u, s0 + (s1 - s0) * u, l0 + (l1 - l0) * u)
}

/// A 256-entry spectrogram palette (RGB triples, flattened): near black for
/// silence, fading into `low` for quiet sound and up the color wheel to `high`
/// for the loudest. A color that does not read falls back to the default.
pub fn spectrum_lut(low_hex: &str, high_hex: &str) -> Vec<u8> {
    let low = hex_to_rgb(low_hex)
        .or_else(|| hex_to_rgb(SPECTRUM_LOW))
        .unwrap_or([0.0; 3]);
    let high = hex_to_rgb(high_hex)
        .or_else(|| hex_to_rgb(SPECTRUM_HIGH))
        .unwrap_or([0.0; 3]);
    let black: Rgb = [8.0, 8.0, 13.0];
    let mut out = Vec::with_capacity(256 * 3);
    for i in 0..256 {
        let t = f64::from(i) / 255.0;
        let c: Rgb = if t < 0.25 {
            let k = t / 0.25;
            [
                black[0] + (low[0] - black[0]) * k,
                black[1] + (low[1] - black[1]) * k,
                black[2] + (low[2] - black[2]) * k,
            ]
        } else {
            ramp(low, high, (t - 0.25) / 0.75)
        };
        out.extend(c.iter().map(|&v| byte(v) as u8));
    }
    out
}
