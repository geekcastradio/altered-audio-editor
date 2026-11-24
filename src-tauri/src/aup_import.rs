// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Reads `.aup3` and `.aup4` project files into a session.
//!
//! Both are SQLite databases. The `project` table holds one row: a name dictionary and the
//! project document, a compact binary form of an XML tree. The `sampleblocks` table holds the
//! audio in blocks of raw samples. We read the document into a small tree, then write each clip's
//! audio out to a WAV file in a folder next to the project and place clips that point at those
//! files, the same way the other project importers place clips. The file is only read, never
//! changed.
//!
//! The binary form, as found in the sample files: a dictionary section is a list of name entries
//! (tag 15, a 16-bit id, a byte length, UTF-16 text). The document is a list of items: start tag
//! (1) and end tag (2) carry a name id; attributes carry a name id and a value (string 3, int 4,
//! bool 5, long 6, long long 7, size 8, float 9, double 10); text (11) and raw bytes (12) carry a
//! length; a binary attribute (16) carries a name id and a length. Strings are UTF-16 little endian.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use thiserror::Error;

use crate::session::model::{
    new_id, ChannelLayout, Clip, Id, Marker, MarkerKind, MediaRef, Session, Track,
};

#[derive(Debug, Error)]
pub enum AupError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("This is not a readable project file: {0}")]
    Database(String),
    #[error("The project document could not be read: {0}")]
    Format(String),
    #[error("The audio could not be written: {0}")]
    Write(String),
}

impl From<rusqlite::Error> for AupError {
    fn from(e: rusqlite::Error) -> Self {
        AupError::Database(e.to_string())
    }
}

/// What came across, for the notice shown after import.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AupReport {
    pub tracks: usize,
    pub clips: usize,
    pub markers: usize,
    /// Things the project had that are not brought in.
    pub left_out: Vec<String>,
    /// Sample blocks the file did not hold. Their audio is silence.
    pub missing_blocks: usize,
    /// The folder the audio was written to.
    pub audio_folder: PathBuf,
}

#[derive(Debug)]
pub struct AupImported {
    pub session: Session,
    pub report: AupReport,
    pub missing: Vec<Id>,
}

// ---- the binary document ----

/// One element of the project document.
#[derive(Debug, Default, Clone)]
pub struct Node {
    pub name: String,
    pub attrs: HashMap<String, Value>,
    pub kids: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Int(i64),
    Bool(bool),
    Float(f64),
    Bytes(usize),
}

impl Node {
    fn f64(&self, key: &str) -> Option<f64> {
        match self.attrs.get(key)? {
            Value::Float(v) => Some(*v),
            Value::Int(v) => Some(*v as f64),
            Value::Str(s) => s.trim().parse().ok(),
            _ => None,
        }
    }
    fn i64(&self, key: &str) -> Option<i64> {
        match self.attrs.get(key)? {
            Value::Int(v) => Some(*v),
            Value::Float(v) => Some(*v as i64),
            Value::Bool(b) => Some(i64::from(*b)),
            Value::Str(s) => s.trim().parse().ok(),
            _ => None,
        }
    }
    fn bool(&self, key: &str) -> bool {
        match self.attrs.get(key) {
            Some(Value::Bool(b)) => *b,
            Some(Value::Int(v)) => *v != 0,
            _ => false,
        }
    }
    fn str(&self, key: &str) -> Option<&str> {
        match self.attrs.get(key)? {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    fn children<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> {
        self.kids.iter().filter(move |k| k.name == name)
    }
}

struct Reader<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], AupError> {
        let end = self
            .i
            .checked_add(n)
            .filter(|e| *e <= self.b.len())
            .ok_or_else(|| AupError::Format("the document ends early".into()))?;
        let s = &self.b[self.i..end];
        self.i = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, AupError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, AupError> {
        let s = self.take(2)?;
        Ok(u16::from_le_bytes([s[0], s[1]]))
    }
    fn u32(&mut self) -> Result<u32, AupError> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn i32(&mut self) -> Result<i32, AupError> {
        Ok(self.u32()? as i32)
    }
    fn i64(&mut self) -> Result<i64, AupError> {
        let s = self.take(8)?;
        Ok(i64::from_le_bytes(s.try_into().expect("8 bytes")))
    }
    fn f32(&mut self) -> Result<f32, AupError> {
        Ok(f32::from_bits(self.u32()?))
    }
    fn f64(&mut self) -> Result<f64, AupError> {
        let s = self.take(8)?;
        Ok(f64::from_le_bytes(s.try_into().expect("8 bytes")))
    }
}

fn utf16(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&units)
}

/// Reads the name dictionary: id to name.
pub fn read_names(dict: &[u8]) -> Result<HashMap<u16, String>, AupError> {
    let mut r = Reader { b: dict, i: 0 };
    let mut names = HashMap::new();
    // A leading size marker: tag 0 and the width of a character in bytes.
    if dict.first() == Some(&0) {
        r.take(2)?;
    }
    while r.i < dict.len() {
        match r.u8()? {
            15 => {
                let id = r.u16()?;
                let len = usize::from(r.u16()?);
                names.insert(id, utf16(r.take(len)?));
            }
            t => return Err(AupError::Format(format!("unknown dictionary tag {t}"))),
        }
    }
    Ok(names)
}

/// Reads the project document into a tree. Returns its root element.
pub fn read_document(doc: &[u8], names: &HashMap<u16, String>) -> Result<Node, AupError> {
    let name_of = |id: u16| -> Result<String, AupError> {
        names.get(&id).cloned().ok_or_else(|| {
            AupError::Format(format!("a name is missing from the dictionary ({id})"))
        })
    };
    let mut r = Reader { b: doc, i: 0 };
    // `stack` holds the open elements, innermost last.
    let mut stack: Vec<Node> = Vec::new();
    let mut root: Option<Node> = None;
    while r.i < doc.len() {
        let tag = r.u8()?;
        match tag {
            0 => {
                r.take(1)?;
            }
            1 => {
                let n = Node {
                    name: name_of(r.u16()?)?,
                    ..Node::default()
                };
                stack.push(n);
            }
            2 => {
                r.u16()?;
                let done = stack
                    .pop()
                    .ok_or_else(|| AupError::Format("an element ends that never began".into()))?;
                match stack.last_mut() {
                    Some(parent) => parent.kids.push(done),
                    None => root = Some(done),
                }
            }
            3..=10 | 16 => {
                let key = name_of(r.u16()?)?;
                let value = match tag {
                    3 => {
                        let len = r.u32()? as usize;
                        Value::Str(utf16(r.take(len)?))
                    }
                    4 | 6 => Value::Int(i64::from(r.i32()?)),
                    5 => Value::Bool(r.u8()? != 0),
                    7 => Value::Int(r.i64()?),
                    8 => Value::Int(i64::from(r.u32()?)),
                    9 => {
                        let v = f64::from(r.f32()?);
                        r.i32()?;
                        Value::Float(v)
                    }
                    10 => {
                        let v = r.f64()?;
                        r.i32()?;
                        Value::Float(v)
                    }
                    _ => {
                        let len = r.u32()? as usize;
                        r.take(len)?;
                        Value::Bytes(len)
                    }
                };
                let cur = stack
                    .last_mut()
                    .ok_or_else(|| AupError::Format("an attribute with no element".into()))?;
                cur.attrs.insert(key, value);
            }
            11 | 12 => {
                let len = r.u32()? as usize;
                r.take(len)?;
            }
            13 | 14 => {}
            t => return Err(AupError::Format(format!("unknown tag {t}"))),
        }
    }
    root.ok_or_else(|| AupError::Format("the document is empty".into()))
}

// ---- samples ----

/// How the file stores a sample, from the format number the project writes.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SampleFormat {
    Int16,
    Float32,
}

fn sample_format(code: i64) -> Option<SampleFormat> {
    match code {
        0x0002_0001 => Some(SampleFormat::Int16),
        0x0004_000F => Some(SampleFormat::Float32),
        _ => None,
    }
}

fn decode_block(blob: &[u8], fmt: SampleFormat, count: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(count);
    match fmt {
        SampleFormat::Float32 => {
            for c in blob.chunks_exact(4).take(count) {
                out.push(f32::from_le_bytes([c[0], c[1], c[2], c[3]]));
            }
        }
        SampleFormat::Int16 => {
            for c in blob.chunks_exact(2).take(count) {
                out.push(f32::from(i16::from_le_bytes([c[0], c[1]])) / 32768.0);
            }
        }
    }
    out.resize(count, 0.0);
    out
}

/// The audio of one sequence element, as float samples. `missing` counts blocks not in the file.
fn read_sequence(
    db: &Connection,
    seq: &Node,
    fmt: SampleFormat,
    missing: &mut usize,
) -> Result<Vec<f32>, AupError> {
    let total = seq.i64("numsamples").unwrap_or(0).max(0) as usize;
    let mut blocks: Vec<(usize, Option<usize>, i64)> = seq
        .children("waveblock")
        .filter_map(|b| {
            Some((
                b.i64("start")?.max(0) as usize,
                b.i64("length").map(|l| l.max(0) as usize),
                b.i64("blockid")?,
            ))
        })
        .collect();
    blocks.sort_by_key(|b| b.0);
    let mut out = vec![0.0f32; total];
    let mut stmt = db.prepare("SELECT samples FROM sampleblocks WHERE blockid = ?1")?;
    for (n, (start, length, id)) in blocks.iter().enumerate() {
        if *start >= total {
            continue;
        }
        let next = blocks.get(n + 1).map_or(total, |b| b.0.min(total));
        let count = length.unwrap_or(next - start).min(total - start);
        let blob: Option<Vec<u8>> = stmt.query_row([id], |row| row.get(0)).ok();
        match blob {
            Some(blob) => {
                let samples = decode_block(&blob, fmt, count);
                out[*start..*start + count].copy_from_slice(&samples);
            }
            None => *missing += 1,
        }
    }
    Ok(out)
}

fn write_wav(path: &Path, rate: u32, channels: &[Vec<f32>]) -> Result<(), AupError> {
    let spec = hound::WavSpec {
        channels: channels.len() as u16,
        sample_rate: rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec).map_err(|e| AupError::Write(e.to_string()))?;
    let frames = channels.iter().map(Vec::len).min().unwrap_or(0);
    for i in 0..frames {
        for ch in channels {
            w.write_sample(ch[i])
                .map_err(|e| AupError::Write(e.to_string()))?;
        }
    }
    w.finalize().map_err(|e| AupError::Write(e.to_string()))
}

// ---- building the session ----

fn db_of(gain: f64) -> f64 {
    if gain.is_finite() && gain > 0.0 {
        (20.0 * gain.log10()).clamp(-96.0, 24.0)
    } else {
        -96.0
    }
}

fn seconds_to_samples(seconds: f64, rate: u32) -> u64 {
    if seconds.is_finite() && seconds > 0.0 {
        (seconds * f64::from(rate)).round() as u64
    } else {
        0
    }
}

fn file_safe(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let s = s.trim().to_string();
    if s.is_empty() {
        "Track".into()
    } else {
        s
    }
}

/// Reads the current project document from an open database.
fn project_root(db: &Connection) -> Result<Node, AupError> {
    let (dict, doc): (Vec<u8>, Vec<u8>) = db
        .query_row("SELECT dict, doc FROM project WHERE id = 1", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .map_err(|_| AupError::Database("it has no project in it".into()))?;
    let names = read_names(&dict)?;
    read_document(&doc, &names)
}

/// Turns the project into a session, writing audio under `folder`.
pub fn build(
    db: &Connection,
    root: &Node,
    name: &str,
    folder: &Path,
    base: &Path,
) -> Result<AupImported, AupError> {
    let rate = root
        .f64("rate")
        .map_or(44_100, |r| r.round() as u32)
        .clamp(8_000, 384_000);
    let mut session = Session::new(name, rate);
    let mut report = AupReport {
        audio_folder: folder.to_path_buf(),
        ..AupReport::default()
    };
    let mut left_out: Vec<String> = Vec::new();
    let mut missing_ids: Vec<Id> = Vec::new();
    let mut made_folder = false;

    // A stereo pair is two wave tracks, the first marked linked.
    let wave: Vec<&Node> = root.children("wavetrack").collect();
    let mut i = 0;
    let mut clip_counter = 0usize;
    while i < wave.len() {
        let left = wave[i];
        let linked = left.bool("linked") && i + 1 < wave.len();
        let right = if linked { Some(wave[i + 1]) } else { None };
        i += if linked { 2 } else { 1 };

        let track_name = left.str("name").unwrap_or("").trim().to_string();
        let track_name = if track_name.is_empty() {
            format!("Track {}", session.tracks.len() + 1)
        } else {
            track_name
        };
        let layout = if right.is_some() {
            ChannelLayout::Stereo
        } else {
            ChannelLayout::Mono
        };
        let mut track = Track::new(track_name.clone(), layout);
        track.volume_db = db_of(left.f64("gain").unwrap_or(1.0));
        track.pan = left.f64("pan").unwrap_or(0.0).clamp(-1.0, 1.0);
        track.muted = left.bool("mute");
        track.solo = left.bool("solo");
        let file_rate = left
            .f64("rate")
            .map_or(rate, |r| r.round() as u32)
            .clamp(8_000, 384_000);

        let left_clips: Vec<&Node> = left.children("waveclip").collect();
        let right_clips: Vec<&Node> = right
            .map(|r| r.children("waveclip").collect())
            .unwrap_or_default();
        let paired = right.is_some() && left_clips.len() == right_clips.len();
        if right.is_some() && !paired {
            left_out.push("a stereo track whose two sides differ (kept as the left side)".into());
        }
        if left
            .children("effects")
            .next()
            .is_some_and(|e| e.bool("active"))
        {
            left_out.push("track effects".into());
        }

        for (ci, clip) in left_clips.iter().enumerate() {
            let Some(seq) = clip.children("sequence").next() else {
                continue;
            };
            let Some(fmt) = seq.i64("sampleformat").and_then(sample_format) else {
                left_out.push("audio stored in a sample format that is not supported".into());
                continue;
            };
            let mut audio = vec![read_sequence(db, seq, fmt, &mut report.missing_blocks)?];
            if paired {
                if let Some(seq_r) = right_clips[ci].children("sequence").next() {
                    audio.push(read_sequence(db, seq_r, fmt, &mut report.missing_blocks)?);
                }
            }
            let length = audio.iter().map(Vec::len).min().unwrap_or(0);
            if length == 0 {
                continue;
            }
            if !made_folder {
                std::fs::create_dir_all(folder)?;
                made_folder = true;
            }
            clip_counter += 1;
            let file_name = format!("{:03} {}.wav", clip_counter, file_safe(&track_name));
            let wav = folder.join(&file_name);
            write_wav(&wav, file_rate, &audio)?;

            let relative = wav.strip_prefix(base).map(Path::to_path_buf).ok();
            let media = MediaRef {
                id: new_id(),
                path: wav,
                relative_path: relative,
                sample_rate: file_rate,
                channels: audio.len() as u16,
                length: length as u64,
            };
            let media_id = media.id;
            session.media.push(media);

            let trim_left = seconds_to_samples(clip.f64("trimLeft").unwrap_or(0.0), file_rate);
            let trim_right = seconds_to_samples(clip.f64("trimRight").unwrap_or(0.0), file_rate);
            let source_in = trim_left.min(length as u64);
            let source_out = (length as u64).saturating_sub(trim_right);
            if source_out <= source_in {
                continue;
            }
            let ratio = clip.f64("clipStretchRatio").unwrap_or(1.0);
            if (ratio - 1.0).abs() > 1e-6 {
                left_out.push("clip time stretch (the clip plays unstretched)".into());
            }
            if clip
                .children("envelope")
                .any(|e| e.i64("numpoints").unwrap_or(0) > 0)
            {
                left_out.push("clip volume envelopes".into());
            }
            let clip_name = clip
                .str("name")
                .filter(|n| !n.trim().is_empty())
                .unwrap_or(&track_name)
                .to_string();
            track.clips.push(Clip {
                extra: Default::default(),
                id: new_id(),
                name: clip_name,
                media_id,
                start: seconds_to_samples(clip.f64("offset").unwrap_or(0.0), rate),
                source_in,
                source_out,
                gain_db: 0.0,
                muted: false,
                locked: false,
                color: None,
                fade_in: Default::default(),
                fade_out: Default::default(),
                envelopes: Vec::new(),
                effects: Default::default(),
                group: None,
                looped: None,
                stretch: None,
                audio_type: None,
                channel_mix: None,
                reversed: false,
                inverted: false,
                take: None,
            });
            report.clips += 1;
        }
        session.tracks.push(track);
        report.tracks += 1;
    }

    // Label tracks become markers: each label has a start `t`, an end `t1` and a title.
    for lt in root.children("labeltrack") {
        for l in lt.children("label") {
            let start = seconds_to_samples(l.f64("t").unwrap_or(0.0), rate);
            let end = seconds_to_samples(l.f64("t1").unwrap_or(0.0), rate).max(start);
            session.markers.push(Marker {
                id: new_id(),
                name: l.str("title").unwrap_or("").to_string(),
                kind: MarkerKind::Cue,
                position: start,
                duration: end - start,
                description: String::new(),
                chapter: None,
                contracted_duration: None,
                color: None,
            });
        }
    }
    session.markers.sort_by_key(|m| m.position);
    report.markers = session.markers.len();

    for other in ["notetrack", "timetrack"] {
        if root.children(other).next().is_some() {
            left_out.push(format!("{other}s"));
        }
    }
    left_out.sort();
    left_out.dedup();
    report.left_out = left_out;
    missing_ids.sort();
    Ok(AupImported {
        session,
        report,
        missing: missing_ids,
    })
}

/// Reads an `.aup3` or `.aup4` file and builds a session. The audio goes to a folder named after
/// the project, beside it.
pub fn import(path: &Path) -> Result<AupImported, AupError> {
    let db = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let root = project_root(&db)?;
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Imported Project".into());
    let base = path.parent().unwrap_or(Path::new("")).to_path_buf();
    let folder = base.join(format!("{stem} Imported Audio"));
    build(&db, &root, &stem, &folder, &base)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A small writer for the binary form, so the tests build their own project files.
    struct Enc {
        names: Vec<String>,
        doc: Vec<u8>,
    }

    impl Enc {
        fn new() -> Self {
            Enc {
                names: Vec::new(),
                doc: Vec::new(),
            }
        }
        fn id(&mut self, n: &str) -> u16 {
            if let Some(p) = self.names.iter().position(|x| x == n) {
                return p as u16;
            }
            self.names.push(n.to_string());
            (self.names.len() - 1) as u16
        }
        fn start(&mut self, n: &str) {
            let id = self.id(n);
            self.doc.push(1);
            self.doc.extend(id.to_le_bytes());
        }
        fn end(&mut self, n: &str) {
            let id = self.id(n);
            self.doc.push(2);
            self.doc.extend(id.to_le_bytes());
        }
        fn s(&mut self, k: &str, v: &str) {
            let id = self.id(k);
            let u: Vec<u8> = v.encode_utf16().flat_map(u16::to_le_bytes).collect();
            self.doc.push(3);
            self.doc.extend(id.to_le_bytes());
            self.doc.extend((u.len() as u32).to_le_bytes());
            self.doc.extend(u);
        }
        fn int(&mut self, k: &str, v: i32) {
            let id = self.id(k);
            self.doc.push(4);
            self.doc.extend(id.to_le_bytes());
            self.doc.extend(v.to_le_bytes());
        }
        fn long(&mut self, k: &str, v: i64) {
            let id = self.id(k);
            self.doc.push(7);
            self.doc.extend(id.to_le_bytes());
            self.doc.extend(v.to_le_bytes());
        }
        fn size(&mut self, k: &str, v: u32) {
            let id = self.id(k);
            self.doc.push(8);
            self.doc.extend(id.to_le_bytes());
            self.doc.extend(v.to_le_bytes());
        }
        fn boolean(&mut self, k: &str, v: bool) {
            let id = self.id(k);
            self.doc.push(5);
            self.doc.extend(id.to_le_bytes());
            self.doc.push(u8::from(v));
        }
        fn dbl(&mut self, k: &str, v: f64) {
            let id = self.id(k);
            self.doc.push(10);
            self.doc.extend(id.to_le_bytes());
            self.doc.extend(v.to_le_bytes());
            self.doc.extend(8i32.to_le_bytes());
        }
        fn dict(&self) -> Vec<u8> {
            let mut d = vec![0u8, 2];
            for (i, n) in self.names.iter().enumerate() {
                let u: Vec<u8> = n.encode_utf16().flat_map(u16::to_le_bytes).collect();
                d.push(15);
                d.extend((i as u16).to_le_bytes());
                d.extend((u.len() as u16).to_le_bytes());
                d.extend(u);
            }
            d
        }
    }

    const FLOAT: i64 = 0x0004_000F;

    fn float_blob(v: &[f32]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }

    /// Two linked mono sides (a stereo track) and a mono track, each one clip, plus a label.
    fn make_project(path: &Path) {
        let mut e = Enc::new();
        e.start("project");
        e.dbl("rate", 48_000.0);
        let mut block = 1;
        let mut blocks: Vec<(i64, Vec<f32>)> = Vec::new();
        let mut track =
            |e: &mut Enc, name: &str, linked: bool, channel: i32, gain: f64, data: Vec<f32>| {
                e.start("wavetrack");
                e.s("name", name);
                e.int("channel", channel);
                e.boolean("linked", linked);
                e.boolean("mute", false);
                e.boolean("solo", false);
                e.int("rate", 44_100);
                e.dbl("gain", gain);
                e.dbl("pan", 0.0);
                e.start("waveclip");
                e.dbl("offset", 2.0);
                e.start("sequence");
                e.size("maxsamples", 4);
                e.int("sampleformat", FLOAT as i32);
                e.long("numsamples", data.len() as i64);
                // Two blocks, split at 3 samples.
                for (start, part) in [(0usize, &data[..3]), (3usize, &data[3..])] {
                    e.start("waveblock");
                    e.long("start", start as i64);
                    e.long("blockid", block);
                    e.end("waveblock");
                    blocks.push((block, part.to_vec()));
                    block += 1;
                }
                e.end("sequence");
                e.end("waveclip");
                e.end("wavetrack");
            };
        track(
            &mut e,
            "Host",
            true,
            0,
            0.5,
            vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
        );
        track(
            &mut e,
            "Host",
            false,
            1,
            0.5,
            vec![-0.1, -0.2, -0.3, -0.4, -0.5, -0.6],
        );
        track(&mut e, "Music", false, 2, 1.0, vec![0.9, 0.8, 0.7, 0.6]);
        e.start("labeltrack");
        e.start("label");
        e.dbl("t", 1.5);
        e.dbl("t1", 2.5);
        e.s("title", "Ad break");
        e.end("label");
        e.end("labeltrack");
        e.end("project");

        let db = Connection::open(path).unwrap();
        db.execute_batch(
            "CREATE TABLE project (id INTEGER PRIMARY KEY, dict BLOB, doc BLOB);
             CREATE TABLE sampleblocks (blockid INTEGER PRIMARY KEY, sampleformat INTEGER, samples BLOB);",
        )
        .unwrap();
        db.execute(
            "INSERT INTO project (id, dict, doc) VALUES (1, ?1, ?2)",
            rusqlite::params![e.dict(), e.doc],
        )
        .unwrap();
        for (id, data) in blocks {
            db.execute(
                "INSERT INTO sampleblocks (blockid, sampleformat, samples) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, FLOAT, float_blob(&data)],
            )
            .unwrap();
        }
    }

    #[test]
    fn names_and_document_round_trip() {
        let mut e = Enc::new();
        e.start("a");
        e.s("x", "héllo");
        e.int("n", -7);
        e.boolean("b", true);
        e.dbl("d", 1.25);
        e.start("kid");
        e.end("kid");
        e.end("a");
        let names = read_names(&e.dict()).unwrap();
        let root = read_document(&e.doc, &names).unwrap();
        assert_eq!(root.name, "a");
        assert_eq!(root.str("x"), Some("héllo"));
        assert_eq!(root.i64("n"), Some(-7));
        assert!(root.bool("b"));
        assert_eq!(root.f64("d"), Some(1.25));
        assert_eq!(root.kids.len(), 1);
    }

    #[test]
    fn a_cut_off_document_is_an_error_not_a_panic() {
        let mut e = Enc::new();
        e.start("a");
        e.s("x", "text");
        let names = read_names(&e.dict()).unwrap();
        assert!(read_document(&e.doc[..e.doc.len() - 3], &names).is_err());
        assert!(read_document(&[99], &names).is_err());
    }

    #[test]
    fn project_becomes_tracks_clips_and_markers() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Show.aup3");
        make_project(&file);
        let r = import(&file).unwrap();
        assert_eq!(r.report.tracks, 2);
        assert_eq!(r.report.clips, 2);
        assert_eq!(r.report.markers, 1);
        assert_eq!(r.report.missing_blocks, 0);
        let host = &r.session.tracks[0];
        assert_eq!(host.name, "Host");
        assert_eq!(host.clips.len(), 1);
        // 2 s at the project rate of 48 kHz.
        assert_eq!(host.clips[0].start, 96_000);
        assert!((host.volume_db + 6.0206).abs() < 0.01);
        assert_eq!(r.session.tracks[1].name, "Music");
        assert_eq!(r.session.markers[0].name, "Ad break");
        assert_eq!(r.session.markers[0].position, 72_000);
        assert_eq!(r.session.markers[0].duration, 48_000);
    }

    #[test]
    fn audio_is_written_next_to_the_project_with_both_sides() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Show.aup3");
        make_project(&file);
        let r = import(&file).unwrap();
        assert_eq!(
            r.report.audio_folder,
            dir.path().join("Show Imported Audio")
        );
        let m = &r.session.media[0];
        assert_eq!(m.channels, 2);
        assert_eq!(m.sample_rate, 44_100);
        assert_eq!(m.length, 6);
        let mut w = hound::WavReader::open(&m.path).unwrap();
        let s: Vec<f32> = w.samples::<f32>().map(Result::unwrap).collect();
        // Interleaved left and right.
        assert_eq!(&s[..4], &[0.1, -0.1, 0.2, -0.2]);
        assert_eq!(r.session.media[1].channels, 1);
        assert!(m.relative_path.is_some());
    }

    #[test]
    fn missing_blocks_are_silence_and_counted() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("Gap.aup3");
        make_project(&file);
        {
            let db = Connection::open(&file).unwrap();
            db.execute("DELETE FROM sampleblocks WHERE blockid = 2", [])
                .unwrap();
        }
        let r = import(&file).unwrap();
        assert_eq!(r.report.missing_blocks, 1);
        let mut w = hound::WavReader::open(&r.session.media[0].path).unwrap();
        let s: Vec<f32> = w.samples::<f32>().map(Result::unwrap).collect();
        // Frames 3 to 5 of the left side are silent.
        assert_eq!(s[6], 0.0);
        assert_eq!(s[0], 0.1);
    }

    #[test]
    fn a_file_that_is_not_a_project_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("x.aup3");
        std::fs::write(&file, b"not a database at all").unwrap();
        assert!(import(&file).is_err());
    }
}

#[cfg(test)]
mod sample_files {
    /// Runs against real projects when AUP_SAMPLES names a folder holding them. Skipped otherwise.
    #[test]
    fn real_projects_import_when_available() {
        let Ok(dir) = std::env::var("AUP_SAMPLES") else {
            return;
        };
        for ext in ["aup3", "aup4"] {
            let tmp = tempfile::tempdir().unwrap();
            let src = std::path::Path::new(&dir).join(format!("a.{ext}"));
            if !src.exists() {
                continue;
            }
            let dst = tmp.path().join(format!("a.{ext}"));
            std::fs::copy(&src, &dst).unwrap();
            let r = super::import(&dst).unwrap();
            eprintln!("{ext}: {:?} media={}", r.report, r.session.media.len());
            assert!(r.report.clips >= 1);
            assert_eq!(r.report.missing_blocks, 0);
        }
    }
}
