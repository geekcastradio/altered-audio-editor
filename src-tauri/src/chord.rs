// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Key chords as text, for example "Ctrl+Shift+K": one canonical spelling per chord,
//! with modifiers in the order Ctrl, Alt, Shift and then the key. The Rust copy of
//! `src/keymap/chord.ts`, tested against `shared-fixtures/keys-tempo-midi.json`.

use crate::js_compat::js_trim;

/// The key names a chord can use, by their lower-case spellings.
fn named(lower: &str) -> Option<&'static str> {
    Some(match lower {
        " " | "space" | "spacebar" => "Space",
        "del" | "delete" => "Delete",
        "backspace" => "Backspace",
        "enter" | "return" => "Enter",
        "esc" | "escape" => "Escape",
        "tab" => "Tab",
        "insert" | "ins" => "Insert",
        "home" => "Home",
        "end" => "End",
        "pageup" => "PageUp",
        "pagedown" => "PageDown",
        "up" | "arrowup" => "ArrowUp",
        "down" | "arrowdown" => "ArrowDown",
        "left" | "arrowleft" => "ArrowLeft",
        "right" | "arrowright" => "ArrowRight",
        _ => return None,
    })
}

const MODIFIER_KEYS: [&str; 6] = ["Control", "Alt", "Shift", "Meta", "AltGraph", "OS"];

/// The length in UTF-16 units, which is what a JavaScript string reports.
fn units(s: &str) -> usize {
    s.encode_utf16().count()
}

/// `F1` to `F24` from a lower-case `f1` to `f24`.
fn function_key(lower: &str) -> Option<String> {
    let n = lower.strip_prefix('f')?;
    if n.is_empty() || n.len() > 2 || !n.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let v: u32 = n.parse().ok()?;
    // Leading zeros ("f01") do not match, so the text must be the number as written.
    if n != v.to_string() || !(1..=24).contains(&v) {
        return None;
    }
    Some(format!("F{v}"))
}

fn canonical_key(raw: &str) -> Option<String> {
    if units(raw) == 1 {
        return Some(if raw == " " {
            "Space".to_string()
        } else {
            raw.to_uppercase()
        });
    }
    let lower = raw.to_lowercase();
    if let Some(n) = named(&lower) {
        return Some(n.to_string());
    }
    function_key(&lower)
}

/// Parses a chord string. Returns the canonical form, or None if it is not valid.
pub fn parse_chord(text: &str) -> Option<String> {
    let trimmed = js_trim(text);
    if trimmed.is_empty() {
        return None;
    }
    // A chord can end in "+" as its key, as in "Ctrl++".
    let (key_part, mod_part) = if let Some(rest) = trimmed.strip_suffix("++") {
        ("+", rest)
    } else if trimmed == "+" {
        ("+", "")
    } else {
        match trimmed.rfind('+') {
            Some(i) => (&trimmed[i + 1..], &trimmed[..i]),
            None => (trimmed, ""),
        }
    };

    let (mut ctrl, mut alt, mut shift) = (false, false, false);
    for m in mod_part.split('+').filter(|m| !m.is_empty()) {
        match js_trim(m).to_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" => alt = true,
            "shift" => shift = true,
            _ => return None,
        }
    }

    let tk = js_trim(key_part);
    let key = canonical_key(if tk.is_empty() { key_part } else { tk })?;
    Some(join(ctrl, alt, shift, &key))
}

fn join(ctrl: bool, alt: bool, shift: bool, key: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if ctrl {
        parts.push("Ctrl");
    }
    if alt {
        parts.push("Alt");
    }
    if shift {
        parts.push("Shift");
    }
    parts.push(key);
    parts.join("+")
}

/// What the interface knows about a key press.
#[derive(Debug, Clone, Default)]
pub struct KeyEvent {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
    /// The physical key (KeyT, Backspace), used when `key` does not name one.
    pub code: Option<String>,
}

/// The key a physical key code names, for the keys shortcuts use.
fn key_from_code(code: Option<&str>) -> Option<String> {
    let code = code?;
    if code.is_empty() {
        return None;
    }
    if let Some(l) = code.strip_prefix("Key") {
        if l.len() == 1 && l.as_bytes()[0].is_ascii_uppercase() {
            return Some(l.to_string());
        }
    }
    for p in ["Digit", "Numpad"] {
        if let Some(d) = code.strip_prefix(p) {
            if d.len() == 1 && d.as_bytes()[0].is_ascii_digit() {
                return Some(d.to_string());
            }
        }
    }
    canonical_key(code)
}

/// Builds the chord for a key event, or None for a lone modifier key.
pub fn chord_from_event(e: &KeyEvent) -> Option<String> {
    if MODIFIER_KEYS.contains(&e.key.as_str()) {
        return None;
    }
    let mut key = canonical_key(&e.key);
    // On Windows, Ctrl+Alt counts as the AltGr key: the key then reads as what AltGr
    // types (a control character for Backspace, or a symbol on some layouts). Read the
    // physical key then.
    let single = units(&e.key) == 1;
    let unit = if single {
        e.key.encode_utf16().next().unwrap_or(0) as i64
    } else {
        -1
    };
    let control = unit >= 0 && (unit < 0x20 || unit == 0x7f);
    let alt_gr = e.ctrl && e.alt && single && !e.key.chars().all(|c| c.is_ascii_alphanumeric());
    if key.is_none() || control || alt_gr {
        key = key_from_code(e.code.as_deref()).or(if control { None } else { key });
    }
    let key = key?;
    Some(join(e.ctrl || e.meta, e.alt, e.shift, &key))
}
