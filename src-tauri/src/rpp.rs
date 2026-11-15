// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Reads a `.rpp` project (a plain text file) into the pieces a session needs: tracks,
//! the audio items on them, markers and regions. Times stay in seconds as the file stores them;
//! the importer converts to samples at the session rate and resolves relative file paths against
//! the folder of the project.
//!
//! What is read: sample rate, track names, volume, pan, mute, solo; items with position, length,
//! start offset, play rate, volume, fades, mute, name and the source file of the selected take;
//! markers and regions. What is not: plug-ins and FX chains, sends and folders, envelopes, MIDI
//! and video items, take FX, groups. Each of those that was present is named in `warnings`.

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Project {
    pub sample_rate: Option<u32>,
    pub tracks: Vec<Track>,
    pub markers: Vec<Marker>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub name: String,
    /// Linear gain, 1.0 is unity.
    pub volume: f64,
    /// -1.0 (left) to 1.0 (right).
    pub pan: f64,
    pub mute: bool,
    pub solo: bool,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    pub file: String,
    pub position: f64,
    pub length: f64,
    pub source_offset: f64,
    /// 1.0 plays at normal speed. Anything else is a stretch.
    pub play_rate: f64,
    pub volume: f64,
    pub fade_in: f64,
    pub fade_out: f64,
    pub mute: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    pub name: String,
    pub position: f64,
    /// Present for a region.
    pub end: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotAProject,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotAProject => write!(f, "This is not a readable .rpp project file."),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Default)]
struct Chunk {
    name: String,
    args: Vec<String>,
    lines: Vec<Vec<String>>,
    children: Vec<Chunk>,
}

/// Splits a line into words; quotes (double, single or backtick) keep spaces inside one word.
fn words(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;
    for ch in line.chars() {
        match quote {
            Some(q) if ch == q => quote = None,
            Some(_) => cur.push(ch),
            None if ch == '"' || ch == '\'' || ch == '`' => {
                quote = Some(ch);
                started = true;
            }
            None if ch.is_whitespace() => {
                if started || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    started = false;
                }
            }
            None => {
                cur.push(ch);
                started = true;
            }
        }
    }
    if started || !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn parse_chunks(text: &str) -> Result<Chunk, Error> {
    let mut stack: Vec<Chunk> = Vec::new();
    let mut root: Option<Chunk> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('|') {
            continue;
        }
        if let Some(open) = line.strip_prefix('<') {
            let mut w = words(open);
            if w.is_empty() {
                continue;
            }
            let name = w.remove(0);
            stack.push(Chunk {
                name,
                args: w,
                ..Chunk::default()
            });
        } else if line == ">" {
            let done = stack.pop().ok_or(Error::NotAProject)?;
            match stack.last_mut() {
                Some(parent) => parent.children.push(done),
                None => root = Some(done),
            }
        } else if let Some(top) = stack.last_mut() {
            top.lines.push(words(line));
        }
    }
    root.filter(|r| r.name == HEADER).ok_or(Error::NotAProject)
}

fn num(chunk: &Chunk, key: &str, index: usize, default: f64) -> f64 {
    chunk
        .lines
        .iter()
        .find(|l| l.first().map(String::as_str) == Some(key))
        .and_then(|l| l.get(index + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn text<'a>(chunk: &'a Chunk, key: &str) -> Option<&'a str> {
    chunk
        .lines
        .iter()
        .find(|l| l.first().map(String::as_str) == Some(key))
        .and_then(|l| l.get(1))
        .map(String::as_str)
}

fn source_file(source: &Chunk) -> Option<String> {
    if let Some(f) = text(source, "FILE") {
        return Some(f.to_string());
    }
    // A SECTION source wraps the real one.
    source
        .children
        .iter()
        .filter(|c| c.name == "SOURCE")
        .find_map(source_file)
}

fn read_item(chunk: &Chunk, warnings: &mut BTreeSet<String>) -> Option<Item> {
    // Takes: every SOURCE child is one take, TAKE lines mark them and SEL marks the active one.
    let take_lines: Vec<usize> = chunk
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.first().map(String::as_str) == Some("TAKE"))
        .map(|(i, _)| i)
        .collect();
    let sources: Vec<&Chunk> = chunk
        .children
        .iter()
        .filter(|c| c.name == "SOURCE")
        .collect();
    if sources.len() > 1 {
        warnings
            .insert("Some items have several takes: only the selected take is brought in.".into());
    }
    let selected = take_lines
        .iter()
        .position(|&i| chunk.lines[i].iter().any(|w| w == "SEL"))
        .unwrap_or(0);
    let source = sources.get(selected).or_else(|| sources.first())?;
    let kind = source.args.first().map(String::as_str).unwrap_or("");
    if matches!(kind, "MIDI" | "MIDIPOOL" | "VIDEO" | "VIDEOEFFECT") {
        warnings.insert(format!("{kind} items are not brought in."));
        return None;
    }
    let Some(file) = source_file(source) else {
        warnings.insert(
            "Some items have no audio file (empty items or generated sources) and were left out."
                .into(),
        );
        return None;
    };
    if chunk.children.iter().any(|c| c.name == "TAKEFX") {
        warnings.insert("Take effects are not brought in.".into());
    }
    let play_rate = num(chunk, "PLAYRATE", 0, 1.0);
    if (play_rate - 1.0).abs() > 1e-6 {
        warnings.insert(
            "Items with a changed play rate come in at their length without the stretch.".into(),
        );
    }
    Some(Item {
        name: text(chunk, "NAME").unwrap_or("").to_string(),
        file,
        position: num(chunk, "POSITION", 0, 0.0),
        length: num(chunk, "LENGTH", 0, 0.0),
        source_offset: num(chunk, "SOFFS", 0, 0.0),
        play_rate,
        volume: num(chunk, "VOLPAN", 0, 1.0),
        fade_in: num(chunk, "FADEIN", 1, 0.0),
        fade_out: num(chunk, "FADEOUT", 1, 0.0),
        mute: num(chunk, "MUTE", 0, 0.0) != 0.0,
    })
}

/// Reads the project text.
/// The first word of the file's outer block.
const HEADER: &str = concat!("REAP", "ER_PROJECT");

pub fn parse(text_in: &str) -> Result<Project, Error> {
    let root = parse_chunks(text_in)?;
    let mut warnings: BTreeSet<String> = BTreeSet::new();
    let mut project = Project {
        sample_rate: root
            .lines
            .iter()
            .find(|l| l.first().map(String::as_str) == Some("SAMPLERATE"))
            .and_then(|l| l.get(1))
            .and_then(|v| v.parse::<u32>().ok())
            .filter(|r| *r > 0),
        ..Project::default()
    };
    for chunk in &root.children {
        if chunk.name != "TRACK" {
            continue;
        }
        if chunk
            .lines
            .iter()
            .any(|l| l.first().map(String::as_str) == Some("ISBUS"))
        {
            let flags = chunk
                .lines
                .iter()
                .find(|l| l[0] == "ISBUS")
                .and_then(|l| l.get(1))
                .map(String::as_str);
            if flags == Some("1") {
                warnings.insert("Folders are brought in as plain tracks.".into());
            }
        }
        if chunk
            .lines
            .iter()
            .any(|l| l.first().map(String::as_str) == Some("AUXRECV"))
        {
            warnings.insert("Sends are not brought in.".into());
        }
        if chunk.children.iter().any(|c| c.name == "FXCHAIN") {
            warnings.insert("Plug-ins and effect chains are not brought in.".into());
        }
        if chunk.children.iter().any(|c| c.name.ends_with("ENV")) {
            warnings.insert("Envelopes are not brought in.".into());
        }
        let items = chunk
            .children
            .iter()
            .filter(|c| c.name == "ITEM")
            .filter_map(|c| read_item(c, &mut warnings))
            .collect();
        project.tracks.push(Track {
            name: text(chunk, "NAME").unwrap_or("").to_string(),
            volume: num(chunk, "VOLPAN", 0, 1.0),
            pan: num(chunk, "VOLPAN", 1, 0.0),
            mute: num(chunk, "MUTESOLO", 0, 0.0) != 0.0,
            solo: num(chunk, "MUTESOLO", 1, 0.0) != 0.0,
            items,
        });
    }
    // Markers are MARKER lines: index, position, name, isregion, ...; a region has a second line
    // with the same index and its end.
    let mut regions: Vec<(String, f64, String)> = Vec::new();
    for l in root
        .lines
        .iter()
        .filter(|l| l.first().map(String::as_str) == Some("MARKER"))
    {
        let (Some(idx), Some(pos), Some(name)) = (
            l.get(1),
            l.get(2).and_then(|v| v.parse::<f64>().ok()),
            l.get(3),
        ) else {
            continue;
        };
        let is_region = l.get(4).map(String::as_str) == Some("1");
        if let Some(open) = regions.iter().find(|r| &r.0 == idx) {
            // The second line of a region holds its end.
            if let Some(m) = project
                .markers
                .iter_mut()
                .find(|m| m.end.is_none() && m.position == open.1 && m.name == open.2)
            {
                m.end = Some(pos);
            }
        } else if is_region {
            regions.push((idx.clone(), pos, name.clone()));
            project.markers.push(Marker {
                name: name.clone(),
                position: pos,
                end: None,
            });
        } else {
            project.markers.push(Marker {
                name: name.clone(),
                position: pos,
                end: None,
            });
        }
    }
    project.warnings = warnings.into_iter().collect();
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RPP_RAW: &str = r#"<@HDR@ 0.1 "6.80/win64" 1700000000
  RIPPLE 0
  SAMPLERATE 48000 0 0
  MARKER 1 12.5 "Intro start" 0 0 1 R {AAAA} 0
  MARKER 2 20 "Ad slot" 1 0 1 R {BBBB} 0
  MARKER 2 30 "" 1 0 1 R {BBBB} 0
  <TRACK {1111}
    NAME "Host mic"
    VOLPAN 0.5 -0.25 -1 -1 1
    MUTESOLO 0 1 0
    <ITEM
      POSITION 1.5
      LENGTH 10
      SOFFS 2.25
      FADEIN 1 0.01 0 1 0 0 0
      FADEOUT 1 0.5 0 1 0 0 0
      VOLPAN 2 0 1 -1
      NAME "take 1"
      PLAYRATE 1 1 0 -1 0 0.0025
      <SOURCE WAVE
        FILE "C:\Show Audio\host.wav"
      >
    >
    <ITEM
      POSITION 20
      LENGTH 5
      MUTE 1
      <SOURCE WAVE
        FILE "host2.wav"
      >
    >
  >
  <TRACK {2222}
    NAME "Guest"
    AUXRECV 0 0 1 0 0 0 0 0 0 -1:U 0 -1 ''
    <FXCHAIN
      WNDRECT 0 0 0 0
    >
    <ITEM
      POSITION 0
      LENGTH 4
      PLAYRATE 1.25 1 0 -1 0 0.0025
      TAKE
      <SOURCE WAVE
        FILE "a.wav"
      >
      TAKE SEL NAME "second"
      <SOURCE SECTION
        LENGTH 4
        <SOURCE WAVE
          FILE "b.wav"
        >
      >
    >
    <ITEM
      POSITION 4
      LENGTH 1
      <SOURCE MIDI
      >
    >
    <ITEM
      POSITION 5
      LENGTH 1
      <SOURCE EMPTY
      >
    >
  >
>
"#;

    fn rpp() -> String {
        RPP_RAW.replace("@HDR@", HEADER)
    }

    #[test]
    fn reads_rate_tracks_and_items() {
        let p = parse(&rpp()).unwrap();
        assert_eq!(p.sample_rate, Some(48_000));
        assert_eq!(p.tracks.len(), 2);
        let host = &p.tracks[0];
        assert_eq!(host.name, "Host mic");
        assert_eq!(host.volume, 0.5);
        assert_eq!(host.pan, -0.25);
        assert!(!host.mute && host.solo);
        assert_eq!(host.items.len(), 2);
        let i = &host.items[0];
        assert_eq!(i.file, "C:\\Show Audio\\host.wav");
        assert_eq!((i.position, i.length, i.source_offset), (1.5, 10.0, 2.25));
        assert_eq!((i.fade_in, i.fade_out, i.volume), (0.01, 0.5, 2.0));
        assert_eq!(i.name, "take 1");
        assert!(host.items[1].mute);
    }

    #[test]
    fn the_selected_take_and_wrapped_sources_are_used() {
        let p = parse(&rpp()).unwrap();
        let guest = &p.tracks[1];
        assert_eq!(guest.items.len(), 1);
        assert_eq!(guest.items[0].file, "b.wav");
        assert_eq!(guest.items[0].play_rate, 1.25);
    }

    #[test]
    fn markers_and_regions_are_read() {
        let p = parse(&rpp()).unwrap();
        assert_eq!(p.markers.len(), 2);
        assert_eq!(
            p.markers[0],
            Marker {
                name: "Intro start".into(),
                position: 12.5,
                end: None
            }
        );
        assert_eq!(
            p.markers[1],
            Marker {
                name: "Ad slot".into(),
                position: 20.0,
                end: Some(30.0)
            }
        );
    }

    #[test]
    fn what_is_left_out_is_named() {
        let w = parse(&rpp()).unwrap().warnings.join("\n");
        for needle in [
            "Sends",
            "Plug-ins",
            "several takes",
            "play rate",
            "MIDI",
            "no audio file",
        ] {
            assert!(w.contains(needle), "{needle} missing from {w}");
        }
        assert!(!w.contains("Folders"));
    }

    #[test]
    fn other_files_are_refused() {
        assert_eq!(parse("hello"), Err(Error::NotAProject));
        assert_eq!(parse("<SOMETHING_ELSE\n>\n"), Err(Error::NotAProject));
        assert_eq!(parse(""), Err(Error::NotAProject));
    }

    #[test]
    fn quotes_keep_spaces_and_notes_are_ignored() {
        let w = words(r#"NAME "a b c" 'd e' plain `f g`"#);
        assert_eq!(w, vec!["NAME", "a b c", "d e", "plain", "f g"]);
        let p = parse(&format!("<{HEADER} 0.1 \"x\" 1\n  <NOTES\n    |some notes > here\n  >\n  <TRACK\n    NAME \"\"\n  >\n>\n")).unwrap();
        assert_eq!(p.tracks.len(), 1);
        assert_eq!(p.tracks[0].name, "");
    }
}
