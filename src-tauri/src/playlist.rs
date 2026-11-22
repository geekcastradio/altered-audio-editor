// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Playlist rules: an ordered list of range markers that previews back to back
//! and exports as one file per range. The Rust copy of the logic in
//! `src/panels/playlist/playlist.ts`; both are tested against
//! `shared-fixtures/playlist-review.json`. The functions take the markers and
//! the saved id list instead of the whole session.

use crate::js_compat::{is_js_space, js_trim};
use std::collections::{HashMap, HashSet};

/// The parts of a marker the playlist reads.
#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    pub id: String,
    pub name: String,
    pub position: f64,
    pub duration: f64,
}

/// Range markers in playlist order. Stale ids and point markers are skipped.
pub fn playlist_items<'a>(markers: &'a [Marker], playlist: &[String]) -> Vec<&'a Marker> {
    // Like the TypeScript Map, the last marker with an id wins.
    let by_id: HashMap<&str, &Marker> = markers.iter().map(|m| (m.id.as_str(), m)).collect();
    playlist
        .iter()
        .filter_map(|id| by_id.get(id.as_str()).copied())
        .filter(|m| m.duration > 0.0)
        .collect()
}

/// Range markers not in the playlist yet, in time order.
pub fn available<'a>(markers: &'a [Marker], playlist: &[String]) -> Vec<&'a Marker> {
    let in_list: HashSet<&str> = playlist.iter().map(String::as_str).collect();
    let mut out: Vec<&Marker> = markers
        .iter()
        .filter(|m| m.duration > 0.0 && !in_list.contains(m.id.as_str()))
        .collect();
    out.sort_by(|a, b| a.position.total_cmp(&b.position));
    out
}

/// The playlist with stale ids removed, so edits start from what is shown.
fn clean(markers: &[Marker], playlist: &[String]) -> Vec<String> {
    playlist_items(markers, playlist)
        .into_iter()
        .map(|m| m.id.clone())
        .collect()
}

pub fn with_added(markers: &[Marker], playlist: &[String], ids: &[String]) -> Vec<String> {
    let mut cur = clean(markers, playlist);
    let base = cur.clone();
    cur.extend(ids.iter().filter(|id| !base.contains(id)).cloned());
    cur
}

pub fn with_removed(markers: &[Marker], playlist: &[String], id: &str) -> Vec<String> {
    clean(markers, playlist)
        .into_iter()
        .filter(|x| x != id)
        .collect()
}

pub fn with_moved(markers: &[Marker], playlist: &[String], id: &str, by: i64) -> Vec<String> {
    let mut cur = clean(markers, playlist);
    let Some(i) = cur.iter().position(|x| x == id) else {
        return cur;
    };
    let j = i as i64 + by;
    if j < 0 || j >= cur.len() as i64 {
        return cur;
    }
    cur.swap(i, j as usize);
    cur
}

/// While previewing item `index`: where playback should go once the playhead
/// passes its end, as (next index, position). None means stop.
pub fn next_start(items: &[&Marker], index: usize) -> Option<(usize, f64)> {
    items.get(index + 1).map(|n| (index + 1, n.position))
}

/// File names for exporting each item: "01 Cold open", safe for any system, unique.
pub fn export_names(items: &[&Marker]) -> Vec<String> {
    let mut used: HashSet<String> = HashSet::new();
    let digits = items.len().to_string().len();
    let width = digits.max(2);
    items
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let raw = format!("{:0>width$} {}", i + 1, m.name);
            let base = tidy(&raw);
            let mut name = base.clone();
            let mut k = 2;
            while used.contains(&name.to_lowercase()) {
                name = format!("{base} ({k})");
                k += 1;
            }
            used.insert(name.to_lowercase());
            name
        })
        .collect()
}

/// Characters a file name cannot hold, and control characters, become a space;
/// runs of white space become one; the ends are trimmed.
fn tidy(s: &str) -> String {
    let bad = |c: char| {
        matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c <= '\u{1f}'
    };
    let mut out = String::new();
    let mut in_run = false;
    for c in s.chars() {
        if bad(c) || is_js_space(c) {
            if !in_run {
                out.push(' ');
            }
            in_run = true;
        } else {
            out.push(c);
            in_run = false;
        }
    }
    js_trim(&out).to_string()
}
