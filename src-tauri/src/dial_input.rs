// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Typed dial values, in Rust.
//!
//! These mirror `src/panels/editor/dial-input.ts`. Both are tested against
//! `shared-fixtures/dial-input.json`, so the two copies cannot drift. When the
//! interface moves to Rust, the TypeScript copy goes away.

/// A typed number such as "-2", "+3.5", "−2.0 dB" or "-6dB". None when there is no number.
pub fn parse_number(text: &str) -> Option<f64> {
    let t = text.trim().replace('\u{2212}', "-").replace(',', ".");
    let mut rest = t.as_str();
    let mut number = String::new();
    if let Some(c) = rest.chars().next().filter(|c| *c == '+' || *c == '-') {
        number.push(c);
        rest = &rest[1..];
    }
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    number.push_str(&rest[..digits]);
    rest = &rest[digits..];
    let mut fraction = 0;
    if let Some(after) = rest.strip_prefix('.') {
        fraction = after.chars().take_while(char::is_ascii_digit).count();
        if fraction == 0 {
            return None;
        }
        number.push('.');
        number.push_str(&after[..fraction]);
        rest = &after[fraction..];
    }
    if digits == 0 && fraction == 0 {
        return None;
    }
    let rest = rest.trim_start();
    if !(rest.is_empty() || rest == "%" || rest.eq_ignore_ascii_case("db")) {
        return None;
    }
    number.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// A typed pan: "C", "L30", "R20", or a plain number from -100 (left) to 100 (right).
pub fn parse_pan(text: &str) -> Option<f64> {
    let t = text.trim();
    if ["c", "center", "centre"]
        .iter()
        .any(|w| t.eq_ignore_ascii_case(w))
    {
        return Some(0.0);
    }
    let mut chars = t.chars();
    if let Some(side @ ('l' | 'L' | 'r' | 'R')) = chars.next() {
        let rest = chars.as_str().trim_start();
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if digits > 0 {
            let mut end = digits;
            if let Some(after) = rest[digits..].strip_prefix('.') {
                let fraction = after.chars().take_while(char::is_ascii_digit).count();
                if fraction > 0 {
                    end = digits + 1 + fraction;
                }
            }
            if end == rest.len() {
                let v: f64 = rest.parse().ok()?;
                return Some(if side.eq_ignore_ascii_case(&'l') {
                    -v
                } else {
                    v
                });
            }
        }
    }
    parse_number(t)
}
