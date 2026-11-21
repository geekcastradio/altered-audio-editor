// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Type to jump in a list or menu. The Rust copy of `src/app/typeahead.ts`;
//! both are tested against `shared-fixtures/typeahead.json`.

use crate::js_compat::js_trim;

/// How long a pause starts a new prefix, in milliseconds.
pub const PAUSE_MS: f64 = 800.0;

/// True for a key that types a letter, digit or symbol (not Space, not a named key).
pub fn is_typing_key(key: &str, ctrl: bool, alt: bool, meta: bool) -> bool {
    key.encode_utf16().count() == 1 && key != " " && !ctrl && !alt && !meta
}

/// The typed prefix and when the last key arrived.
#[derive(Debug, Default, Clone)]
pub struct Typeahead {
    prefix: String,
    last: f64,
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

impl Typeahead {
    pub fn new() -> Self {
        Self::default()
    }

    /// The row to jump to after `key` is typed, or None when the key is not a
    /// letter to jump by or nothing matches. `labels` is every row's label
    /// (None for rows that cannot be picked); `current` is the highlighted row.
    pub fn next(
        &mut self,
        key: &str,
        labels: &[Option<&str>],
        current: i64,
        now: f64,
    ) -> Option<usize> {
        if utf16_len(key) != 1 || key == " " {
            return None;
        }
        let k = key.to_lowercase();
        if now - self.last > PAUSE_MS {
            self.prefix = k.clone();
        } else {
            self.prefix.push_str(&k);
        }
        self.last = now;
        // The same letter again: step through the rows starting with it.
        let repeat = utf16_len(&self.prefix) > 1 && self.prefix.chars().all(|c| c.to_string() == k);
        let want = if repeat { k } else { self.prefix.clone() };
        let n = labels.len() as i64;
        // A longer prefix may still match the row already highlighted.
        let from = if utf16_len(&want) > 1 && current >= 0 {
            current
        } else {
            current + 1
        };
        for i in 0..n {
            let at = (((from + i) % n) + n) % n;
            if let Some(l) = labels[at as usize] {
                if js_trim(l).to_lowercase().starts_with(&want) {
                    return Some(at as usize);
                }
            }
        }
        None
    }
}
