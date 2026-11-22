// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The text a person can copy into a bug report. The Rust copy of
//! `src/app/bug-info.ts`; both are tested against
//! `shared-fixtures/app-rules.json`. User names in paths are replaced before
//! anything is shown or copied.

use crate::js_compat::is_js_space;

/// Errors kept, and stack lines kept per error.
pub const KEEP: usize = 20;
pub const STACK_LINES: usize = 14;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedError {
    pub at: String,
    pub message: String,
    pub stack: String,
}

fn is_sep(c: char) -> bool {
    c == '\\' || c == '/'
}

/// A character that can be part of a user name in a path.
fn is_name(c: char, with_colon_and_paren: bool) -> bool {
    !is_sep(c) && !is_js_space(c) && !(with_colon_and_paren && (c == ':' || c == ')'))
}

/// Length of a match for `[A-Za-z]:[\\/]+Users[\\/]+[^\\/\s:)]+` at the start
/// of `s` (letters compared without case), or None.
fn windows_user(s: &[char]) -> Option<usize> {
    if s.len() < 3 || !s[0].is_ascii_alphabetic() || s[1] != ':' {
        return None;
    }
    let mut i = 2;
    let start = i;
    while i < s.len() && is_sep(s[i]) {
        i += 1;
    }
    if i == start
        || i + 5 > s.len()
        || !s[i..i + 5]
            .iter()
            .collect::<String>()
            .eq_ignore_ascii_case("users")
    {
        return None;
    }
    i += 5;
    let start = i;
    while i < s.len() && is_sep(s[i]) {
        i += 1;
    }
    if i == start {
        return None;
    }
    let start = i;
    while i < s.len() && is_name(s[i], true) {
        i += 1;
    }
    (i > start).then_some(i)
}

/// Length of a match for `/(?:home|Users)/[^/\s:)]+` at the start of `s`.
fn unix_user(s: &[char]) -> Option<usize> {
    if s.first() != Some(&'/') {
        return None;
    }
    let rest: String = s[1..s.len().min(6)].iter().collect();
    let word = if rest.starts_with("home") {
        4
    } else if rest.starts_with("Users") {
        5
    } else {
        return None;
    };
    let mut i = 1 + word;
    if s.get(i) != Some(&'/') {
        return None;
    }
    i += 1;
    let start = i;
    while i < s.len() && s[i] != '/' && is_name(s[i], true) {
        i += 1;
    }
    (i > start).then_some(i)
}

fn replace_all(text: &str, find: fn(&[char]) -> Option<usize>) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        match find(&chars[i..]) {
            Some(n) => {
                out.push_str("<user>");
                i += n;
            }
            None => {
                out.push(chars[i]);
                i += 1;
            }
        }
    }
    out
}

/// Replaces the user name in Windows, Linux and macOS home paths.
pub fn scrub(text: &str) -> String {
    replace_all(&replace_all(text, windows_user), unix_user)
}

/// Adds an error to the list, scrubbed and with its stack cut short, and drops
/// the oldest past `KEEP`.
pub fn record_error(list: &mut Vec<RecordedError>, at: &str, message: &str, stack: &str) {
    let stack: Vec<&str> = stack.split('\n').take(STACK_LINES).collect();
    list.push(RecordedError {
        at: at.to_string(),
        message: scrub(message),
        stack: scrub(&stack.join("\n")),
    });
    if list.len() > KEEP {
        list.remove(0);
    }
}

pub struct Context<'a> {
    pub app_version: &'a str,
    pub system: &'a str,
    pub state: &'a [String],
    pub plugins: &'a [String],
    pub audio: &'a [String],
    pub startup: &'a [String],
}

/// The full text: version, system, state, audio, plug-ins, startup timing and
/// the errors recorded since the app started.
pub fn bug_info_text(ctx: &Context, list: &[RecordedError]) -> String {
    let mut out: Vec<String> = vec![
        format!("Altered Audio Editor {}", ctx.app_version),
        scrub(ctx.system),
    ];
    out.extend(ctx.state.iter().cloned());
    out.extend(ctx.audio.iter().map(|a| scrub(a)));
    out.push(if ctx.plugins.is_empty() {
        "Plug-ins in use: none".to_string()
    } else {
        format!("Plug-ins in use: {}", ctx.plugins.join(", "))
    });
    if !ctx.startup.is_empty() {
        out.push(String::new());
        out.push("Startup timing:".to_string());
        out.extend(ctx.startup.iter().map(|s| scrub(s)));
    }
    if list.is_empty() {
        out.push(String::new());
        out.push("No errors recorded since the app started.".to_string());
    } else {
        out.push(String::new());
        out.push(format!(
            "Errors since the app started (newest last, {}):",
            list.len()
        ));
        for e in list {
            out.push(String::new());
            out.push(format!("[{}] {}", e.at, e.message));
            if !e.stack.is_empty() {
                out.push(e.stack.clone());
            }
        }
    }
    out.join("\n")
}
