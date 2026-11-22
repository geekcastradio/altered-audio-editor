// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Marker rules: kinds and colors, new marker names, picking and searching
//! markers, and the chapter export lists and files. The Rust copy of
//! `src/panels/markers/markers.ts` and `chapters.ts`; both are tested against
//! `shared-fixtures/markers.json`.

use crate::js_compat::{is_js_space, js_number_to_string, js_round, js_trim};
use crate::session::model::MarkerKind;
use serde::Deserialize;
use std::collections::HashSet;

/// A chapter marker's extra data: only the link matters here.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct ChapterLink {
    pub url: Option<String>,
}

/// A marker as these rules see it. Ids are plain text so the rules do not
/// care how the session names them; `From<&session::model::Marker>` fills it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Mark {
    pub id: String,
    pub name: String,
    pub kind: MarkerKind,
    pub position: u64,
    pub duration: u64,
    pub description: String,
    pub chapter: Option<ChapterLink>,
    pub contracted_duration: Option<u64>,
    #[serde(default)]
    pub color: Option<String>,
}

impl From<&crate::session::model::Marker> for Mark {
    fn from(m: &crate::session::model::Marker) -> Self {
        Mark {
            id: m.id.to_string(),
            name: m.name.clone(),
            kind: m.kind,
            position: m.position,
            duration: m.duration,
            description: m.description.clone(),
            chapter: m
                .chapter
                .as_ref()
                .map(|c| ChapterLink { url: c.url.clone() }),
            contracted_duration: m.contracted_duration,
            color: m.color.clone(),
        }
    }
}

pub fn kind_label(k: MarkerKind) -> &'static str {
    match k {
        MarkerKind::Cue => "Cue",
        MarkerKind::Subclip => "Subclip",
        MarkerKind::Chapter => "Chapter",
        MarkerKind::AdSlot => "Ad Slot",
        MarkerKind::SocialClip => "Social Clip",
        MarkerKind::CdTrack => "CD Track",
        MarkerKind::CartTimer => "Cart Timer",
    }
}

/// Design token for each kind's color.
pub fn kind_color(k: MarkerKind) -> &'static str {
    match k {
        MarkerKind::Cue => "--clip-5",
        MarkerKind::Subclip => "--clip-1",
        MarkerKind::Chapter => "--clip-3",
        MarkerKind::AdSlot => "--clip-4",
        MarkerKind::SocialClip => "--clip-6",
        MarkerKind::CdTrack => "--clip-2",
        MarkerKind::CartTimer => "--clip-7",
    }
}

/// A marker's color for CSS: its own, or its type's.
pub fn marker_css(kind: MarkerKind, color: Option<&str>) -> String {
    match color {
        Some(c) if !c.is_empty() => c.to_string(),
        _ => format!("var({})", kind_color(kind)),
    }
}

fn whole(x: f64) -> u64 {
    js_round(x).max(0.0) as u64
}

pub fn new_marker(id: &str, name: &str, position: f64, duration: f64, kind: MarkerKind) -> Mark {
    Mark {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        position: whole(position),
        duration: whole(duration),
        description: String::new(),
        chapter: None,
        contracted_duration: None,
        color: None,
    }
}

/// "Marker 3" style name that is not taken yet.
pub fn next_marker_name(markers: &[Mark], base: &str) -> String {
    let used: HashSet<&str> = markers.iter().map(|m| m.name.as_str()).collect();
    let mut n = markers.len() + 1;
    loop {
        let name = format!("{base} {n}");
        if !used.contains(name.as_str()) {
            return name;
        }
        n += 1;
    }
}

/// The markers whose ids are in `ids`, in time order (same spot keeps list order).
pub fn picked_markers(markers: &[Mark], ids: &HashSet<String>) -> Vec<Mark> {
    let mut v: Vec<Mark> = markers
        .iter()
        .filter(|m| ids.contains(&m.id))
        .cloned()
        .collect();
    v.sort_by_key(|m| m.position);
    v
}

/// Ids of every marker from `a` to `b` in time order, both ends included.
/// Empty when either is missing.
pub fn markers_between(markers: &[Mark], a: &str, b: &str) -> Vec<String> {
    let mut ordered: Vec<&Mark> = markers.iter().collect();
    ordered.sort_by_key(|m| m.position);
    let (Some(i), Some(j)) = (
        ordered.iter().position(|m| m.id == a),
        ordered.iter().position(|m| m.id == b),
    ) else {
        return Vec::new();
    };
    ordered[i.min(j)..=i.max(j)]
        .iter()
        .map(|m| m.id.clone())
        .collect()
}

/// The undo label for deleting `n` markers.
pub fn delete_label(n: usize) -> String {
    if n == 1 {
        "Delete Marker".to_string()
    } else {
        format!("Delete {n} Markers")
    }
}

/// Where a marker ends, in samples: its start for a point marker.
pub fn marker_end(m: &Mark) -> u64 {
    m.position + m.duration
}

/// The markers matching the search text: name, description or type, any case.
pub fn filter_markers(markers: &[Mark], query: &str) -> Vec<Mark> {
    let q = js_trim(query).to_lowercase();
    if q.is_empty() {
        return markers.to_vec();
    }
    markers
        .iter()
        .filter(|m| {
            m.name.to_lowercase().contains(&q)
                || m.description.to_lowercase().contains(&q)
                || kind_label(m.kind).to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}

/// How long a point marker becomes when converted to a range: up to the end of
/// the time selection when one reaches past it, else five seconds.
pub fn range_length_for(position: f64, selection: Option<(f64, f64)>, rate: f64) -> f64 {
    match selection {
        Some((_, end)) if end > position => js_round(end - position),
        _ => js_round(5.0 * rate),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindChange {
    pub kind: MarkerKind,
    pub contracted_duration: Option<f64>,
    pub clear_contracted: bool,
}

/// The change that gives a marker a new type. Only ad slots carry a contracted
/// length, so it is added or dropped with the type.
pub fn kind_change(
    duration: f64,
    contracted: Option<f64>,
    kind: MarkerKind,
    rate: f64,
) -> KindChange {
    if kind == MarkerKind::AdSlot {
        let fallback = if duration != 0.0 {
            duration
        } else {
            30.0 * rate
        };
        return KindChange {
            kind,
            contracted_duration: Some(contracted.unwrap_or(fallback)),
            clear_contracted: false,
        };
    }
    KindChange {
        kind,
        contracted_duration: None,
        clear_contracted: contracted.is_some(),
    }
}

/// The playlist with the marker added at the end, or taken out when it is
/// already there. Point markers never join.
pub fn toggle_playlist(playlist: &[String], id: &str, duration: u64) -> Vec<String> {
    if playlist.iter().any(|p| p == id) {
        return playlist.iter().filter(|p| *p != id).cloned().collect();
    }
    let mut out = playlist.to_vec();
    if duration > 0 {
        out.push(id.to_string());
    }
    out
}

/// The index of the marker whose flag is under `x` (pixels), nearest first,
/// within `tolerance`. The first of equal distances wins.
pub fn marker_at(
    markers: &[Mark],
    x: f64,
    x_of: impl Fn(f64) -> f64,
    tolerance: f64,
) -> Option<usize> {
    let mut best = None;
    let mut best_d = f64::INFINITY;
    for (i, m) in markers.iter().enumerate() {
        let d = (x_of(m.position as f64) - x).abs();
        if d <= tolerance && d < best_d {
            best = Some(i);
            best_d = d;
        }
    }
    best
}

// ---- chapter export ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChapterFormat {
    Timestamps,
    Csv,
    PodcastJson,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chapter {
    pub title: String,
    /// Seconds.
    pub start: f64,
    /// Seconds. None for a point marker.
    pub end: Option<f64>,
    pub kind: String,
    pub description: String,
    pub url: Option<String>,
}

fn kind_id(k: MarkerKind) -> &'static str {
    match k {
        MarkerKind::Cue => "cue",
        MarkerKind::Subclip => "subclip",
        MarkerKind::Chapter => "chapter",
        MarkerKind::AdSlot => "ad_slot",
        MarkerKind::SocialClip => "social_clip",
        MarkerKind::CdTrack => "cd_track",
        MarkerKind::CartTimer => "cart_timer",
    }
}

fn title_of(m: &Mark) -> String {
    let t = js_trim(&m.name);
    if t.is_empty() {
        "Untitled".to_string()
    } else {
        t.to_string()
    }
}

/// A chapter marker's link, or "".
pub fn chapter_url(m: &Mark) -> String {
    m.chapter
        .as_ref()
        .and_then(|c| c.url.clone())
        .unwrap_or_default()
}

/// The markers to export, in time order: chapter markers only, or every marker.
pub fn chapters_of(markers: &[Mark], rate: f64, chapters_only: bool) -> Vec<Chapter> {
    let mut v: Vec<&Mark> = markers
        .iter()
        .filter(|m| !chapters_only || m.kind == MarkerKind::Chapter)
        .collect();
    v.sort_by_key(|m| m.position);
    v.into_iter()
        .map(|m| Chapter {
            title: title_of(m),
            start: m.position as f64 / rate,
            end: (m.duration > 0).then(|| (m.position + m.duration) as f64 / rate),
            kind: kind_id(m.kind).to_string(),
            description: m.description.clone(),
            url: m.chapter.as_ref().and_then(|c| c.url.clone()),
        })
        .collect()
}

/// 0:00, 4:05, 1:02:03.
pub fn clock(seconds: f64) -> String {
    let t = seconds.floor().max(0.0) as u64;
    let (h, m, s) = (t / 3600, (t % 3600) / 60, t % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// 00:01:02.345
fn precise(seconds: f64) -> String {
    let ms = js_round(seconds * 1000.0).max(0.0) as u64;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        ms / 3_600_000,
        (ms % 3_600_000) / 60_000,
        (ms % 60_000) / 1000,
        ms % 1000
    )
}

fn csv_cell(v: &str) -> String {
    if v.contains(['"', ',', '\r', '\n']) {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

fn json_str(s: &str) -> String {
    serde_json::to_string(s).unwrap()
}

fn thousandths(x: f64) -> String {
    js_number_to_string(js_round(x * 1000.0) / 1000.0)
}

pub fn render(list: &[Chapter], format: ChapterFormat) -> String {
    match format {
        ChapterFormat::Timestamps => {
            list.iter()
                .map(|c| format!("{} {}", clock(c.start), c.title))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        }
        ChapterFormat::Csv => {
            let mut rows = vec!["Name,Start,End,Duration,Type,Description".to_string()];
            for c in list {
                let (end, dur) = match c.end {
                    Some(e) => (precise(e), precise(e - c.start)),
                    None => (String::new(), String::new()),
                };
                let cells = [
                    c.title.clone(),
                    precise(c.start),
                    end,
                    dur,
                    c.kind.clone(),
                    c.description.clone(),
                ];
                rows.push(
                    cells
                        .iter()
                        .map(|x| csv_cell(x))
                        .collect::<Vec<_>>()
                        .join(","),
                );
            }
            rows.join("\r\n") + "\r\n"
        }
        ChapterFormat::PodcastJson => {
            let mut out = String::from("{\n  \"version\": \"1.2.0\",\n  \"chapters\": ");
            if list.is_empty() {
                out.push_str("[]");
            } else {
                let items: Vec<String> = list
                    .iter()
                    .map(|c| {
                        let mut f = vec![
                            format!("      \"startTime\": {}", thousandths(c.start)),
                            format!("      \"title\": {}", json_str(&c.title)),
                        ];
                        if let Some(e) = c.end {
                            f.push(format!("      \"endTime\": {}", thousandths(e)));
                        }
                        if let Some(u) = c.url.as_deref().filter(|u| !u.is_empty()) {
                            f.push(format!("      \"url\": {}", json_str(u)));
                        }
                        format!("    {{\n{}\n    }}", f.join(",\n"))
                    })
                    .collect();
                out.push_str(&format!("[\n{}\n  ]", items.join(",\n")));
            }
            out.push_str("\n}\n");
            out
        }
    }
}

/// Problems a site or app would reject, as plain warnings. Empty when fine.
pub fn warnings(list: &[Chapter], format: ChapterFormat) -> Vec<String> {
    let mut out = Vec::new();
    if list.is_empty() {
        return vec!["There are no markers to export.".to_string()];
    }
    if format == ChapterFormat::Timestamps {
        if list[0].start >= 1.0 {
            out.push(
                "Video chapters must start at 0:00. Add a marker at the very start.".to_string(),
            );
        }
        if list.len() < 3 {
            out.push("Video sites need at least three chapters before they show them.".to_string());
        }
        for i in 1..list.len() {
            if list[i].start - list[i - 1].start < 10.0 {
                out.push(format!(
                    "\"{}\" is shorter than 10 seconds; video sites ignore chapters that short.",
                    list[i - 1].title
                ));
                break;
            }
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
pub struct Embedded {
    pub title: String,
    pub start_ms: f64,
    pub end_ms: Option<f64>,
    pub url: Option<String>,
    pub kind: String,
}

fn embedded(
    markers: &[Mark],
    rate: f64,
    range: Option<(f64, f64)>,
    chapters_only: bool,
) -> Vec<Embedded> {
    let (a, b) = range.unwrap_or((0.0, f64::INFINITY));
    let mut v: Vec<&Mark> = markers
        .iter()
        .filter(|m| {
            (!chapters_only || m.kind == MarkerKind::Chapter)
                && m.position as f64 >= a
                && (m.position as f64) < b
        })
        .collect();
    v.sort_by_key(|m| m.position);
    v.into_iter()
        .map(|m| {
            let p = m.position as f64;
            Embedded {
                title: title_of(m),
                start_ms: js_round(((p - a) / rate) * 1000.0),
                end_ms: (m.duration > 0)
                    .then(|| js_round(((p + m.duration as f64).min(b) - a) / rate * 1000.0)),
                url: Some(chapter_url(m)).filter(|u| !u.is_empty()),
                kind: kind_id(m.kind).to_string(),
            }
        })
        .collect()
}

/// Chapter markers to write into an exported file, in milliseconds from the
/// start of the export. With a range, only chapters inside it go, moved so the
/// range starts at 0.
pub fn embed_list(markers: &[Mark], rate: f64, range: Option<(f64, f64)>) -> Vec<Embedded> {
    embedded(markers, rate, range, true)
}

/// Every marker, of any type, for WAV cue points or a FLAC cue sheet. Same
/// timing rules as `embed_list`.
pub fn marker_list(markers: &[Mark], rate: f64, range: Option<(f64, f64)>) -> Vec<Embedded> {
    embedded(markers, rate, range, false)
}

/// A link a chapter can carry: http or https, no spaces.
pub fn is_chapter_url(text: &str) -> bool {
    let t = js_trim(text);
    let Some(head) = t.get(..7) else { return false };
    let rest = if head.eq_ignore_ascii_case("http://") {
        &t[7..]
    } else if t
        .get(..8)
        .is_some_and(|h| h.eq_ignore_ascii_case("https://"))
    {
        &t[8..]
    } else {
        return false;
    };
    !rest.is_empty() && !rest.chars().any(is_js_space)
}

/// Plain tips for a chapter title: 45 characters or fewer, no more than five
/// words, no repeated words. Empty when the title is fine.
pub fn title_tips(title: &str) -> Vec<String> {
    let t = js_trim(title);
    if t.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let words: Vec<&str> = t.split(is_js_space).filter(|w| !w.is_empty()).collect();
    let len = t.encode_utf16().count();
    if len > 45 {
        out.push(format!("{len} characters; apps show about 45."));
    }
    if words.len() > 5 {
        out.push(format!("{} words; five or fewer reads best.", words.len()));
    }
    let mut seen = HashSet::new();
    for w in words
        .iter()
        .map(|x| {
            x.to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|x| x.encode_utf16().count() > 3)
    {
        if seen.contains(&w) {
            out.push(format!("\"{w}\" appears twice."));
            break;
        }
        seen.insert(w);
    }
    out
}
