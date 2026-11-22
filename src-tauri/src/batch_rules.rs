// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Batch Convert's list, naming and saved-chain rules. The Rust copy of the pure
//! parts of `src/panels/batch/batch.ts` and `batch-chain.ts`, tested against
//! `shared-fixtures/batch-rules.json`. The list store (`batchRows`) stays in
//! TypeScript.
//!
//! Name tokens follow JavaScript's `String.replace` with a text replacement, so
//! a file name holding `$&` or `$$` fills in the same way as in the interface.

use crate::js_compat::{is_js_space, js_number_to_string, js_slice_units, js_trim};
use crate::output_format::{sanitize_format, Kind, OutputFormat};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub normalize: bool,
    pub normalize_db: f64,
    pub fade_in_ms: f64,
    pub fade_out_ms: f64,
    pub loudness: bool,
    pub loudness_lufs: f64,
    pub ceiling_db: f64,
    pub trim: bool,
    pub trim_db: f64,
    pub trim_keep_ms: f64,
}

pub const DEFAULT_LEVEL: Level = Level {
    normalize: false,
    normalize_db: -1.0,
    fade_in_ms: 0.0,
    fade_out_ms: 0.0,
    loudness: false,
    loudness_lufs: -16.0,
    ceiling_db: -1.0,
    trim: false,
    trim_db: -50.0,
    trim_keep_ms: 150.0,
};

fn bounded(x: Option<&Value>, lo: f64, hi: f64, default: f64) -> f64 {
    match x.and_then(Value::as_f64) {
        Some(v) if v.is_finite() => v.max(lo).min(hi),
        _ => default,
    }
}

fn is_true(x: Option<&Value>) -> bool {
    x.and_then(Value::as_bool) == Some(true)
}

/// Level settings read back from saved text: anything off is limited to its range.
pub fn sanitize_level(raw: &Value) -> Level {
    let g = |k: &str| raw.as_object().and_then(|m| m.get(k));
    let d = &DEFAULT_LEVEL;
    Level {
        normalize: is_true(g("normalize")),
        normalize_db: bounded(g("normalizeDb"), -30.0, 0.0, d.normalize_db),
        fade_in_ms: bounded(g("fadeInMs"), 0.0, 60_000.0, 0.0),
        fade_out_ms: bounded(g("fadeOutMs"), 0.0, 60_000.0, 0.0),
        loudness: is_true(g("loudness")),
        loudness_lufs: bounded(g("loudnessLufs"), -40.0, -5.0, d.loudness_lufs),
        ceiling_db: bounded(g("ceilingDb"), -30.0, 0.0, d.ceiling_db),
        trim: is_true(g("trim")),
        trim_db: bounded(g("trimDb"), -80.0, -10.0, d.trim_db),
        trim_keep_ms: bounded(g("trimKeepMs"), 0.0, 5_000.0, d.trim_keep_ms),
    }
}

/// What the core needs for the level work.
#[derive(Debug, Clone, PartialEq)]
pub struct Shaping {
    pub normalize_db: Option<f64>,
    pub fade_in_ms: f64,
    pub fade_out_ms: f64,
    pub loudness_lufs: Option<f64>,
    pub ceiling_db: f64,
    pub trim_db: Option<f64>,
    pub trim_keep_ms: f64,
}

/// None when the level work is all off.
pub fn shaping_for(l: &Level) -> Option<Shaping> {
    if !l.normalize && !l.loudness && !l.trim && l.fade_in_ms <= 0.0 && l.fade_out_ms <= 0.0 {
        return None;
    }
    Some(Shaping {
        normalize_db: l.normalize.then_some(l.normalize_db),
        fade_in_ms: l.fade_in_ms,
        fade_out_ms: l.fade_out_ms,
        loudness_lufs: l.loudness.then_some(l.loudness_lufs),
        ceiling_db: l.ceiling_db,
        trim_db: l.trim.then_some(l.trim_db),
        trim_keep_ms: l.trim_keep_ms,
    })
}

pub const TAG_KEYS: [&str; 6] = ["title", "artist", "album", "year", "genre", "comment"];
pub const DEFAULT_TEMPLATE: &str = "{name}";

/// Tag fields as typed in the panel, in `TAG_KEYS` order.
pub type TagFields = [String; 6];

fn sanitize_tags(raw: Option<&Value>) -> TagFields {
    let mut out: TagFields = Default::default();
    if let Some(m) = raw.and_then(Value::as_object) {
        for (i, k) in TAG_KEYS.iter().enumerate() {
            if let Some(s) = m.get(*k).and_then(Value::as_str) {
                out[i] = s.to_string();
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Naming {
    Affix,
    Template,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub format: OutputFormat,
    pub naming: Naming,
    pub prefix: String,
    pub suffix: String,
    pub template: String,
    pub process: Option<String>,
    pub level: Level,
    pub folder: Option<String>,
    pub overwrite: bool,
    pub remove_done: bool,
    pub copy_tags: bool,
    pub tags: TagFields,
}

/// Batch settings read back from saved text, with older layouts upgraded.
pub fn sanitize_batch(raw: &Value) -> Settings {
    let g = |k: &str| raw.as_object().and_then(|m| m.get(k));
    let text = |k: &str| g(k).and_then(Value::as_str).unwrap_or("").to_string();
    let template_text = text("template");
    let old = format!("{}{{name}}{}", text("prefix"), text("suffix"));
    let naming = if g("naming").and_then(Value::as_str) == Some("template")
        || (g("naming").is_none()
            && !js_trim(&template_text).is_empty()
            && template_text != DEFAULT_TEMPLATE)
    {
        Naming::Template
    } else {
        Naming::Affix
    };
    let non_empty = |k: &str| {
        g(k).and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    Settings {
        format: sanitize_format(g("format").unwrap_or(&Value::Null)),
        naming,
        prefix: text("prefix"),
        suffix: text("suffix"),
        process: non_empty("process"),
        level: sanitize_level(g("level").unwrap_or(&Value::Null)),
        template: if js_trim(&template_text).is_empty() {
            old
        } else {
            template_text
        },
        folder: non_empty("folder"),
        overwrite: is_true(g("overwrite")),
        remove_done: is_true(g("removeDone")),
        copy_tags: g("copyTags").and_then(Value::as_bool) != Some(false),
        tags: sanitize_tags(g("tags")),
    }
}

fn last_sep(p: &str) -> Option<usize> {
    p.rfind(['/', '\\'])
}

/// The part of a path after the last slash or backslash.
pub fn file_name(path: &str) -> &str {
    last_sep(path).map_or(path, |i| &path[i + 1..])
}

/// The file name without its last extension; a leading dot is not an extension.
fn stem_of(name: &str) -> &str {
    match name.rfind('.') {
        Some(dot) if dot > 0 => &name[..dot],
        _ => name,
    }
}

/// Characters Windows won't take in a file name.
fn clean_part(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if (c as u32) < 0x20 => '_',
            c => c,
        })
        .collect()
}

/// What the tokens in a name template can use.
#[derive(Debug, Clone, Copy)]
pub struct NameContext {
    /// Position in the list, starting at 1.
    pub number: i64,
    /// How many files are in the list.
    pub count: i64,
    pub year: i64,
    /// 1 to 12.
    pub month: i64,
    pub day: i64,
}

fn pad(n: i64, w: usize) -> String {
    format!("{n:0>w$}")
}

/// The template that naming settings stand for.
pub fn effective_template(naming: Naming, prefix: &str, suffix: &str, template: &str) -> String {
    match naming {
        Naming::Template => template.to_string(),
        Naming::Affix => format!("{prefix}{{name}}{suffix}"),
    }
}

/// JavaScript's expansion of `$$`, `$&`, `` $` `` and `$'` in a replacement text.
/// (`$1` and `$<` stay as written, since the token patterns have no groups.)
fn expand_replacement(repl: &str, matched: &str, before: &str, after: &str) -> String {
    let chars: Vec<char> = repl.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' && i + 1 < chars.len() {
            match chars[i + 1] {
                '$' => out.push('$'),
                '&' => out.push_str(matched),
                '`' => out.push_str(before),
                '\'' => out.push_str(after),
                _ => {
                    out.push('$');
                    i += 1;
                    continue;
                }
            }
            i += 2;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// `text.replace(/token/gi, repl)` for a token of ASCII letters and braces.
fn replace_token(text: &str, token: &str, repl: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let token = token.to_ascii_lowercase();
    let mut out = String::new();
    let mut from = 0;
    while let Some(at) = lower[from..].find(&token) {
        let start = from + at;
        let end = start + token.len();
        out.push_str(&text[from..start]);
        out.push_str(&expand_replacement(
            repl,
            &text[start..end],
            &text[..start],
            &text[end..],
        ));
        from = end;
    }
    out.push_str(&text[from..]);
    out
}

/// Fills the tokens {name}, {number}, {date} (2026-10-15) and {format}.
fn fill_tokens(text: &str, stem: &str, kind: Kind, ctx: &NameContext) -> String {
    let date = format!("{}-{}-{}", ctx.year, pad(ctx.month, 2), pad(ctx.day, 2));
    let width = 2.max(ctx.count.to_string().len());
    let t = replace_token(text, "{name}", stem);
    let t = replace_token(&t, "{number}", &pad(ctx.number, width));
    let t = replace_token(&t, "{date}", &date);
    replace_token(&t, "{format}", kind.as_str())
}

/// Fills a name template and makes it safe as a file name.
pub fn expand_template(template: &str, stem: &str, kind: Kind, ctx: &NameContext) -> String {
    let source = if js_trim(template).is_empty() {
        DEFAULT_TEMPLATE
    } else {
        template
    };
    let out = clean_part(&fill_tokens(source, stem, kind, ctx));
    let out = js_trim(&out);
    if out.is_empty() {
        clean_part(stem)
    } else {
        out.to_string()
    }
}

/// The tags one file gets: the typed fields with their tokens filled in. Blank
/// fields are left out so the copied value stays. None when nothing is set.
pub fn tags_for(
    fields: &TagFields,
    source: &str,
    kind: Kind,
    ctx: &NameContext,
) -> Option<Vec<(&'static str, String)>> {
    let stem = stem_of(file_name(source));
    let out: Vec<(&'static str, String)> = TAG_KEYS
        .iter()
        .zip(fields)
        .filter_map(|(k, v)| {
            let t = js_trim(v);
            (!t.is_empty()).then(|| (*k, fill_tokens(t, stem, kind, ctx)))
        })
        .collect();
    (!out.is_empty()).then_some(out)
}

/// Where a source file converts to.
pub fn dest_for(
    source: &str,
    template: &str,
    folder: Option<&str>,
    kind: Kind,
    ctx: &NameContext,
) -> String {
    let cut = last_sep(source);
    let base = match folder {
        Some(f) => f.to_string(),
        None => cut.map_or(".".to_string(), |c| source[..c].to_string()),
    };
    let dir = base.trim_end_matches(['/', '\\']);
    let name = file_name(source);
    let sep = if dir.contains('\\') || (folder.is_none() && source.contains('\\')) {
        '\\'
    } else {
        '/'
    };
    format!(
        "{dir}{sep}{}.{}",
        expand_template(template, stem_of(name), kind, ctx),
        kind.as_str()
    )
}

fn same_path(a: &str, b: &str) -> bool {
    a.replace('\\', "/").to_lowercase() == b.replace('\\', "/").to_lowercase()
}

/// One row of the list: its id and source path.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub id: String,
    pub source: String,
}

/// Problems that stop a row from converting: two rows writing the same file, or
/// a row writing over its own source without Replace on. Row id and message, in
/// row order.
pub fn conflicts(
    rows: &[Row],
    dests: &[(String, String)],
    overwrite: bool,
) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();
    for r in rows {
        let Some((_, d)) = dests.iter().find(|(id, _)| *id == r.id) else {
            continue;
        };
        if d.is_empty() {
            continue;
        }
        let key = d.replace('\\', "/").to_lowercase();
        let had = |out: &[(String, String)]| out.iter().any(|(id, _)| *id == r.id);
        match seen.iter().find(|(k, _)| *k == key) {
            Some((_, first)) => {
                let msg = format!(
                    "Writes the same file as {}. Add a prefix or suffix.",
                    file_name(first)
                );
                out.retain(|(id, _)| *id != r.id);
                out.push((r.id.clone(), msg));
            }
            None => seen.push((key, r.source.clone())),
        }
        if !had(&out) && same_path(d, &r.source) && !overwrite {
            out.push((
                r.id.clone(),
                "Would replace the original. Turn on Replace existing files, or add a prefix or suffix."
                    .to_string(),
            ));
        }
    }
    out
}

/// Adds files not already listed; returns the new list and how many were new.
pub fn add_rows(
    rows: &[Row],
    paths: &[String],
    mut new_id: impl FnMut() -> String,
) -> (Vec<Row>, usize) {
    let key = |p: &str| p.replace('\\', "/").to_lowercase();
    let mut have: HashSet<String> = rows.iter().map(|r| key(&r.source)).collect();
    let mut next = rows.to_vec();
    for p in paths {
        if have.insert(key(p)) {
            next.push(Row {
                id: new_id(),
                source: p.clone(),
            });
        }
    }
    let added = next.len() - rows.len();
    (next, added)
}

// Saved chains: the processing half of Batch Convert under a name.

pub const MAX_CHAINS: usize = 40;
pub const MAX_NAME: usize = 60;

#[derive(Debug, Clone, PartialEq)]
pub struct Chain {
    pub name: String,
    pub process: Option<String>,
    pub level: Level,
    pub format: Option<OutputFormat>,
}

fn clean_name(name: &str) -> String {
    let mut collapsed = String::new();
    let mut in_space = false;
    for c in name.chars() {
        if is_js_space(c) {
            if !in_space {
                collapsed.push(' ');
            }
            in_space = true;
        } else {
            collapsed.push(c);
            in_space = false;
        }
    }
    js_slice_units(js_trim(&collapsed), MAX_NAME)
}

/// Chains read back from saved settings or a shared file; anything odd is dropped.
pub fn sanitize_chains(raw: &Value) -> Vec<Chain> {
    let Some(list) = raw.as_array() else {
        return Vec::new();
    };
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<Chain> = Vec::new();
    for item in list {
        let Some(v) = item.as_object() else {
            continue;
        };
        let name = v
            .get("name")
            .and_then(Value::as_str)
            .map(clean_name)
            .unwrap_or_default();
        if name.is_empty() || !seen.insert(name.to_lowercase()) {
            continue;
        }
        out.push(Chain {
            name,
            process: v
                .get("process")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            level: sanitize_level(v.get("level").unwrap_or(&Value::Null)),
            format: v
                .get("format")
                .filter(|f| f.is_object() || f.is_array())
                .map(sanitize_format),
        });
        if out.len() >= MAX_CHAINS {
            break;
        }
    }
    out
}

/// The current Batch settings as a chain; `with_format` also keeps the format.
pub fn chain_from(name: &str, s: &Settings, with_format: bool) -> Chain {
    Chain {
        name: clean_name(name),
        process: s.process.clone(),
        level: s.level.clone(),
        format: with_format.then(|| s.format.clone()),
    }
}

/// Saves a chain: one with the same name (any case) is replaced. None when the
/// name is empty.
pub fn save_chain(chains: &[Chain], chain: &Chain) -> Option<Vec<Chain>> {
    let name = clean_name(&chain.name);
    if name.is_empty() {
        return None;
    }
    let key = name.to_lowercase();
    let mut next: Vec<Chain> = chains
        .iter()
        .filter(|c| c.name.to_lowercase() != key)
        .cloned()
        .collect();
    next.push(Chain {
        name,
        ..chain.clone()
    });
    let skip = next.len().saturating_sub(MAX_CHAINS);
    Some(next.split_off(skip))
}

pub fn remove_chain(chains: &[Chain], name: &str) -> Vec<Chain> {
    chains.iter().filter(|c| c.name != name).cloned().collect()
}

/// The Batch settings with a chain's steps in place.
pub fn apply_chain(s: &Settings, chain: &Chain) -> Settings {
    Settings {
        process: chain.process.clone(),
        level: chain.level.clone(),
        format: chain.format.clone().unwrap_or_else(|| s.format.clone()),
        ..s.clone()
    }
}

/// Plain words for a chain's steps, in the order they run.
pub fn describe_chain(c: &Chain) -> String {
    let l = &c.level;
    let n = js_number_to_string;
    let mut steps: Vec<String> = Vec::new();
    if let Some(p) = &c.process {
        steps.push(p.clone());
    }
    if l.trim {
        steps.push(format!("trim silence below {} dB", n(l.trim_db)));
    }
    if l.loudness {
        steps.push(format!(
            "{} LUFS, peaks under {} dB",
            n(l.loudness_lufs),
            n(l.ceiling_db)
        ));
    }
    if l.normalize {
        steps.push(format!("peak to {} dB", n(l.normalize_db)));
    }
    if l.fade_in_ms > 0.0 {
        steps.push(format!("{} ms fade in", n(l.fade_in_ms)));
    }
    if l.fade_out_ms > 0.0 {
        steps.push(format!("{} ms fade out", n(l.fade_out_ms)));
    }
    if let Some(f) = &c.format {
        steps.push(format!("save as {}", f.kind.as_str().to_uppercase()));
    }
    if steps.is_empty() {
        "no steps".to_string()
    } else {
        steps.join(", then ")
    }
}

/// The starter chains: the common jobs, set up with sensible numbers.
pub fn builtin_chains() -> Vec<Chain> {
    let lvl = |f: &dyn Fn(&mut Level)| {
        let mut l = DEFAULT_LEVEL;
        f(&mut l);
        l
    };
    let job = |name: &str, process: Option<&str>, level: Level| Chain {
        name: name.to_string(),
        process: process.map(str::to_string),
        level,
        format: None,
    };
    let preset = |name: &str, process: &str| job(name, Some(process), DEFAULT_LEVEL);
    vec![
        job(
            "Podcast, -16 LUFS",
            None,
            lvl(&|l| {
                l.trim = true;
                l.loudness = true;
                l.loudness_lufs = -16.0;
                l.ceiling_db = -1.0;
                l.fade_out_ms = 300.0;
            }),
        ),
        job(
            "Voiceover, trim and level",
            None,
            lvl(&|l| {
                l.trim = true;
                l.trim_keep_ms = 400.0;
                l.loudness = true;
                l.loudness_lufs = -19.0;
                l.ceiling_db = -3.0;
            }),
        ),
        job(
            "Audiobook, quiet and clean",
            None,
            lvl(&|l| {
                l.trim = true;
                l.trim_keep_ms = 700.0;
                l.loudness = true;
                l.loudness_lufs = -20.0;
                l.ceiling_db = -3.0;
            }),
        ),
        job(
            "Quick peak match, -1 dB",
            None,
            lvl(&|l| {
                l.normalize = true;
                l.normalize_db = -1.0;
            }),
        ),
        job(
            "Peak to -0.1 dB",
            None,
            lvl(&|l| {
                l.normalize = true;
                l.normalize_db = -0.1;
            }),
        ),
        job(
            "Peak to -3 dB",
            None,
            lvl(&|l| {
                l.normalize = true;
                l.normalize_db = -3.0;
            }),
        ),
        job("Fade in, 2 s", None, lvl(&|l| l.fade_in_ms = 2000.0)),
        job("Fade out, 2 s", None, lvl(&|l| l.fade_out_ms = 2000.0)),
        preset("Hard limit to -0.1 dB", "Hard Limit to -0.1 dB"),
        preset("Voice-over compressor", "Voice-Over Compressor"),
        preset("De-ess voice", "De-Ess Voice"),
        preset("Reduce noise", "Noise, Medium"),
        preset("Reduce hiss", "Reduce Hiss"),
        preset("Reduce room echo", "Reduce Room Echo"),
        preset("Remove 60 Hz hum", "Hum Cut 60 Hz (US)"),
        preset("Remove vocals", "Remove Vocals"),
        preset("Fix DC offset", "Fix DC Offset"),
        preset("Telephone voice", "Telephone - Receiver"),
        preset("Raise pitch, 2 semitones", "Raise Pitch"),
        preset("Lower pitch, 2 semitones", "Lower Pitch"),
    ]
}
