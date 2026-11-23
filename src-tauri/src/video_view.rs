// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Video reference helpers in Rust: the line for the Video panel, reading the
//! frame bytes, time mapping, timecode, when to fetch a new picture, how wide
//! to ask for it, and a message for an error of any shape. Mirrors
//! `src/app/video.ts`, tested against `shared-fixtures/video-view.json`.

use crate::js_compat::js_round;
use serde_json::Value;

/// One line for the Video panel: what the file is and how decoding is going.
/// `stats` is `(frames, slow, average_ms, max_ms, hardware)`.
pub fn describe_video(
    width: f64,
    height: f64,
    fps: f64,
    codec: Option<&str>,
    stats: Option<(f64, f64, f64, f64, bool)>,
) -> String {
    let rate = if fps > 0.0 {
        format!(
            " at {} fps",
            crate::js_compat::js_number_to_string(js_round(fps * 100.0) / 100.0)
        )
    } else {
        String::new()
    };
    let codec = match codec {
        Some(c) if !c.is_empty() => format!("{c}, "),
        _ => String::new(),
    };
    let size = |n: f64| crate::js_compat::js_number_to_string(n);
    let mut parts = vec![format!("{codec}{}x{}{rate}", size(width), size(height))];
    if let Some((frames, slow, average, max, _)) = stats {
        if frames > 0.0 {
            parts.push(format!(
                "{} ms per picture (slowest {})",
                size(js_round(average)),
                size(js_round(max))
            ));
            parts.push(format!(
                "{} of {} slower than one frame",
                size(slow),
                size(frames)
            ));
        }
    }
    let hardware = stats.is_some_and(|s| s.4);
    parts.push(
        if hardware {
            "hardware decoder in use"
        } else {
            "software decoding"
        }
        .to_string(),
    );
    parts.join(" | ")
}

/// A decoded picture: its size, its time in the video, and RGBA bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoPicture {
    pub width: u32,
    pub height: u32,
    pub time: f64,
    pub rgba: Vec<u8>,
}

const HEADER: usize = 16;

/// Reads the bytes `video_frame` returns: width u32, height u32, time f64 (all
/// little endian), then RGBA. None when they do not fit together.
pub fn decode_picture(bytes: &[u8]) -> Option<VideoPicture> {
    if bytes.len() < HEADER {
        return None;
    }
    let width = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
    let height = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
    let time = f64::from_le_bytes(bytes[8..16].try_into().ok()?);
    if width == 0
        || height == 0
        || (bytes.len() - HEADER) as u64 != u64::from(width) * u64::from(height) * 4
    {
        return None;
    }
    Some(VideoPicture {
        width,
        height,
        time,
        rgba: bytes[HEADER..].to_vec(),
    })
}

/// The time in the video that plays at `session_time`, or None when the video
/// has not started yet or has ended there.
pub fn video_time(session_time: f64, offset: f64, duration: f64) -> Option<f64> {
    let t = session_time - offset;
    if !t.is_finite() || t < 0.0 {
        return None;
    }
    if duration > 0.0 && t > duration {
        return None;
    }
    Some(t)
}

/// Hours:minutes:seconds:frames, the way picture editors read time.
pub fn timecode(seconds: f64, fps: f64) -> String {
    let rate = if fps > 0.0 { fps } else { 25.0 };
    let whole = 0.0_f64.max(js_round(seconds * rate));
    let base = js_round(rate);
    let frames = whole % base;
    let total = (whole / base).floor();
    let pad = |n: f64| format!("{:02}", n as i64);
    format!(
        "{}:{}:{}:{}",
        pad((total / 3600.0).floor()),
        pad((total / 60.0).floor() % 60.0),
        pad(total % 60.0),
        pad(frames)
    )
}

/// Whether the picture should be fetched again: the playhead moved a frame or more.
pub fn needs_frame(last: Option<f64>, next: Option<f64>, fps: f64) -> bool {
    let Some(next) = next else {
        return last.is_some();
    };
    let Some(last) = last else {
        return true;
    };
    let frame = 1.0 / if fps > 0.0 { fps } else { 25.0 };
    (next - last).abs() >= frame * 0.9
}

/// Width to ask for: the panel's width in device pixels, held to a sensible range.
pub fn ask_width(panel_width: f64, ratio: f64, video_width: f64) -> f64 {
    let want = js_round(panel_width.max(1.0) * ratio.max(1.0));
    want.max(160.0).min(video_width.max(160.0)).min(1920.0)
}

/// A message for an error of any shape.
pub fn error_text(e: &Value, fallback: &str) -> String {
    if let Some(s) = e.as_str() {
        if !s.is_empty() {
            return s.to_string();
        }
    }
    if let Some(m) = e.get("message").and_then(Value::as_str) {
        if !m.is_empty() {
            return m.to_string();
        }
    }
    if matches!(e, Value::Object(_) | Value::Array(_)) {
        if let Ok(j) = serde_json::to_string(e) {
            if !j.is_empty() && j != "{}" {
                return format!("{fallback} ({j})");
            }
        }
    }
    fallback.to_string()
}
