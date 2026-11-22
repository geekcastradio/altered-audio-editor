// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! 3D Space sessions in the interface, in Rust: each layout's speakers and
//! where they stand (x left -1 to right 1, y back -1 to front 1, z floor 0 to
//! ceiling 1), and the room view's sums. Mirrors `src/panels/space/space.ts`;
//! tested against `shared-fixtures/wave-space.json`.

use crate::js_compat::js_round;
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct SpeakerSpot {
    pub name: &'static str,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

const fn s(name: &'static str, x: f64, y: f64, z: f64) -> SpeakerSpot {
    SpeakerSpot { name, x, y, z }
}

const FRONT: [SpeakerSpot; 3] = [
    s("L", -1.0, 1.0, 0.0),
    s("C", 0.0, 1.0, 0.0),
    s("R", 1.0, 1.0, 0.0),
];
const REAR5: [SpeakerSpot; 2] = [s("Ls", -1.0, -1.0, 0.0), s("Rs", 1.0, -1.0, 0.0)];
const SIDE7: [SpeakerSpot; 4] = [
    s("Lss", -1.0, 0.0, 0.0),
    s("Rss", 1.0, 0.0, 0.0),
    s("Lrs", -1.0, -1.0, 0.0),
    s("Rrs", 1.0, -1.0, 0.0),
];
const TOP2: [SpeakerSpot; 2] = [s("Ltm", -1.0, 0.0, 1.0), s("Rtm", 1.0, 0.0, 1.0)];
const TOP4: [SpeakerSpot; 4] = [
    s("Ltf", -1.0, 1.0, 1.0),
    s("Rtf", 1.0, 1.0, 1.0),
    s("Ltr", -1.0, -1.0, 1.0),
    s("Rtr", 1.0, -1.0, 1.0),
];

/// The layouts, in the order menus list them.
pub const LAYOUTS: [&str; 7] = ["2.0", "5.1", "7.1", "5.1.2", "7.1.2", "5.1.4", "7.1.4"];

/// The placed speakers of a layout (its low-frequency channel has no place).
pub fn speaker_spots(layout: &str) -> Vec<SpeakerSpot> {
    let cat = |parts: &[&[SpeakerSpot]]| parts.iter().flat_map(|p| p.iter().cloned()).collect();
    match layout {
        "2.0" => vec![s("L", -1.0, 1.0, 0.0), s("R", 1.0, 1.0, 0.0)],
        "5.1" => cat(&[&FRONT, &REAR5]),
        "7.1" => cat(&[&FRONT, &SIDE7]),
        "5.1.2" => cat(&[&FRONT, &REAR5, &TOP2]),
        "7.1.2" => cat(&[&FRONT, &SIDE7, &TOP2]),
        "5.1.4" => cat(&[&FRONT, &REAR5, &TOP4]),
        "7.1.4" => cat(&[&FRONT, &SIDE7, &TOP4]),
        _ => vec![],
    }
}

/// What each layout is, for menus.
pub fn layout_name(layout: &str) -> Option<&'static str> {
    Some(match layout {
        "2.0" => "2.0 Stereo",
        "5.1" => "5.1",
        "7.1" => "7.1",
        "5.1.2" => "5.1.2 (two height speakers)",
        "7.1.2" => "7.1.2 (two height speakers)",
        "5.1.4" => "5.1.4 (four height speakers)",
        "7.1.4" => "7.1.4 (four height speakers)",
        _ => return None,
    })
}

/// `Math.min(hi, Math.max(lo, v))`: NaN stays NaN.
fn keep(v: f64, lo: f64, hi: f64) -> f64 {
    if v.is_nan() {
        v
    } else {
        v.clamp(lo, hi)
    }
}

/// A spot with every value in range (missing or odd parts take the default).
pub fn tidy_pos(p: Option<&Value>) -> Value {
    let c = |key: &str, lo: f64, hi: f64, def: f64| -> f64 {
        match p.and_then(|p| p.get(key)).and_then(Value::as_f64) {
            Some(v) if v.is_finite() => keep(v, lo, hi),
            _ => def,
        }
    };
    json!({
        "x": c("x", -1.0, 1.0, 0.0),
        "y": c("y", -1.0, 1.0, 1.0),
        // Below 0 is below ear level.
        "z": c("z", -1.0, 1.0, 0.0),
        "size": c("size", 0.0, 1.0, 0.0),
        "width": c("width", 0.0, 1.0, 1.0),
        "rotation": c("rotation", -180.0, 180.0, 0.0),
        "lfe": c("lfe", 0.0, 1.0, 0.0),
        "spread": c("spread", 0.0, 1.0, 0.0),
        "spatialize": c("spatialize", 0.0, 1.0, 0.0),
    })
}

/// Room units (-1 to 1) to pixels in a square `size` across, front at the top, with a margin.
pub fn to_view(x: f64, y: f64, size: f64) -> (f64, f64) {
    let m = size * 0.1;
    let span = size - 2.0 * m;
    (m + ((x + 1.0) / 2.0) * span, m + ((1.0 - y) / 2.0) * span)
}

/// Pixels in the room view to room units, kept inside the room.
pub fn from_view(px: f64, py: f64, size: f64) -> (f64, f64) {
    let m = size * 0.1;
    let span = size - 2.0 * m;
    let x = ((px - m) / span) * 2.0 - 1.0;
    let y = 1.0 - ((py - m) / span) * 2.0;
    (keep(x, -1.0, 1.0), keep(y, -1.0, 1.0))
}

/// Where a stereo source's two sides sit: `width` either side of its spot, the
/// pair turned by its rotation, kept in the room.
pub fn sides(x: f64, y: f64, width: f64, rotation: f64) -> [(f64, f64); 2] {
    let a = rotation * std::f64::consts::PI / 180.0;
    let dx = width * a.cos();
    let dy = -width * a.sin();
    let k = |v: f64| keep(v, -1.0, 1.0);
    [(k(x - dx), k(y - dy)), (k(x + dx), k(y + dy))]
}

/// Every channel of a layout in file order, the low-frequency one included.
pub fn channel_names(layout: &str) -> Vec<&'static str> {
    let base = ["L", "R", "C", "LFE"];
    let tail: &[&'static str] = match layout {
        "2.0" => return vec!["L", "R"],
        "5.1" => &["Ls", "Rs"],
        "7.1" => &["Lrs", "Rrs", "Lss", "Rss"],
        "5.1.2" => &["Ls", "Rs", "Ltm", "Rtm"],
        "7.1.2" => &["Lrs", "Rrs", "Lss", "Rss", "Ltm", "Rtm"],
        "5.1.4" => &["Ls", "Rs", "Ltf", "Rtf", "Ltr", "Rtr"],
        "7.1.4" => &["Lrs", "Rrs", "Lss", "Rss", "Ltf", "Rtf", "Ltr", "Rtr"],
        _ => return vec![],
    };
    base.iter().chain(tail).copied().collect()
}

/// Turns degrees into -180 to 180.
pub fn wrap_deg(d: f64) -> f64 {
    let mut v = ((d + 180.0) % 360.0 + 360.0) % 360.0 - 180.0;
    if v == -180.0 && d > 0.0 {
        v = 180.0;
    }
    v
}

/// The angle from (cx, cy) to (x, y) in the room, degrees clockwise seen from above.
pub fn angle_of(cx: f64, cy: f64, x: f64, y: f64) -> f64 {
    (-(y - cy)).atan2(x - cx) * 180.0 / std::f64::consts::PI
}

/// A spot pulled onto a speaker, the center lines or a quarter-room grid when within `tol`.
pub fn snap_spot(x: f64, y: f64, speakers: &[(f64, f64)], tol: f64) -> (f64, f64) {
    for &(sx, sy) in speakers {
        if (sx - x).hypot(sy - y) <= tol * 1.5 {
            return (sx, sy);
        }
    }
    let grid = |v: f64| {
        let g = js_round(v * 4.0) / 4.0;
        if (g - v).abs() <= tol {
            g
        } else {
            v
        }
    };
    (grid(x), grid(y))
}

/// The default distance for `snap_spot`.
pub const SNAP_TOL: f64 = 0.06;

/// A rotation snapped to 15 degree steps when within 4 degrees.
pub fn snap_angle(d: f64) -> f64 {
    let g = js_round(d / 15.0) * 15.0;
    if (g - d).abs() <= 4.0 {
        wrap_deg(g)
    } else {
        d
    }
}
