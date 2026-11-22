// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Metadata panel's rules: saved tag sets, the year and track checks, the
//! file list filter and the path list. The Rust copy of the pure parts of
//! `src/panels/metadata/metadata.ts`, tested against
//! `shared-fixtures/panel-lists.json`.
//!
//! Sets sort by name the way the browser's default locale order does for names
//! made of letters, digits and spaces: capitals ignored first, a lowercase
//! letter before its capital when nothing else differs, numbers digit by digit
//! ("Show 10" before "Show 2"). Names with punctuation or accents can differ.

use crate::js_compat::{is_js_space, js_slice_units, js_trim};
use crate::tags::TAG_FIELDS;
use serde_json::{Map, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;

pub const MAX_SETS: usize = 100;
const MAX_SET_NAME: usize = 80;

/// A named group of tag values to put on many files.
#[derive(Debug, Clone, PartialEq)]
pub struct TagSet {
    pub name: String,
    pub tags: BTreeMap<String, String>,
}

fn rank(c: char) -> (u8, char) {
    if c == ' ' {
        (0, c)
    } else if c.is_ascii_digit() {
        (1, c)
    } else {
        (2, c.to_ascii_lowercase())
    }
}

/// The browser's default name order for letters, digits and spaces.
pub fn locale_cmp(a: &str, b: &str) -> Ordering {
    let primary = a.chars().map(rank).cmp(b.chars().map(rank));
    if primary != Ordering::Equal {
        return primary;
    }
    // Same letters: a lowercase one comes before its capital.
    for (x, y) in a.chars().zip(b.chars()) {
        if x != y {
            return if x.is_ascii_lowercase() {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
    }
    Ordering::Equal
}

fn clean_tags(get: impl Fn(&str) -> Option<String>) -> BTreeMap<String, String> {
    let mut tags = BTreeMap::new();
    for k in TAG_FIELDS {
        if let Some(v) = get(k) {
            let t = js_trim(&v);
            if !t.is_empty() {
                tags.insert(k.to_string(), t.to_string());
            }
        }
    }
    tags
}

/// Saved sets read back from preferences, keeping only well-formed ones.
pub fn sanitize_sets(raw: &Value) -> Vec<TagSet> {
    let Some(list) = raw.as_array() else {
        return Vec::new();
    };
    let mut out: Vec<TagSet> = Vec::new();
    for r in list {
        if out.len() >= MAX_SETS {
            break;
        }
        if !r.is_object() {
            continue;
        }
        let name = r
            .get("name")
            .and_then(Value::as_str)
            .map(|n| js_slice_units(js_trim(n), MAX_SET_NAME))
            .unwrap_or_default();
        let lower = name.to_lowercase();
        if name.is_empty() || out.iter().any(|s| s.name.to_lowercase() == lower) {
            continue;
        }
        let tags = match r.get("tags").and_then(Value::as_object) {
            Some(t) => clean_tags(|k| t.get(k).and_then(Value::as_str).map(str::to_string)),
            None => BTreeMap::new(),
        };
        if !tags.is_empty() {
            out.push(TagSet { name, tags });
        }
    }
    out
}

/// Adds a set, or replaces the one with the same name (ignoring case). Blank
/// fields are left out.
pub fn upsert_set(
    sets: &[TagSet],
    name: &str,
    values: &BTreeMap<String, Option<String>>,
) -> Vec<TagSet> {
    let clean = js_slice_units(js_trim(name), MAX_SET_NAME);
    let tags = clean_tags(|k| values.get(k).cloned().flatten());
    if clean.is_empty() || tags.is_empty() {
        return sets.to_vec();
    }
    let lower = clean.to_lowercase();
    let mut next: Vec<TagSet> = sets
        .iter()
        .filter(|s| s.name.to_lowercase() != lower)
        .cloned()
        .collect();
    next.push(TagSet { name: clean, tags });
    next.sort_by(|a, b| locale_cmp(&a.name, &b.name));
    next.truncate(MAX_SETS);
    next
}

pub fn remove_set(sets: &[TagSet], name: &str) -> Vec<TagSet> {
    sets.iter().filter(|s| s.name != name).cloned().collect()
}

fn all_ascii_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// The year check: four digits from 1000 to 2999, or blank. A message when wrong.
pub fn check_year(v: &str) -> Option<&'static str> {
    let t = js_trim(v);
    if t.is_empty() {
        return None;
    }
    let ok = t.len() == 4 && all_ascii_digits(t) && matches!(t.as_bytes()[0], b'1' | b'2');
    if ok {
        None
    } else {
        Some("Year should be four digits, such as 2026.")
    }
}

/// The track check: a number, or a number and a total such as 3/12, or blank.
pub fn check_track(v: &str) -> Option<&'static str> {
    let t = js_trim(v);
    if t.is_empty() {
        return None;
    }
    let part = |s: &str| all_ascii_digits(s) && s.len() <= 3;
    let ok = match t.split_once('/') {
        Some((a, b)) => part(a) && part(b),
        None => part(t),
    };
    if ok {
        None
    } else {
        Some("Track should be a number, or a number and a total such as 3/12.")
    }
}

/// Every problem with the draft, in field order.
pub fn problems(year: &str, track: &str) -> Vec<&'static str> {
    [check_year(year), check_track(track)]
        .into_iter()
        .flatten()
        .collect()
}

/// The part of a path after the last slash or backslash.
pub fn base_name(p: &str) -> &str {
    p.rsplit(['\\', '/']).next().unwrap_or(p)
}

/// One file in the panel's list: its path and the tags the filter reads.
#[derive(Debug, Clone)]
pub struct Entry {
    pub path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
}

/// The entries whose name or tags contain every word typed (ignoring case).
pub fn filter_entries<'a>(entries: &'a [Entry], query: &str) -> Vec<&'a Entry> {
    let lowered = query.to_lowercase();
    let words: Vec<&str> = lowered
        .split(is_js_space)
        .filter(|w| !w.is_empty())
        .collect();
    entries
        .iter()
        .filter(|e| {
            if words.is_empty() {
                return true;
            }
            let hay = [
                Some(base_name(&e.path)),
                e.title.as_deref(),
                e.artist.as_deref(),
                e.album.as_deref(),
                e.genre.as_deref(),
            ]
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
            .to_lowercase();
            words.iter().all(|w| hay.contains(w))
        })
        .collect()
}

/// Shows a set's values over a file's tags, keeping fields the set does not have.
pub fn overlay(tags: &Map<String, Value>, s: &TagSet) -> Map<String, Value> {
    let mut out = tags.clone();
    for (k, v) in &s.tags {
        out.insert(k.clone(), Value::String(v.clone()));
    }
    out
}

/// Adds paths to the list without repeating any (compared ignoring case and
/// slash direction).
pub fn add_paths(have: &[String], more: &[String]) -> Vec<String> {
    let key = |p: &str| p.replace('\\', "/").to_lowercase();
    let mut seen: std::collections::HashSet<String> = have.iter().map(|p| key(p)).collect();
    let mut out = have.to_vec();
    for p in more {
        if seen.insert(key(p)) {
            out.push(p.clone());
        }
    }
    out
}
