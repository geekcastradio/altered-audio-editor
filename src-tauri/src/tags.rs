// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The tag fields the File Tags dialog edits. The Rust copy of the helpers in
//! `src/app/tags.ts`; both are tested against `shared-fixtures/tags.json`.

use crate::js_compat::js_trim;
use std::collections::HashMap;

/// The fields, in the order the dialog shows them.
pub const TAG_FIELDS: [&str; 7] = [
    "title", "artist", "album", "year", "genre", "track", "comment",
];

/// Formats that take tags when saved (matched on the end of the path, ignoring case).
pub fn is_taggable(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    ["wav", "wave", "flac", "mp3", "m4a", "mp4", "m4b"]
        .iter()
        .any(|ext| lower.ends_with(&format!(".{ext}")))
}

/// The tags worth writing: trimmed, with blank fields left out so they don't
/// touch what the file has.
pub fn clean_tags(draft: &HashMap<String, String>) -> Vec<(&'static str, String)> {
    TAG_FIELDS
        .iter()
        .filter_map(|&f| {
            let v = js_trim(draft.get(f).map_or("", String::as_str));
            (!v.is_empty()).then(|| (f, v.to_string()))
        })
        .collect()
}

/// The tags to save from the editor: filled fields trimmed, blank ones as ""
/// so the file loses those tags.
pub fn save_tags(draft: &HashMap<String, String>) -> Vec<(&'static str, String)> {
    TAG_FIELDS
        .iter()
        .map(|&f| {
            (
                f,
                js_trim(draft.get(f).map_or("", String::as_str)).to_string(),
            )
        })
        .collect()
}
