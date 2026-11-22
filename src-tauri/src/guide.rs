// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Reading the User Guide's Markdown into sections, and searching them.
//! Matches `src/app/guide/guide.ts`; both are checked against
//! `shared-fixtures/guide-media.json`.

use crate::js_compat::{is_js_space, js_trim};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Span {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Block {
    P { spans: Vec<Span> },
    Ul { items: Vec<Vec<Span>> },
    Ol { items: Vec<Vec<Span>> },
    Img { name: String, alt: String },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GuideSection {
    pub id: String,
    pub title: String,
    pub blocks: Vec<Block>,
    /// Plain text, lower case, for searching.
    pub text: String,
}

fn plain_span(text: &str) -> Span {
    Span {
        text: text.to_string(),
        bold: None,
        code: None,
    }
}

/// A character `.` does not match in a JavaScript pattern.
fn is_term(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

/// The first `**bold**` or `` `code` `` run at or after `from`, as (start, end, text, is_bold).
fn next_run(chars: &[char], from: usize) -> Option<(usize, usize, String, bool)> {
    let n = chars.len();
    let mut i = from;
    while i < n {
        if chars[i] == '*'
            && i + 1 < n
            && chars[i + 1] == '*'
            && i + 2 < n
            && !is_term(chars[i + 2])
        {
            // One or more characters, then the first `**` after them.
            let mut k = i + 3;
            while k + 1 < n {
                if chars[k] == '*' && chars[k + 1] == '*' {
                    let text: String = chars[i + 2..k].iter().collect();
                    return Some((i, k + 2, text, true));
                }
                if is_term(chars[k]) {
                    break;
                }
                k += 1;
            }
        }
        if chars[i] == '`' {
            let mut j = i + 2;
            if i + 1 < n && !is_term(chars[i + 1]) {
                while j < n {
                    if chars[j] == '`' {
                        let text: String = chars[i + 1..j].iter().collect();
                        return Some((i, j + 1, text, false));
                    }
                    if is_term(chars[j]) {
                        break;
                    }
                    j += 1;
                }
            }
        }
        i += 1;
    }
    None
}

/// Splits a line into plain, bold and code runs.
pub fn spans(line: &str) -> Vec<Span> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut last = 0;
    let mut from = 0;
    while let Some((start, end, text, bold)) = next_run(&chars, from) {
        if start > last {
            out.push(plain_span(&chars[last..start].iter().collect::<String>()));
        }
        out.push(Span {
            text,
            bold: bold.then_some(true),
            code: (!bold).then_some(true),
        });
        last = end;
        from = end;
    }
    if last < chars.len() {
        out.push(plain_span(&chars[last..].iter().collect::<String>()));
    }
    out
}

fn plain(s: &[Span]) -> String {
    s.iter().map(|x| x.text.as_str()).collect()
}

fn slug(t: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in t.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    let out = out.strip_prefix('-').unwrap_or(&out);
    out.strip_suffix('-').unwrap_or(out).to_string()
}

fn trim_end_js(s: &str) -> &str {
    s.trim_end_matches(is_js_space)
}

/// `##` then white space then text with no line break in it.
fn heading(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("##")?;
    let title = rest.trim_start_matches(is_js_space);
    if title.len() == rest.len() || title.is_empty() || title.chars().any(is_term) {
        return None;
    }
    Some(title)
}

/// `![alt](name)` filling the whole line.
fn image(line: &str) -> Option<(String, String)> {
    let body = line.strip_prefix("![")?;
    if body.chars().any(is_term) {
        return None;
    }
    let mut from = 0;
    while let Some(off) = body[from..].find("](") {
        let p = from + off;
        let rest = &body[p + 2..];
        if rest.len() >= 2 && rest.ends_with(')') {
            return Some((body[..p].to_string(), rest[..rest.len() - 1].to_string()));
        }
        from = p + 1;
    }
    None
}

/// `-` or `1.` then white space then text; the text and which kind of list.
fn list_item(line: &str) -> Option<(bool, &str)> {
    fn after(rest: &str) -> Option<&str> {
        let text = rest.trim_start_matches(is_js_space);
        if text.len() == rest.len() || text.is_empty() || text.chars().any(is_term) {
            None
        } else {
            Some(text)
        }
    }
    if let Some(rest) = line.strip_prefix('-') {
        return after(rest).map(|t| (true, t));
    }
    let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 {
        if let Some(rest) = line[digits..].strip_prefix('.') {
            return after(rest).map(|t| (false, t));
        }
    }
    None
}

pub fn parse_guide(md: &str) -> Vec<GuideSection> {
    let mut sections: Vec<GuideSection> = Vec::new();
    let mut para: Vec<String> = Vec::new();
    let mut list: Option<(bool, Vec<Vec<Span>>)> = None;
    let mut have = false;

    fn flush_para(sections: &mut [GuideSection], have: bool, para: &mut Vec<String>) {
        if have && !para.is_empty() {
            if let Some(cur) = sections.last_mut() {
                cur.blocks.push(Block::P {
                    spans: spans(&para.join(" ")),
                });
            }
        }
        para.clear();
    }
    fn flush_list(
        sections: &mut [GuideSection],
        have: bool,
        list: &mut Option<(bool, Vec<Vec<Span>>)>,
    ) {
        if let Some((bullet, items)) = list.take() {
            if have {
                if let Some(cur) = sections.last_mut() {
                    cur.blocks.push(if bullet {
                        Block::Ul { items }
                    } else {
                        Block::Ol { items }
                    });
                }
            }
        }
    }

    for raw in md.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let line = trim_end_js(raw);
        if let Some(title) = heading(line) {
            flush_para(&mut sections, have, &mut para);
            flush_list(&mut sections, have, &mut list);
            sections.push(GuideSection {
                id: slug(title),
                title: js_trim(title).to_string(),
                blocks: Vec::new(),
                text: String::new(),
            });
            have = true;
            continue;
        }
        if !have {
            continue;
        }
        if let Some((alt, name)) = image(js_trim(line)) {
            flush_para(&mut sections, have, &mut para);
            flush_list(&mut sections, have, &mut list);
            sections
                .last_mut()
                .unwrap()
                .blocks
                .push(Block::Img { name, alt });
            continue;
        }
        if let Some((bullet, text)) = list_item(line) {
            flush_para(&mut sections, have, &mut para);
            if matches!(&list, Some((b, _)) if *b != bullet) {
                flush_list(&mut sections, have, &mut list);
            }
            list.get_or_insert((bullet, Vec::new())).1.push(spans(text));
            continue;
        }
        if js_trim(line).is_empty() {
            flush_para(&mut sections, have, &mut para);
            flush_list(&mut sections, have, &mut list);
            continue;
        }
        // A wrapped line continues the paragraph or the last list item.
        if let Some((_, items)) = list.as_mut() {
            let last = items.last_mut().unwrap();
            last.extend(spans(&format!(" {}", js_trim(line))));
        } else {
            para.push(js_trim(line).to_string());
        }
    }
    flush_para(&mut sections, have, &mut para);
    flush_list(&mut sections, have, &mut list);
    for s in &mut sections {
        let body: Vec<String> = s
            .blocks
            .iter()
            .map(|b| match b {
                Block::P { spans } => plain(spans),
                Block::Img { alt, .. } => alt.clone(),
                Block::Ul { items } | Block::Ol { items } => {
                    items.iter().map(|i| plain(i)).collect::<Vec<_>>().join(" ")
                }
            })
            .collect();
        s.text = format!("{} {}", s.title, body.join(" ")).to_lowercase();
    }
    sections
}

/// Sections that contain every word of `query` (all of them for an empty query).
pub fn search_guide<'a>(sections: &'a [GuideSection], query: &str) -> Vec<&'a GuideSection> {
    let lower = query.to_lowercase();
    let words: Vec<&str> = lower.split(is_js_space).filter(|w| !w.is_empty()).collect();
    sections
        .iter()
        .filter(|s| words.iter().all(|w| s.text.contains(w)))
        .collect()
}
