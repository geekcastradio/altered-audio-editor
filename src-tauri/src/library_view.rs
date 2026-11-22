// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! How the Libraries panel groups and orders items, and how the Files panel
//! sorts and searches its list. The Rust copy of the pure parts of
//! `src/panels/libraries/arrange.ts` and `src/panels/files/format.ts`; both are
//! tested against `shared-fixtures/library-view.json`.
//!
//! Names sort the way a person expects: "Take 2" before "Take 10", capitals
//! ignored. The interface gets this from the browser's locale collation; here
//! `natural_cmp` copies it for names made of letters, digits and spaces (a
//! space sorts before a digit, a digit before a letter). Other characters sort
//! by their code, which is not the browser's order, so the tests stay inside
//! that range. When the interface moves to Rust, this is the one place that
//! needs a real collation library.

use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

fn class_of(c: char) -> u8 {
    if c == ' ' {
        0
    } else if c.is_ascii_digit() {
        1
    } else if c.is_alphabetic() {
        2
    } else {
        3
    }
}

/// Compares names with numbers by value and letters without regard to case.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i].is_ascii_digit() && b[j].is_ascii_digit() {
            let run = |s: &[char], from: usize| {
                let mut end = from;
                while end < s.len() && s[end].is_ascii_digit() {
                    end += 1;
                }
                (
                    end,
                    s[from..end]
                        .iter()
                        .skip_while(|c| **c == '0')
                        .collect::<String>(),
                )
            };
            let (ei, da) = run(&a, i);
            let (ej, db) = run(&b, j);
            let by_value = da.len().cmp(&db.len()).then_with(|| da.cmp(&db));
            if by_value != Ordering::Equal {
                return by_value;
            }
            i = ei;
            j = ej;
            continue;
        }
        let ka = (class_of(a[i]), a[i].to_lowercase().next().unwrap_or(a[i]));
        let kb = (class_of(b[j]), b[j].to_lowercase().next().unwrap_or(b[j]));
        if ka != kb {
            return ka.cmp(&kb);
        }
        i += 1;
        j += 1;
    }
    (a.len() - i).cmp(&(b.len() - j))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewState {
    pub group_by: String,
    pub sort_by: String,
    pub filter: String,
    pub view: String,
    /// Section ids the user opened. Every other section starts folded shut.
    pub open: Vec<String>,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState {
            group_by: "bin".into(),
            sort_by: "custom".into(),
            filter: "all".into(),
            view: "list".into(),
            open: vec![],
        }
    }
}

/// A saved view, with anything unreadable put back to its default.
pub fn sanitize_view(raw: &Value) -> ViewState {
    let d = ViewState::default();
    let pick = |key: &str, allowed: &[&str], def: &str| match raw.get(key).and_then(Value::as_str) {
        Some(s) if allowed.contains(&s) => s.to_string(),
        _ => def.to_string(),
    };
    ViewState {
        group_by: pick("groupBy", &["bin", "type", "none"], &d.group_by),
        sort_by: pick("sortBy", &["custom", "name", "date"], &d.sort_by),
        filter: pick("filter", &["all", "audio", "image"], &d.filter),
        view: pick("view", &["list", "grid"], &d.view),
        open: raw
            .get("open")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .take(200)
                    .collect()
            })
            .unwrap_or_default(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub name: String,
    /// "audio" or "image".
    pub kind: String,
    /// When it was added, as sortable text.
    pub added: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bin {
    pub id: String,
    pub name: String,
    pub items: Vec<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    /// Folding key: a bin id, "type:audio", "unsorted", or "all".
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    /// The bin that dropping items on this section files them into, if it is one.
    pub bin: Option<String>,
    pub items: Vec<Item>,
}

fn by_name(a: &Item, b: &Item) -> Ordering {
    natural_cmp(&a.name, &b.name)
}

fn by_date(a: &Item, b: &Item) -> Ordering {
    // Newest first; the same moment falls back to the name.
    b.added.cmp(&a.added).then_with(|| by_name(a, b))
}

fn sorted(items: &[Item], by: &str) -> Vec<Item> {
    let mut v = items.to_vec();
    match by {
        "name" => v.sort_by(by_name),
        "date" => v.sort_by(by_date),
        _ => {}
    }
    v
}

fn section(
    id: &str,
    name: &str,
    color: Option<String>,
    bin: Option<String>,
    items: Vec<Item>,
) -> Section {
    Section {
        id: id.into(),
        name: name.into(),
        color,
        bin,
        items,
    }
}

/// The items to show, in sections. Empty sections are dropped, except bins when
/// grouping by bin.
pub fn arrange(collections: &[Bin], items: &[Item], v: &ViewState) -> Vec<Section> {
    let kept: Vec<Item> = if v.filter == "all" {
        items.to_vec()
    } else {
        items
            .iter()
            .filter(|i| i.kind == v.filter)
            .cloned()
            .collect()
    };
    if v.group_by == "none" {
        return vec![section("all", "All", None, None, sorted(&kept, &v.sort_by))];
    }
    if v.group_by == "type" {
        let audio: Vec<Item> = kept.iter().filter(|i| i.kind == "audio").cloned().collect();
        let images: Vec<Item> = kept.iter().filter(|i| i.kind == "image").cloned().collect();
        return vec![
            section(
                "type:audio",
                "Audio",
                None,
                None,
                sorted(&audio, &v.sort_by),
            ),
            section(
                "type:image",
                "Pictures",
                None,
                None,
                sorted(&images, &v.sort_by),
            ),
        ]
        .into_iter()
        .filter(|s| !s.items.is_empty())
        .collect();
    }
    // The map keeps the last item for a repeated id, like a JavaScript Map.
    let by_id: HashMap<&str, &Item> = kept.iter().map(|i| (i.id.as_str(), i)).collect();
    let mut binned: HashSet<String> = HashSet::new();
    let mut sections: Vec<Section> = collections
        .iter()
        .map(|c| {
            let in_bin: Vec<Item> = c
                .items
                .iter()
                .filter_map(|id| by_id.get(id.as_str()).map(|i| (*i).clone()))
                .collect();
            for i in &in_bin {
                binned.insert(i.id.clone());
            }
            section(
                &c.id,
                &c.name,
                c.color.clone(),
                Some(c.id.clone()),
                sorted(&in_bin, &v.sort_by),
            )
        })
        .collect();
    let loose: Vec<Item> = kept
        .iter()
        .filter(|i| !binned.contains(&i.id))
        .cloned()
        .collect();
    if !loose.is_empty() || sections.is_empty() {
        let name = if sections.is_empty() {
            "All"
        } else {
            "Unsorted"
        };
        sections.push(section(
            "unsorted",
            name,
            None,
            None,
            sorted(&loose, &v.sort_by),
        ));
    }
    sections
}

/// Item ids in the order they are drawn, each once, for range selection.
pub fn flat_ids(sections: &[Section], collapsed: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for s in sections {
        if collapsed.contains(&s.id) {
            continue;
        }
        for i in &s.items {
            if seen.insert(i.id.clone()) {
                out.push(i.id.clone());
            }
        }
    }
    out
}

/// The sections drawn folded shut. Sections start folded and stay open once the
/// user opens them. A search opens every section so its matches show, and a
/// list with only one section is never folded, since it would show nothing.
pub fn folded_ids(sections: &[Section], v: &ViewState, searching: bool) -> Vec<String> {
    if searching || sections.len() <= 1 {
        return Vec::new();
    }
    sections
        .iter()
        .filter(|s| !v.open.contains(&s.id))
        .map(|s| s.id.clone())
        .collect()
}

/// Opens a folded section, or folds an open one. `folded` is whether it is
/// drawn folded now.
pub fn toggle_section(v: &ViewState, id: &str, folded: bool) -> ViewState {
    let mut out = v.clone();
    out.open = if folded {
        if v.open.iter().any(|x| x == id) {
            v.open.clone()
        } else {
            v.open.iter().cloned().chain([id.to_string()]).collect()
        }
    } else {
        v.open.iter().filter(|x| *x != id).cloned().collect()
    };
    out
}

// ---- Files panel list ----

#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub sample_rate: f64,
    pub channels: f64,
    /// Length in sample frames at the file's own rate.
    pub length: f64,
    pub format: String,
    pub codec: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileRow {
    pub path: String,
    pub name: String,
    /// "probing", "ready" or "error".
    pub status: String,
    pub info: Option<FileInfo>,
}

enum SortValue {
    Text(String),
    Number(f64),
}

/// Sorts a copy. Items still being read sort after measured ones. `key` is
/// "name", "status", "duration", "rate" or "channels"; None keeps the order.
pub fn sort_items(items: &[FileRow], key: Option<&str>, descending: bool) -> Vec<FileRow> {
    let mut v = items.to_vec();
    let Some(key) = key else { return v };
    let value = |it: &FileRow| -> Option<SortValue> {
        match key {
            "name" => Some(SortValue::Text(it.name.to_lowercase())),
            "status" => Some(SortValue::Text(it.status.clone())),
            "duration" => it
                .info
                .as_ref()
                .filter(|i| i.sample_rate > 0.0)
                .map(|i| SortValue::Number(i.length / i.sample_rate)),
            "rate" => it.info.as_ref().map(|i| SortValue::Number(i.sample_rate)),
            "channels" => it.info.as_ref().map(|i| SortValue::Number(i.channels)),
            _ => None,
        }
    };
    let dir = if descending { -1.0 } else { 1.0 };
    v.sort_by(|a, b| match (value(a), value(b)) {
        (None, None) => Ordering::Equal,
        (None, _) => Ordering::Greater,
        (_, None) => Ordering::Less,
        (Some(SortValue::Text(x)), Some(SortValue::Text(y))) => {
            let o = natural_cmp(&x, &y);
            if descending {
                o.reverse()
            } else {
                o
            }
        }
        (Some(SortValue::Number(x)), Some(SortValue::Number(y))) => {
            (dir * (x - y)).partial_cmp(&0.0).unwrap_or(Ordering::Equal)
        }
        _ => Ordering::Equal,
    });
    v
}

/// Search box match on name, folder, format or codec: every word must appear.
pub fn matches(item: &FileRow, query: &str) -> bool {
    let q = crate::js_compat::js_trim(query).to_lowercase();
    if q.is_empty() {
        return true;
    }
    let (format, codec) = item
        .info
        .as_ref()
        .map_or(("", ""), |i| (i.format.as_str(), i.codec.as_str()));
    let hay = [item.path.as_str(), format, codec].join(" ").to_lowercase();
    q.split(crate::js_compat::is_js_space)
        .filter(|w| !w.is_empty())
        .all(|w| hay.contains(w))
}
