// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Typed times and selection edits, in Rust.
//!
//! These mirror `parseTime` and `editSelection` in `src/panels/time-info/time-info.ts`.
//! Both are tested against `shared-fixtures/time-input.json`, so the two copies
//! cannot drift. The bars format needs the session tempo and stays in TypeScript
//! until the tempo moves to the core. When the interface moves to Rust, the
//! TypeScript copy goes away. All positions are in samples.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeFormat {
    Time,
    Samples,
    Timecode,
    Seconds,
}

fn all_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// One to `max` ASCII digits.
fn short_digits(s: &str, max: usize) -> bool {
    all_digits(s) && s.len() <= max
}

/// "h:mm:ss", "m:ss" or "s", the last part with an optional fraction
/// (`^\d+(:\d{1,2}){0,2}(\.\d*)?$`). Seconds, or None when it does not fit.
fn parse_clock(t: &str) -> Option<f64> {
    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() > 3 {
        return None;
    }
    let last = parts.len() - 1;
    let mut secs = 0.0;
    for (i, part) in parts.iter().enumerate() {
        // Only the last part may hold the fraction.
        let (whole, frac) = if i == last {
            part.split_once('.').unwrap_or((part, ""))
        } else {
            (*part, "")
        };
        let whole_ok = if i == 0 {
            all_digits(whole)
        } else {
            short_digits(whole, 2)
        };
        if !whole_ok || !frac.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        secs = secs * 60.0 + part.parse::<f64>().ok()?;
    }
    secs.is_finite().then_some(secs)
}

/// "hh:mm:ss:ff" or "hh:mm:ss;ff" at `fps`.
fn parse_timecode(t: &str, rate: f64, fps: f64) -> Option<f64> {
    let parts: Vec<&str> = t.split([':', ';']).collect();
    if parts.len() != 4 {
        return None;
    }
    // Only the last separator may be a semicolon.
    let first_three: Vec<&str> = t.rsplitn(2, [':', ';']).collect();
    if first_three.len() != 2 || first_three[1].contains(';') {
        return None;
    }
    if !all_digits(parts[0]) || !parts[1..].iter().all(|p| short_digits(p, 2)) {
        return None;
    }
    let n = |i: usize| parts[i].parse::<f64>().ok();
    let secs = (n(0)? * 60.0 + n(1)?) * 60.0 + n(2)? + n(3)? / fps;
    Some((secs * rate).round())
}

/// A typed time in samples. Reads "1:05.250", "65.25", "0:01:05" and,
/// in the samples format, a plain count; in timecode, "00:01:05:12" at
/// `fps` too. None when it is not a time. The bars format is not handled here.
pub fn parse_time(text: &str, rate: f64, format: TimeFormat, fps: f64) -> Option<f64> {
    let t = text.trim();
    if format == TimeFormat::Samples && all_digits(t) {
        return t.parse::<f64>().ok();
    }
    if format == TimeFormat::Timecode {
        if let Some(v) = parse_timecode(t, rate, fps) {
            return Some(v);
        }
    }
    let secs = parse_clock(t)?;
    Some((secs * rate).round())
}

/// A selection with one edge or the length changed, kept in order and at or after 0.
pub fn edit_selection(sel: (f64, f64), part: &str, value: f64) -> (f64, f64) {
    let v = value.round().max(0.0);
    if part == "duration" {
        return (sel.0, sel.0 + v);
    }
    let a = if part == "start" { v } else { sel.0 };
    let b = if part == "end" { v } else { sel.1 };
    (a.min(b), a.max(b))
}
