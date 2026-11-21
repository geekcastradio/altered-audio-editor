// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Dial maths, in Rust.
//!
//! These mirror `src/app/knob.ts`. Both are tested against
//! `shared-fixtures/knob.json`, so the two copies cannot drift. When the
//! interface moves to Rust, the TypeScript copy goes away.

/// Pixels of vertical drag for the dial's whole range.
pub const DRAG_PIXELS: f64 = 160.0;

fn clamp01(t: f64) -> f64 {
    t.clamp(0.0, 1.0)
}

/// The dial's value after dragging `dy` pixels up (negative is down) from
/// `start`. Ctrl moves in fine steps, Shift in coarse ones.
pub fn drag_value(start: f64, dy: f64, ctrl: bool, shift: bool) -> f64 {
    let scale = if ctrl {
        0.2
    } else if shift {
        3.0
    } else {
        1.0
    };
    clamp01(start + (dy / DRAG_PIXELS) * scale)
}

/// The dial's value after a key press, or None when the key does nothing.
pub fn key_value(value: f64, key: &str, home: f64) -> Option<f64> {
    match key {
        "ArrowUp" | "ArrowRight" => Some(clamp01(value + 0.01)),
        "ArrowDown" | "ArrowLeft" => Some(clamp01(value - 0.01)),
        "PageUp" => Some(clamp01(value + 0.1)),
        "PageDown" => Some(clamp01(value - 0.1)),
        "Home" => Some(home),
        _ => None,
    }
}

/// Where the dial's pointer points, in degrees from straight up.
pub fn pointer_angle(value: f64) -> f64 {
    -135.0 + clamp01(value) * 270.0
}
