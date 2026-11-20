// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Builds a session from an OpenTimelineIO file (`.otio`), the JSON edit-list format other editors
//! write. Written from the public format description, not from any other program's code.
//!
//! Each audio track becomes a track; clips keep their place, their length and where in the media
//! they start; gaps keep the space between them; markers on the timeline and on tracks become
//! markers. Fades, volume, pan, effects, transitions and speed changes are not brought in, because
//! the format does not hold them in a form every editor agrees on; the report names what was left
//! out. A clip whose file cannot be found is still placed, so the edit survives and the file can
//! be relinked.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::rpp_import::media_for;
use crate::session::model::{
    new_id, ChannelLayout, Clip, Fade, Id, Marker, MarkerKind, Session, Track,
};

#[derive(Debug, Error)]
pub enum OtioError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parse(String),
}

/// What came across, for the notice shown after import.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtioReport {
    pub tracks: usize,
    pub clips: usize,
    pub markers: usize,
    /// Things the file had that are not brought in.
    pub left_out: Vec<String>,
    /// Media files that could not be found. Their clips are still placed.
    pub missing_files: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct OtioImported {
    pub session: Session,
    pub report: OtioReport,
    pub missing: Vec<Id>,
}

/// A time range in seconds, with the rate its times were written in.
struct Span {
    start: f64,
    dur: f64,
    rate: f64,
}

/// The schema name without its version: `Clip.2` is `Clip`.
fn kind(v: &Value) -> &str {
    v.get("OTIO_SCHEMA")
        .and_then(Value::as_str)
        .and_then(|s| s.split('.').next())
        .unwrap_or("")
}

/// A `RationalTime` as (seconds, rate).
fn rational(v: &Value) -> Option<(f64, f64)> {
    let value = v.get("value")?.as_f64()?;
    let rate = v.get("rate")?.as_f64()?;
    if rate.is_finite() && rate > 0.0 && value.is_finite() {
        Some((value / rate, rate))
    } else {
        None
    }
}

/// A `TimeRange`, or None when the value is missing or null.
fn span(v: Option<&Value>) -> Option<Span> {
    let v = v?;
    let (start, _) = rational(v.get("start_time")?)?;
    let (dur, rate) = rational(v.get("duration")?)?;
    Some(Span { start, dur, rate })
}

fn samples(seconds: f64, rate: u32) -> u64 {
    if seconds.is_finite() && seconds > 0.0 {
        (seconds * f64::from(rate)).round() as u64
    } else {
        0
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            let hex = std::str::from_utf8(&b[i + 1..i + 3]).ok();
            if let Some(v) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The file path a `target_url` names. `file:///C:/a/b.wav` gives `C:/a/b.wav`,
/// `file:///home/a.wav` gives `/home/a.wav`, and a plain path is returned as written.
fn url_to_path(url: &str) -> String {
    let rest = url
        .strip_prefix("file://localhost")
        .or_else(|| url.strip_prefix("file://"))
        .unwrap_or(url);
    let decoded = percent_decode(rest);
    let b = decoded.as_bytes();
    // `/C:/...` is a Windows drive path.
    if b.len() > 2 && b[0] == b'/' && b[2] == b':' && b[1].is_ascii_alphabetic() {
        decoded[1..].to_string()
    } else {
        decoded
    }
}

/// The media reference a clip plays: the active one, else any, else the older single field.
fn media_ref(clip: &Value) -> Option<&Value> {
    if let Some(map) = clip.get("media_references").and_then(Value::as_object) {
        let key = clip
            .get("active_media_reference_key")
            .and_then(Value::as_str)
            .unwrap_or("DEFAULT_MEDIA");
        if let Some(r) = map.get(key).or_else(|| map.values().next()) {
            return Some(r);
        }
    }
    clip.get("media_reference")
}

fn note(list: &mut Vec<String>, what: &str) {
    if !list.iter().any(|x| x == what) {
        list.push(what.to_string());
    }
}

fn array<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    match v.get(key).and_then(Value::as_array) {
        Some(a) => a.as_slice(),
        None => &[],
    }
}

/// The audio rate the file's clips were written at, when it looks like one. Video frame rates
/// (24, 25, 30) are not audio rates, so those files get 48 kHz.
fn session_rate(tracks: &[Value]) -> u32 {
    for t in tracks {
        for c in array(t, "children") {
            if kind(c) != "Clip" {
                continue;
            }
            if let Some(s) = span(c.get("source_range")) {
                if s.rate >= 8_000.0 && s.rate <= 384_000.0 {
                    return s.rate.round() as u32;
                }
            }
        }
    }
    48_000
}

fn markers_of(v: &Value, rate: u32, out: &mut Vec<Marker>) {
    for m in array(v, "markers") {
        let Some(s) = span(m.get("marked_range")) else {
            continue;
        };
        let name = m
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        out.push(Marker {
            id: new_id(),
            name,
            kind: MarkerKind::Cue,
            position: samples(s.start, rate),
            duration: samples(s.dur, rate),
            description: m
                .get("comment")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            chapter: None,
            contracted_duration: None,
            color: None,
        });
    }
}

/// Turns a parsed `.otio` timeline into a session. `base` is the folder the file sits in.
pub fn build(doc: &Value, name: &str, base: &Path) -> Result<OtioImported, OtioError> {
    if kind(doc) != "Timeline" {
        return Err(OtioError::Parse(
            "This is not an OpenTimelineIO timeline (.otio).".into(),
        ));
    }
    let stack = doc
        .get("tracks")
        .ok_or_else(|| OtioError::Parse("This timeline has no tracks.".into()))?;
    let track_nodes = array(stack, "children");
    let rate = session_rate(track_nodes).clamp(8_000, 384_000);
    let mut session = Session::new(name, rate);
    let mut report = OtioReport::default();
    let mut media_ids: HashMap<String, Id> = HashMap::new();
    let mut missing: Vec<Id> = Vec::new();
    let mut markers: Vec<Marker> = Vec::new();

    for node in track_nodes {
        if kind(node) != "Track" {
            note(&mut report.left_out, "nested sequences");
            continue;
        }
        let is_audio = node
            .get("kind")
            .and_then(Value::as_str)
            .is_some_and(|k| k.eq_ignore_ascii_case("audio"));
        if !is_audio {
            note(&mut report.left_out, "video tracks");
            continue;
        }
        let track_name = node
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let mut track = Track::new(
            if track_name.is_empty() {
                format!("Track {}", session.tracks.len() + 1)
            } else {
                track_name
            },
            ChannelLayout::Stereo,
        );
        markers_of(node, rate, &mut markers);
        let mut cursor = 0.0_f64;
        for child in array(node, "children") {
            match kind(child) {
                "Gap" => {
                    cursor += span(child.get("source_range")).map_or(0.0, |s| s.dur.max(0.0));
                }
                "Transition" => note(&mut report.left_out, "transitions"),
                "Clip" => {
                    let reference = media_ref(child);
                    let available = reference.and_then(|r| span(r.get("available_range")));
                    let range = span(child.get("source_range")).or_else(|| {
                        available.as_ref().map(|a| Span {
                            start: a.start,
                            dur: a.dur,
                            rate: a.rate,
                        })
                    });
                    let Some(range) = range else {
                        continue;
                    };
                    let length = range.dur.max(0.0);
                    let url = reference
                        .and_then(|r| r.get("target_url"))
                        .and_then(Value::as_str)
                        .filter(|u| !u.is_empty());
                    let position = cursor;
                    cursor += length;
                    let Some(url) = url else {
                        note(&mut report.left_out, "clips with no media file");
                        continue;
                    };
                    if length <= 0.0 {
                        continue;
                    }
                    if !array(child, "effects").is_empty() {
                        note(&mut report.left_out, "clip effects and speed changes");
                    }
                    if !array(child, "markers").is_empty() {
                        note(&mut report.left_out, "markers on clips");
                    }
                    let file = url_to_path(url);
                    // Times in the file count from where the media starts (often a timecode).
                    let offset =
                        (range.start - available.as_ref().map_or(0.0, |a| a.start)).max(0.0);
                    let media_id = match media_ids.get(&file) {
                        Some(id) => *id,
                        None => {
                            let need = samples(offset + length, rate);
                            let (m, found) = media_for(&file, base, rate, 2, need);
                            if !found {
                                missing.push(m.id);
                                report.missing_files.push(m.path.clone());
                            }
                            let id = m.id;
                            session.media.push(m);
                            media_ids.insert(file.clone(), id);
                            id
                        }
                    };
                    let (m_rate, m_len) = session
                        .media
                        .iter()
                        .find(|m| m.id == media_id)
                        .map_or((rate, u64::MAX), |m| (m.sample_rate, m.length));
                    let source_in = samples(offset, m_rate);
                    let mut source_out = source_in + samples(length, m_rate).max(1);
                    if missing.contains(&media_id) {
                        if let Some(m) = session.media.iter_mut().find(|m| m.id == media_id) {
                            m.length = m.length.max(source_out);
                        }
                    } else if m_len != u64::MAX {
                        source_out = source_out.min(m_len);
                    }
                    if source_out <= source_in {
                        continue;
                    }
                    let clip_name = child
                        .get("name")
                        .and_then(Value::as_str)
                        .filter(|n| !n.is_empty())
                        .map(str::to_string)
                        .or_else(|| {
                            Path::new(&file)
                                .file_stem()
                                .map(|s| s.to_string_lossy().into_owned())
                        })
                        .unwrap_or_else(|| "Clip".into());
                    track.clips.push(Clip {
                        id: new_id(),
                        name: clip_name,
                        media_id,
                        start: samples(position, rate),
                        source_in,
                        source_out,
                        gain_db: 0.0,
                        muted: child.get("enabled").and_then(Value::as_bool) == Some(false),
                        locked: false,
                        color: None,
                        fade_in: Fade::default(),
                        fade_out: Fade::default(),
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
                _ => {
                    note(&mut report.left_out, "nested sequences");
                    cursor += span(child.get("source_range")).map_or(0.0, |s| s.dur.max(0.0));
                }
            }
        }
        session.tracks.push(track);
        report.tracks += 1;
    }
    if report.tracks == 0 {
        return Err(OtioError::Parse(
            "This timeline has no audio tracks to bring in.".into(),
        ));
    }

    markers_of(stack, rate, &mut markers);
    markers.sort_by_key(|m| m.position);
    report.markers = markers.len();
    session.markers = markers;
    Ok(OtioImported {
        session,
        report,
        missing,
    })
}

/// Reads a `.otio` file and builds a session.
pub fn import(path: &Path) -> Result<OtioImported, OtioError> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8_lossy(&bytes);
    let text = text.trim_start_matches('\u{feff}');
    let doc: Value = serde_json::from_str(text)
        .map_err(|e| OtioError::Parse(format!("This is not a readable .otio file: {e}")))?;
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Imported Timeline".into());
    build(&doc, &name, path.parent().unwrap_or(Path::new("")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::interchange::{InClip, InMarker, InSession};
    use crate::export::otio::otio;
    use serde_json::json;

    fn t(value: f64, rate: f64) -> Value {
        json!({ "OTIO_SCHEMA": "RationalTime.1", "rate": rate, "value": value })
    }

    fn r(start: f64, dur: f64, rate: f64) -> Value {
        json!({ "OTIO_SCHEMA": "TimeRange.1", "start_time": t(start, rate), "duration": t(dur, rate) })
    }

    fn clip(name: &str, url: &str, start: f64, dur: f64, avail_start: f64) -> Value {
        json!({
            "OTIO_SCHEMA": "Clip.2",
            "name": name,
            "source_range": r(start, dur, 48_000.0),
            "media_references": { "DEFAULT_MEDIA": {
                "OTIO_SCHEMA": "ExternalReference.1",
                "target_url": url,
                "available_range": r(avail_start, 480_000.0, 48_000.0)
            }},
            "active_media_reference_key": "DEFAULT_MEDIA",
            "effects": [], "markers": []
        })
    }

    fn timeline(children: Vec<Value>) -> Value {
        json!({
            "OTIO_SCHEMA": "Timeline.1",
            "name": "Ep",
            "tracks": {
                "OTIO_SCHEMA": "Stack.1",
                "children": [{
                    "OTIO_SCHEMA": "Track.1", "name": "Host", "kind": "Audio",
                    "children": children, "markers": []
                }],
                "markers": [{
                    "OTIO_SCHEMA": "Marker.2", "name": "Intro",
                    "marked_range": r(48_000.0, 0.0, 48_000.0), "comment": "start here"
                }]
            }
        })
    }

    fn base() -> &'static Path {
        Path::new("Z:/nowhere")
    }

    #[test]
    fn gaps_and_offsets_become_positions_in_samples() {
        let gap = json!({ "OTIO_SCHEMA": "Gap.1", "source_range": r(0.0, 96_000.0, 48_000.0) });
        let doc = timeline(vec![
            gap,
            clip(
                "a",
                "file:///C:/Show%20Audio/a.wav",
                12_000.0,
                48_000.0,
                0.0,
            ),
        ]);
        let out = build(&doc, "Ep", base()).unwrap();
        let c = &out.session.tracks[0].clips[0];
        assert_eq!(out.session.sample_rate, 48_000);
        assert_eq!(c.start, 96_000);
        assert_eq!(c.source_in, 12_000);
        assert_eq!(c.source_out, 60_000);
        assert_eq!(
            out.session.media[0].path,
            PathBuf::from("C:/Show Audio/a.wav")
        );
        assert_eq!(out.session.tracks[0].name, "Host");
    }

    #[test]
    fn a_media_start_time_is_not_counted_as_an_offset() {
        // The media begins at one hour of timecode; the clip starts 2 s into it.
        let doc = timeline(vec![clip(
            "a",
            "file:///tmp/a.wav",
            172_800_000.0 + 96_000.0,
            48_000.0,
            172_800_000.0,
        )]);
        let out = build(&doc, "Ep", base()).unwrap();
        assert_eq!(out.session.tracks[0].clips[0].source_in, 96_000);
    }

    #[test]
    fn markers_come_from_the_timeline_and_are_sorted() {
        let doc = timeline(vec![clip("a", "file:///tmp/a.wav", 0.0, 48_000.0, 0.0)]);
        let out = build(&doc, "Ep", base()).unwrap();
        assert_eq!(out.report.markers, 1);
        assert_eq!(out.session.markers[0].name, "Intro");
        assert_eq!(out.session.markers[0].position, 48_000);
        assert_eq!(out.session.markers[0].description, "start here");
    }

    #[test]
    fn missing_files_are_placed_once_and_reported() {
        let doc = timeline(vec![
            clip("a", "file:///tmp/gone.wav", 0.0, 48_000.0, 0.0),
            clip("b", "file:///tmp/gone.wav", 48_000.0, 48_000.0, 0.0),
        ]);
        let out = build(&doc, "Ep", base()).unwrap();
        assert_eq!(out.session.media.len(), 1);
        assert_eq!(out.missing.len(), 1);
        assert_eq!(out.report.missing_files.len(), 1);
        assert_eq!(out.session.tracks[0].clips.len(), 2);
        assert!(out.session.media[0].length >= 96_000);
    }

    #[test]
    fn what_is_not_carried_is_named() {
        let mut c = clip("a", "file:///tmp/a.wav", 0.0, 48_000.0, 0.0);
        c["effects"] = json!([{ "OTIO_SCHEMA": "LinearTimeWarp.1", "time_scalar": 2.0 }]);
        c["enabled"] = json!(false);
        let transition = json!({ "OTIO_SCHEMA": "Transition.1", "name": "fade" });
        let mut doc = timeline(vec![c, transition]);
        doc["tracks"]["children"]
            .as_array_mut()
            .unwrap()
            .push(json!({ "OTIO_SCHEMA": "Track.1", "kind": "Video", "children": [] }));
        let out = build(&doc, "Ep", base()).unwrap();
        assert!(out.session.tracks[0].clips[0].muted);
        assert_eq!(out.session.tracks.len(), 1);
        for what in [
            "clip effects and speed changes",
            "transitions",
            "video tracks",
        ] {
            assert!(out.report.left_out.iter().any(|x| x == what), "{what}");
        }
    }

    #[test]
    fn other_files_and_empty_timelines_are_refused() {
        let e = build(&json!({ "OTIO_SCHEMA": "Clip.2" }), "x", base()).unwrap_err();
        assert!(e.to_string().contains("not an OpenTimelineIO"));
        let video_only = json!({
            "OTIO_SCHEMA": "Timeline.1",
            "tracks": { "OTIO_SCHEMA": "Stack.1", "children": [
                { "OTIO_SCHEMA": "Track.1", "kind": "Video", "children": [] }
            ]}
        });
        assert!(build(&video_only, "x", base()).is_err());
    }

    #[test]
    fn urls_become_paths() {
        assert_eq!(url_to_path("file:///C:/A%20B/c%231.wav"), "C:/A B/c#1.wav");
        assert_eq!(url_to_path("file:///home/s/a.wav"), "/home/s/a.wav");
        assert_eq!(url_to_path("media/a.wav"), "media/a.wav");
        assert_eq!(url_to_path("file://localhost/tmp/a.wav"), "/tmp/a.wav");
        assert_eq!(percent_decode("100%"), "100%");
    }

    #[test]
    fn what_we_export_comes_back_in_place() {
        let s = InSession {
            title: "Ep 1".into(),
            sample_rate: 44_100,
            track_names: vec!["Host".into(), "Guest".into()],
            clips: vec![
                InClip {
                    track: 0,
                    name: "a".into(),
                    path: "C:\\Show Audio\\a.wav".into(),
                    start: 44_100,
                    source_start: 0,
                    length: 44_100,
                    media_length: Some(441_000),
                    stretch: 1.0,
                },
                InClip {
                    track: 1,
                    name: "g".into(),
                    path: "C:\\Show Audio\\g.wav".into(),
                    start: 0,
                    source_start: 22_050,
                    length: 22_050,
                    media_length: Some(441_000),
                    stretch: 1.0,
                },
            ],
            markers: vec![InMarker {
                name: "Intro".into(),
                start: 1_000,
                length: 0,
            }],
        };
        let out = build(&otio(&s), "Ep 1", base()).unwrap();
        let session = &out.session;
        assert_eq!(session.sample_rate, 44_100);
        assert_eq!(session.tracks.len(), 2);
        assert_eq!(session.tracks[0].name, "Host");
        assert_eq!(session.tracks[0].clips[0].start, 44_100);
        assert_eq!(session.tracks[1].clips[0].source_in, 22_050);
        assert_eq!(session.tracks[1].clips[0].source_out, 44_100);
        assert_eq!(session.markers[0].position, 1_000);
    }
}
