// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Bringing shortcuts in from another editor's shortcut export (an XML file). The Rust copy
//! of `src/keymap/audition-import.ts`, tested against `shared-fixtures/shortcut-import.json`.
//!
//! Each command in the export that has a matching action here takes the chords the export
//! holds for it. Commands with no matching action, and keys that cannot be decoded, are
//! counted so nothing is lost silently.

use crate::chord::parse_chord;
use crate::js_compat::{is_js_space, js_trim};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::OnceLock;

/// An action id to its chords, in order.
pub type Bindings = BTreeMap<String, Vec<String>>;

/// What an export holds, in our terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutImport {
    /// Our action id to the chords the export gives its command.
    pub bindings: Bindings,
    /// Commands with a shortcut and no matching action here, sorted.
    pub unmapped: Vec<String>,
    /// Entries whose key could not be read.
    pub unreadable: usize,
}

/// The export's command names to our action ids. The same file the interface reads.
fn actions() -> &'static HashMap<String, String> {
    static MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
    MAP.get_or_init(|| {
        serde_json::from_str(include_str!("../../src/keymap/audition-map.json"))
            .expect("audition-map.json")
    })
}

/// The export's numbers for keys that have no character.
fn key_name(n: u64) -> Option<&'static str> {
    Some(match n {
        1 => "Space",
        2 => "Backspace",
        3 => "Tab",
        35 => "Delete",
        36 => "Home",
        37 => "End",
        38 => "PageUp",
        39 => "PageDown",
        42 => "ArrowLeft",
        43 => "ArrowRight",
        44 => "ArrowUp",
        45 => "ArrowDown",
        _ => return None,
    })
}

fn modifier(tag: &str) -> Option<&'static str> {
    match tag {
        "cmdkey" => Some("Ctrl"),
        "alt" => Some("Alt"),
        "shift" => Some("Shift"),
        _ => None,
    }
}

/// `<tag>text</tag>` where the text has no `<` in it: the first such match, as `<tag>([^<]+)</tag>`.
fn tag_text<'a>(body: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut from = 0;
    while let Some(i) = body[from..].find(&open) {
        let start = from + i + open.len();
        let end = start + body[start..].find('<').unwrap_or(body.len() - start);
        if end > start && body[end..].starts_with(&close) {
            return Some(&body[start..end]);
        }
        from += i + 1;
    }
    None
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Whether `rest` starts with optional white space and then `/>`.
fn self_closing(rest: &str) -> bool {
    rest.trim_start_matches(is_js_space).starts_with("/>")
}

/// Whether the body holds `<name/>` (white space allowed before the slash).
fn has_empty_tag(body: &str, name: &str) -> bool {
    let open = format!("<{name}");
    let mut from = 0;
    while let Some(i) = body[from..].find(&open) {
        let after = from + i + open.len();
        if self_closing(&body[after..]) {
            return true;
        }
        from += i + 1;
    }
    false
}

/// The names of the empty tags `<name/>` inside the first `<mod>` block.
fn mod_names(body: &str) -> Vec<&str> {
    let Some(start) = body.find("<mod>") else {
        return Vec::new();
    };
    let start = start + "<mod>".len();
    let Some(len) = body[start..].find("</mod>") else {
        return Vec::new();
    };
    let inner = &body[start..start + len];
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(i) = inner[pos..].find('<') {
        let name_start = pos + i + 1;
        let name_len = inner[name_start..]
            .find(|c: char| !is_word(c))
            .unwrap_or(inner.len() - name_start);
        let after = name_start + name_len;
        if name_len > 0 && self_closing(&inner[after..]) {
            out.push(&inner[name_start..after]);
            pos = after;
        } else {
            pos = name_start;
        }
    }
    out
}

fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// Whether the text has a `<keymap` start tag, as `<keymap[\s>]`.
fn is_export(xml: &str) -> bool {
    let mut from = 0;
    while let Some(i) = xml[from..].find("<keymap") {
        let after = from + i + "<keymap".len();
        if let Some(c) = xml[after..].chars().next() {
            if c == '>' || is_js_space(c) {
                return true;
            }
        }
        from += i + 1;
    }
    false
}

/// Reads a shortcut export. An error when the text is not one.
pub fn parse_export(xml: &str) -> Result<ShortcutImport, String> {
    if !is_export(xml) {
        return Err("That isn't a shortcut export.".to_string());
    }
    let map = actions();
    let mut bindings = Bindings::new();
    let mut unmapped = BTreeSet::new();
    let mut unreadable = 0usize;
    let mut pos = 0;
    while let Some(i) = xml[pos..].find("<keycmd>") {
        let start = pos + i + "<keycmd>".len();
        let Some(len) = xml[start..].find("</keycmd>") else {
            break;
        };
        let body = &xml[start..start + len];
        pos = start + len + "</keycmd>".len();
        // Hold and extended-keypad repeats are the same shortcut again.
        if has_empty_tag(body, "hold") || has_empty_tag(body, "ext") {
            continue;
        }
        let Some(cmd) = tag_text(body, "cmd").map(js_trim).filter(|c| !c.is_empty()) else {
            continue;
        };
        let mods: Vec<&str> = mod_names(body).into_iter().filter_map(modifier).collect();
        let key: Option<String> = match tag_text(body, "uchar") {
            Some(ch) => Some(unescape_xml(ch)),
            None => tag_text(body, "key")
                .filter(|n| n.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|n| n.parse::<u64>().ok())
                .and_then(key_name)
                .map(str::to_string),
        };
        let chord = key.and_then(|k| {
            let mut parts: Vec<&str> = mods.clone();
            parts.push(&k);
            parse_chord(&parts.join("+"))
        });
        let Some(chord) = chord else {
            unreadable += 1;
            continue;
        };
        let Some(action) = map.get(cmd) else {
            unmapped.insert(cmd.to_string());
            continue;
        };
        let list = bindings.entry(action.clone()).or_default();
        if !list.contains(&chord) {
            list.push(chord);
        }
    }
    Ok(ShortcutImport {
        bindings,
        unmapped: unmapped.into_iter().collect(),
        unreadable,
    })
}

/// The same chords, whatever their order.
fn same(a: &[String], b: &[String]) -> bool {
    a.len() == b.len() && a.iter().all(|x| b.contains(x))
}

/// The user's changes after bringing an export in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    pub overrides: Bindings,
    /// Actions that gave up a chord because the export uses it for another command, as (action, chord).
    pub taken_from: Vec<(String, String)>,
}

/// Every imported action gets the export's chords first, then keeps any of our own default
/// chords that no imported command wants. Any other action holding an imported chord loses
/// it, so no chord ever runs two things. `taken_from` lists actions in sorted order.
pub fn apply_import(imported: &Bindings, current: &Bindings, defaults: &Bindings) -> Applied {
    let mut overrides = current.clone();
    let mut owner: HashMap<&str, &str> = HashMap::new();
    for (id, chords) in imported {
        for c in chords {
            owner.insert(c, id);
        }
    }
    let none: Vec<String> = Vec::new();
    for (id, chords) in imported {
        let def = defaults.get(id).unwrap_or(&none);
        let mut merged: Vec<String> = chords.clone();
        merged.extend(
            def.iter()
                .filter(|c| !chords.contains(c) && !owner.contains_key(c.as_str()))
                .cloned(),
        );
        if same(&merged, def) {
            overrides.remove(id);
        } else {
            overrides.insert(id.clone(), merged);
        }
    }
    let mut taken_from = Vec::new();
    let ids: BTreeSet<String> = defaults.keys().chain(overrides.keys()).cloned().collect();
    for id in &ids {
        if imported.contains_key(id) {
            continue;
        }
        let chords = overrides
            .get(id)
            .or_else(|| defaults.get(id))
            .cloned()
            .unwrap_or_default();
        let kept: Vec<String> = chords
            .iter()
            .filter(|c| !owner.contains_key(c.as_str()))
            .cloned()
            .collect();
        if kept.len() == chords.len() {
            continue;
        }
        for c in &chords {
            if owner.contains_key(c.as_str()) {
                taken_from.push((id.clone(), c.clone()));
            }
        }
        if same(&kept, defaults.get(id).unwrap_or(&none)) {
            overrides.remove(id);
        } else {
            overrides.insert(id.clone(), kept);
        }
    }
    Applied {
        overrides,
        taken_from,
    }
}
