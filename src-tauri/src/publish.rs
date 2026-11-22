// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Publish Package: the files that go with an episode when it is uploaded to a
//! podcast host by hand. The Rust copy of `src/app/publish.ts` (everything but
//! `slugOf`, which needs Unicode accent stripping and stays in TypeScript);
//! both are tested against `shared-fixtures/publish.json`. Nothing here
//! touches the network; it builds the text of each file.

use crate::js_compat::{is_js_space, js_number_to_string, js_round, js_trim};
use crate::markers::{self, Chapter, ChapterFormat, Mark};
use crate::session::model::MarkerKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodeType {
    Full,
    Trailer,
    Bonus,
}

impl EpisodeType {
    fn id(self) -> &'static str {
        match self {
            EpisodeType::Full => "full",
            EpisodeType::Trailer => "trailer",
            EpisodeType::Bonus => "bonus",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EpisodeInfo {
    pub title: String,
    pub description: String,
    /// None leaves it out.
    pub season: Option<f64>,
    pub episode: Option<f64>,
    pub kind: EpisodeType,
    pub explicit: bool,
}

/// Hosts cap titles at 250 characters and descriptions at 4000.
pub const TITLE_MAX: usize = 250;
pub const DESCRIPTION_MAX: usize = 4000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageFile {
    pub name: String,
    pub content: String,
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// What would stop a host from taking the episode, in plain words. Empty when
/// fine.
pub fn episode_problems(info: &EpisodeInfo) -> Vec<String> {
    let mut out = Vec::new();
    if js_trim(&info.title).is_empty() {
        out.push("The episode needs a title.".to_string());
    }
    if utf16_len(&info.title) > TITLE_MAX {
        out.push(format!(
            "The title is {} characters; hosts take up to {TITLE_MAX}.",
            utf16_len(&info.title)
        ));
    }
    if utf16_len(&info.description) > DESCRIPTION_MAX {
        out.push(format!(
            "The description is {} characters; hosts take up to {DESCRIPTION_MAX}.",
            utf16_len(&info.description)
        ));
    }
    for (label, n) in [("season", info.season), ("episode", info.episode)] {
        if let Some(n) = n {
            if !(n.is_finite() && n.fract() == 0.0) || n < 0.0 {
                out.push(format!("The {label} number must be a whole number."));
            }
        }
    }
    out
}

/// A JSON value that keeps the order its fields were added in, printed the way
/// JavaScript's `JSON.stringify(value, null, 2)` prints.
enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(&'static str, J)>),
}

fn pretty(j: &J, indent: usize) -> String {
    let pad = " ".repeat(indent + 2);
    let close = " ".repeat(indent);
    match j {
        J::Null => "null".to_string(),
        J::Bool(b) => b.to_string(),
        J::Num(n) => js_number_to_string(*n),
        J::Str(s) => serde_json::to_string(s).unwrap(),
        J::Arr(v) if v.is_empty() => "[]".to_string(),
        J::Arr(v) => format!(
            "[\n{}\n{close}]",
            v.iter()
                .map(|x| format!("{pad}{}", pretty(x, indent + 2)))
                .collect::<Vec<_>>()
                .join(",\n")
        ),
        J::Obj(v) if v.is_empty() => "{}".to_string(),
        J::Obj(v) => format!(
            "{{\n{}\n{close}}}",
            v.iter()
                .map(|(k, x)| format!(
                    "{pad}{}: {}",
                    serde_json::to_string(k).unwrap(),
                    pretty(x, indent + 2)
                ))
                .collect::<Vec<_>>()
                .join(",\n")
        ),
    }
}

/// The whole-session markers of one kind, as chapter-like entries in seconds.
fn of_kind(marks: &[Mark], rate: f64, kind: MarkerKind) -> Vec<Chapter> {
    let mut v: Vec<&Mark> = marks.iter().filter(|m| m.kind == kind).collect();
    v.sort_by_key(|m| m.position);
    v.into_iter()
        .map(|m| {
            let t = js_trim(&m.name);
            Chapter {
                title: if t.is_empty() {
                    "Untitled".to_string()
                } else {
                    t.to_string()
                },
                start: m.position as f64 / rate,
                end: (m.duration > 0).then(|| (m.position + m.duration) as f64 / rate),
                kind: String::new(),
                description: m.description.clone(),
                url: m.chapter.as_ref().and_then(|c| c.url.clone()),
            }
        })
        .collect()
}

fn ms(seconds: f64) -> f64 {
    js_round(seconds * 1000.0)
}

fn span(c: &Chapter) -> J {
    c.end
        .map_or(J::Null, |e| J::Num(js_round((e - c.start) * 1000.0)))
}

fn trim_end(s: &str) -> &str {
    s.trim_end_matches(is_js_space)
}

/// Every file of the package except the audio itself. `audio_name` is the
/// audio file's name inside the folder; `checked` says how the check of the
/// finished file went, for the checklist.
pub fn package_files(
    info: &EpisodeInfo,
    audio_name: &str,
    duration_ms: f64,
    marks: &[Mark],
    rate: f64,
    checked: &str,
) -> Vec<PackageFile> {
    let chapters = of_kind(marks, rate, MarkerKind::Chapter);
    let midrolls = of_kind(marks, rate, MarkerKind::AdSlot);
    let clips = of_kind(marks, rate, MarkerKind::SocialClip);

    let chapter_json = J::Arr(
        chapters
            .iter()
            .map(|c| {
                J::Obj(vec![
                    ("start_time_ms", J::Num(ms(c.start))),
                    ("duration_ms", span(c)),
                    ("title", J::Str(c.title.clone())),
                    ("url", c.url.clone().map_or(J::Null, J::Str)),
                ])
            })
            .collect(),
    );
    let midroll_json = || {
        J::Arr(
            midrolls
                .iter()
                .map(|c| J::Obj(vec![("start_time_ms", J::Num(ms(c.start)))]))
                .collect(),
        )
    };
    let soundbite_json = || {
        J::Arr(
            clips
                .iter()
                .map(|c| {
                    J::Obj(vec![
                        ("start_time_ms", J::Num(ms(c.start))),
                        ("duration_ms", span(c)),
                        ("title", J::Str(c.title.clone())),
                    ])
                })
                .collect(),
        )
    };

    let mut episode = vec![
        ("title", J::Str(js_trim(&info.title).to_string())),
        (
            "description",
            J::Str(js_trim(&info.description).to_string()),
        ),
        ("itunes_explicit", J::Bool(info.explicit)),
        ("itunes_episode_type", J::Str(info.kind.id().to_string())),
    ];
    if let Some(n) = info.episode {
        episode.push(("itunes_episode", J::Num(n)));
    }
    if let Some(n) = info.season {
        episode.push(("itunes_season", J::Num(n)));
    }
    episode.push(("audio_file", J::Str(audio_name.to_string())));
    episode.push(("duration_ms", J::Num(duration_ms)));
    episode.push(("chapters", chapter_json));
    episode.push(("midrolls", midroll_json()));
    episode.push(("soundbites", soundbite_json()));

    let mut notes = vec![js_trim(&info.description).to_string()];
    if !chapters.is_empty() {
        notes.push(String::new());
        notes.push("Chapters".to_string());
        notes.push(trim_end(&markers::render(&chapters, ChapterFormat::Timestamps)).to_string());
    }

    let mut files = vec![
        PackageFile {
            name: "episode.json".into(),
            content: pretty(&J::Obj(episode), 0) + "\n",
        },
        PackageFile {
            name: "show-notes.txt".into(),
            content: trim_end(&notes.join("\n")).to_string() + "\n",
        },
    ];
    if !chapters.is_empty() {
        files.push(PackageFile {
            name: "chapters.json".into(),
            content: markers::render(&chapters, ChapterFormat::PodcastJson),
        });
        files.push(PackageFile {
            name: "chapters.vtt".into(),
            content: chapters_vtt(&chapters, duration_ms),
        });
    }
    if !midrolls.is_empty() {
        files.push(PackageFile {
            name: "midrolls.json".into(),
            content: pretty(&J::Obj(vec![("midrolls", midroll_json())]), 0) + "\n",
        });
    }
    if !clips.is_empty() {
        files.push(PackageFile {
            name: "soundbites.json".into(),
            content: pretty(&J::Obj(vec![("soundbites", soundbite_json())]), 0) + "\n",
        });
    }

    let mut lines = vec![
        js_trim(&info.title).to_string(),
        String::new(),
        "Files in this folder".to_string(),
        format!("  {audio_name}  the episode{}", if chapters.is_empty() { "" } else { ", with its chapters inside the file" }),
        "  episode.json  title, description, numbers, chapters, ad positions and soundbites in one place".to_string(),
        "  show-notes.txt  the description, with a timestamp list when there are chapters".to_string(),
    ];
    if !chapters.is_empty() {
        lines.push("  chapters.json  the chapters as a Podcasting 2.0 chapters file, for hosts that take a file".to_string());
        lines.push("  chapters.vtt  the same chapters as a WebVTT file, for hosts and players that take that".to_string());
    }
    if !midrolls.is_empty() {
        lines.push("  midrolls.json  the ad positions, in milliseconds from the start".to_string());
    }
    if !clips.is_empty() {
        lines.push("  soundbites.json  the soundbite positions".to_string());
    }
    lines.push(String::new());
    lines.push("Before you upload".to_string());
    lines.push(format!("  {checked}"));
    lines.push(format!("  Length: {}.", clock_of(duration_ms)));
    lines.push(if midrolls.is_empty() {
        "  No ad positions are marked.".to_string()
    } else {
        format!(
            "  Ad positions ({}): {}",
            midrolls.len(),
            midrolls
                .iter()
                .map(|m| clock_of(ms(m.start)))
                .collect::<Vec<_>>()
                .join(", ")
        )
    });
    lines.push(String::new());
    lines.push("Upload steps are the same on every host: the audio file first, then the title, description and numbers from episode.json,".to_string());
    lines.push(
        "then the chapters and ad positions if your host asks for them separately.".to_string(),
    );
    lines.push("Made with Altered Audio Editor. Nothing was sent anywhere.".to_string());
    files.push(PackageFile {
        name: "README.txt".into(),
        content: lines.join("\n") + "\n",
    });
    files
}

/// WebVTT time: 00:01:02.345 (hours always shown).
pub fn vtt_time(ms: f64) -> String {
    let t = js_round(ms).max(0.0) as u64;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        t / 3_600_000,
        (t % 3_600_000) / 60_000,
        (t % 60_000) / 1000,
        t % 1000
    )
}

/// The chapters as a WebVTT file: one cue per chapter, each ending where the
/// next begins (the last one at the end of the episode). Titles keep their
/// text; a line break in a title becomes a space, and "-->" cannot appear.
pub fn chapters_vtt(chapters: &[Chapter], duration_ms: f64) -> String {
    let mut lines = vec!["WEBVTT".to_string(), String::new()];
    for (i, c) in chapters.iter().enumerate() {
        let start = ms(c.start);
        let end = start.max(chapters.get(i + 1).map_or(duration_ms, |n| ms(n.start)));
        let collapsed = c
            .title
            .split(is_js_space)
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        // The collapse above drops leading and trailing spaces too, as the
        // trim that follows it in the original does.
        let title = collapsed.replace("-->", "->");
        let title = js_trim(&title);
        lines.push((i + 1).to_string());
        lines.push(format!("{} --> {}", vtt_time(start), vtt_time(end)));
        lines.push(if title.is_empty() {
            "Untitled".to_string()
        } else {
            title.to_string()
        });
        lines.push(String::new());
    }
    lines.join("\n")
}

fn clock_of(total_ms: f64) -> String {
    let t = (total_ms / 1000.0).floor().max(0.0) as u64;
    let (h, m, s) = (t / 3600, (t % 3600) / 60, t % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}
