// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Update notice rules, in Rust.
//!
//! These mirror `src/app/update.ts` (when the optional check is due, release
//! notes as plain text, and the first few changes for the update card) and are
//! tested against `shared-fixtures/playhead-update.json`, which the TypeScript
//! test reads too. The text rules copy what the JavaScript patterns do,
//! including which characters count as white space and how lines end, without
//! a regular expression library.

use crate::js_compat::{is_js_space, js_trim};

pub const WEEK_MS: f64 = 7.0 * 24.0 * 60.0 * 60.0 * 1000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateEvery {
    Launch,
    Week,
}

/// The check runs only when turned on. Every start: always. Weekly: when a
/// week has passed since the last one. A clock set back never blocks it.
pub fn update_due(enabled: bool, last_check: f64, now: f64, every: UpdateEvery) -> bool {
    if !enabled {
        return false;
    }
    if every == UpdateEvery::Launch {
        return true;
    }
    if !last_check.is_finite() || last_check <= 0.0 {
        return true;
    }
    now < last_check || now - last_check >= WEEK_MS
}

/// Where a line starts, as JavaScript's multiline `^` sees it.
fn line_start(c: &[char], i: usize) -> bool {
    i == 0 || matches!(c[i - 1], '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

fn skip_space(c: &[char], mut i: usize) -> usize {
    while i < c.len() && is_js_space(c[i]) {
        i += 1;
    }
    i
}

/// `/^#{1,6}\s*/gm` replaced with nothing.
fn drop_heading_marks(c: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(c.len());
    let mut i = 0;
    while i < c.len() {
        if line_start(c, i) && c[i] == '#' {
            let mut j = i;
            while j < c.len() && c[j] == '#' && j - i < 6 {
                j += 1;
            }
            i = skip_space(c, j);
        } else {
            out.push(c[i]);
            i += 1;
        }
    }
    out
}

/// Replaces `open + [^close]+ + close` runs with their inside. Used for
/// `**bold**` (open and close are two stars) and `` `code` `` (one backtick).
fn unwrap_pairs(c: &[char], mark: char, width: usize) -> Vec<char> {
    let mut out = Vec::with_capacity(c.len());
    let mut i = 0;
    let n = c.len();
    while i < n {
        if i + width <= n && c[i..i + width].iter().all(|&x| x == mark) {
            let start = i + width;
            let mut k = start;
            while k < n && c[k] != mark {
                k += 1;
            }
            if k > start && k + width <= n && c[k..k + width].iter().all(|&x| x == mark) {
                out.extend_from_slice(&c[start..k]);
                i = k + width;
                continue;
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// `/\[([^\]]+)\]\([^)]+\)/g` replaced with the text in the brackets.
fn drop_links(c: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(c.len());
    let mut i = 0;
    let n = c.len();
    while i < n {
        if c[i] == '[' {
            let mut k = i + 1;
            while k < n && c[k] != ']' {
                k += 1;
            }
            if k > i + 1 && k + 1 < n && c[k] == ']' && c[k + 1] == '(' {
                let mut m = k + 2;
                while m < n && c[m] != ')' {
                    m += 1;
                }
                if m > k + 2 && m < n {
                    out.extend_from_slice(&c[i + 1..k]);
                    i = m + 1;
                    continue;
                }
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// `/^\s*[-*]\s+/gm` replaced with a bullet dot and a space.
fn bullets(c: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(c.len());
    let mut i = 0;
    while i < c.len() {
        if line_start(c, i) {
            let j = skip_space(c, i);
            if j < c.len() && (c[j] == '-' || c[j] == '*') {
                let k = skip_space(c, j + 1);
                if k > j + 1 {
                    out.push('\u{2022}');
                    out.push(' ');
                    i = k;
                    continue;
                }
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// `/\n{3,}/g` replaced with two line ends.
fn squeeze_blank_lines(c: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(c.len());
    let mut i = 0;
    while i < c.len() {
        if c[i] == '\n' {
            let mut j = i;
            while j < c.len() && c[j] == '\n' {
                j += 1;
            }
            let run = j - i;
            out.extend(std::iter::repeat_n('\n', if run >= 3 { 2 } else { run }));
            i = j;
        } else {
            out.push(c[i]);
            i += 1;
        }
    }
    out
}

/// Release notes written in Markdown, as plain text: no #, ** or backticks,
/// links as their words, bullets as dots.
pub fn plain_notes(md: &str) -> String {
    let c: Vec<char> = md.replace("\r\n", "\n").chars().collect();
    let c = drop_heading_marks(&c);
    let c = unwrap_pairs(&c, '*', 2);
    let c = unwrap_pairs(&c, '`', 1);
    let c = drop_links(&c);
    let c = bullets(&c);
    let c = squeeze_blank_lines(&c);
    let s: String = c.into_iter().collect();
    js_trim(&s).to_string()
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// The first few changes from the release notes, one line each. Bullets win;
/// with none, the first plain lines are used. Long lines are cut at a word.
pub fn highlights(md: Option<&str>, max: usize, width: usize) -> Vec<String> {
    let Some(md) = md.filter(|m| !m.is_empty()) else {
        return Vec::new();
    };
    let plain = plain_notes(md);
    let lines: Vec<&str> = plain
        .split('\n')
        .map(js_trim)
        .filter(|l| !l.is_empty())
        .collect();
    let bullet_lines: Vec<&str> = lines
        .iter()
        .filter(|l| l.starts_with('\u{2022}'))
        .map(|l| {
            l.trim_start_matches('\u{2022}')
                .trim_start_matches(is_js_space)
        })
        .collect();
    let pool = if bullet_lines.is_empty() {
        lines
    } else {
        bullet_lines
    };
    pool.into_iter()
        .take(max)
        .map(|l| {
            if utf16_len(l) <= width {
                return l.to_string();
            }
            let cut = crate::js_compat::js_slice_units(l, width.saturating_sub(1));
            let mut units = 0;
            let mut sp: Option<usize> = None;
            for ch in cut.chars() {
                if ch == ' ' {
                    sp = Some(units);
                }
                units += ch.len_utf16();
            }
            let head = match sp {
                Some(at) if (at as f64) > width as f64 / 2.0 => {
                    crate::js_compat::js_slice_units(&cut, at)
                }
                _ => cut,
            };
            format!("{}\u{2026}", head.trim_end_matches(is_js_space))
        })
        .collect()
}
