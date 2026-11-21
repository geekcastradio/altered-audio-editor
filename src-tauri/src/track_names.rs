// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Track names. The Rust copy of `src/app/track-names.ts`; both are tested
//! against `shared-fixtures/track-names.json`.

use crate::js_compat::{is_js_space, js_trim};
use std::collections::HashSet;

fn is_line_end(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// The file name after the last `/` or `\`. Like the TypeScript `^.*[\\/]`,
/// nothing is cut when a line break sits before that separator.
fn base_name(path: &str) -> &str {
    match path.rfind(['/', '\\']) {
        Some(i) if !path[..i].chars().any(is_line_end) => &path[i + 1..],
        _ => path,
    }
}

/// Drops a leading recorder number: one to three digits, then dashes,
/// underscores or spaces, as long as a visible character follows.
fn strip_number(s: &str) -> &str {
    let digits = s.bytes().take_while(u8::is_ascii_digit).count();
    if !(1..=3).contains(&digits) {
        return s;
    }
    let rest = &s[digits..];
    let run = rest
        .bytes()
        .take_while(|b| matches!(b, b'-' | b'_' | b' '))
        .count();
    // The regex is greedy and backs off one character at a time until the
    // character after the run is not white space.
    for k in (1..=run).rev() {
        if let Some(next) = rest[k..].chars().next() {
            if !is_js_space(next) {
                return &rest[k..];
            }
        }
    }
    s
}

/// A track name from a file name: the extension goes, and so does the number
/// call recorders put in front of each speaker. Underscores read as spaces.
pub fn track_name_from_file(file_name: &str) -> String {
    let base = base_name(file_name);
    let no_ext = match base.rfind('.') {
        Some(i) if i + 1 < base.len() => &base[..i],
        _ => base,
    };
    // A run of underscores becomes one space, as `_+` did in the TypeScript.
    let mut spaced = String::new();
    let mut in_run = false;
    for c in strip_number(no_ext).chars() {
        if c == '_' {
            if !in_run {
                spaced.push(' ');
            }
            in_run = true;
        } else {
            spaced.push(c);
            in_run = false;
        }
    }
    let s = js_trim(&spaced).to_string();
    if s.is_empty() {
        base.to_string()
    } else {
        s
    }
}

/// Names for `count` new tracks that do not clash with `existing`. With no
/// base they continue "Track N"; with a base the first is the base itself when
/// free, then "Guest 2", "Guest 3" and so on.
pub fn new_track_names(existing: &[String], count: usize, base: &str) -> Vec<String> {
    let mut taken: HashSet<String> = existing.iter().cloned().collect();
    let mut out = Vec::new();
    let b = js_trim(base);
    let mut n = if b.is_empty() { existing.len() + 1 } else { 1 };
    while out.len() < count {
        let name = if b.is_empty() {
            format!("Track {n}")
        } else if n == 1 {
            b.to_string()
        } else {
            format!("{b} {n}")
        };
        if taken.insert(name.clone()) {
            out.push(name);
        }
        n += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_backs_off_before_white_space() {
        assert_eq!(track_name_from_file("1- \t.wav"), "1-");
        assert_eq!(track_name_from_file("1-x.wav"), "x");
        assert_eq!(track_name_from_file("1234-x.wav"), "1234-x");
    }
}
