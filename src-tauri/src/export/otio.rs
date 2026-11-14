// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! OpenTimelineIO JSON for a session: one audio track per session track, clips with their media
//! file, gaps where a track is silent, markers on the timeline. Times are in audio samples
//! (rate = the session rate), so nothing is rounded to frames. Fades, volume, pan and effects
//! are not carried; OTIO has no standard form for them.

use serde_json::{json, Value};

use crate::export::interchange::{InClip, InSession};

fn time(value: u64, rate: u32) -> Value {
    json!({ "OTIO_SCHEMA": "RationalTime.1", "rate": f64::from(rate), "value": value as f64 })
}

fn range(start: u64, duration: u64, rate: u32) -> Value {
    json!({ "OTIO_SCHEMA": "TimeRange.1", "start_time": time(start, rate), "duration": time(duration, rate) })
}

/// `file:///C:/a/b.wav` style URL for an absolute path, the path unchanged when it is relative.
pub fn target_url(path: &str) -> String {
    let p = path.replace('\\', "/");
    let encoded: String = p
        .chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '#' => "%23".to_string(),
            '%' => "%25".to_string(),
            '?' => "%3F".to_string(),
            c => c.to_string(),
        })
        .collect();
    let absolute = encoded.starts_with('/') || encoded.as_bytes().get(1) == Some(&b':');
    if !absolute {
        encoded
    } else if encoded.starts_with('/') {
        format!("file://{encoded}")
    } else {
        format!("file:///{encoded}")
    }
}

fn clip_value(c: &InClip, rate: u32) -> Value {
    let available = c
        .media_length
        .map(|len| range(0, len, rate))
        .unwrap_or(Value::Null);
    json!({
        "OTIO_SCHEMA": "Clip.2",
        "name": c.name,
        "source_range": range(c.source_start, c.length, rate),
        "media_references": {
            "DEFAULT_MEDIA": {
                "OTIO_SCHEMA": "ExternalReference.1",
                "name": c.name,
                "target_url": target_url(&c.path),
                "available_range": available,
                "metadata": {}
            }
        },
        "active_media_reference_key": "DEFAULT_MEDIA",
        "enabled": true,
        "effects": [],
        "markers": [],
        "metadata": {}
    })
}

/// The OTIO document as a JSON value.
pub fn otio(session: &InSession) -> Value {
    let rate = session.sample_rate;
    let mut tracks_in_use: Vec<usize> = session.clips.iter().map(|c| c.track).collect();
    tracks_in_use.sort_unstable();
    tracks_in_use.dedup();
    let mut tracks = Vec::new();
    for t in tracks_in_use {
        let mut clips: Vec<&InClip> = session
            .clips
            .iter()
            .filter(|c| c.track == t && c.length > 0)
            .collect();
        clips.sort_by_key(|c| c.start);
        let mut children = Vec::new();
        let mut cursor = 0u64;
        for c in clips {
            if c.start > cursor {
                children.push(json!({
                    "OTIO_SCHEMA": "Gap.1",
                    "name": "",
                    "source_range": range(0, c.start - cursor, rate),
                    "effects": [], "markers": [], "metadata": {}
                }));
                cursor = c.start;
            }
            // An overlap cannot be written on one OTIO track: the later clip starts where the earlier ends.
            let skip = cursor.saturating_sub(c.start);
            if skip >= c.length {
                continue;
            }
            let mut c2 = c.clone();
            c2.source_start += skip;
            c2.length -= skip;
            children.push(clip_value(&c2, rate));
            cursor = c.start + c.length;
        }
        tracks.push(json!({
            "OTIO_SCHEMA": "Track.1",
            "name": session.track_names.get(t).cloned().unwrap_or_else(|| format!("Track {}", t + 1)),
            "kind": "Audio",
            "children": children,
            "source_range": Value::Null,
            "effects": [], "markers": [], "metadata": {}
        }));
    }
    let markers: Vec<Value> = session
        .markers
        .iter()
        .map(|m| {
            json!({
                "OTIO_SCHEMA": "Marker.2",
                "name": m.name,
                "marked_range": range(m.start, m.length, rate),
                "color": "GREEN",
                "comment": "",
                "metadata": {}
            })
        })
        .collect();
    json!({
        "OTIO_SCHEMA": "Timeline.1",
        "name": session.title,
        "global_start_time": time(0, rate),
        "tracks": {
            "OTIO_SCHEMA": "Stack.1",
            "name": "tracks",
            "children": tracks,
            "source_range": Value::Null,
            "effects": [],
            "markers": markers,
            "metadata": {}
        },
        "metadata": {}
    })
}

pub fn otio_string(session: &InSession) -> String {
    serde_json::to_string_pretty(&otio(session)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::interchange::InMarker;

    fn clip(track: usize, name: &str, start: u64, src: u64, len: u64) -> InClip {
        InClip {
            track,
            name: name.into(),
            path: format!("C:\\Show Audio\\{name}.wav"),
            start,
            source_start: src,
            length: len,
            media_length: Some(480_000),
            stretch: 1.0,
        }
    }

    fn session() -> InSession {
        InSession {
            title: "Ep 1".into(),
            sample_rate: 48_000,
            track_names: vec!["Host".into(), "Guest".into()],
            clips: vec![
                clip(0, "a", 48_000, 0, 48_000),
                clip(0, "b", 96_000, 12_000, 48_000),
                clip(1, "g", 0, 0, 24_000),
            ],
            markers: vec![InMarker {
                name: "Intro".into(),
                start: 1_000,
                length: 0,
            }],
        }
    }

    #[test]
    fn gaps_fill_the_space_before_clips() {
        let v = otio(&session());
        let tracks = v["tracks"]["children"].as_array().unwrap();
        assert_eq!(tracks.len(), 2);
        let host = tracks[0]["children"].as_array().unwrap();
        assert_eq!(host[0]["OTIO_SCHEMA"], "Gap.1");
        assert_eq!(host[0]["source_range"]["duration"]["value"], 48_000.0);
        assert_eq!(host[1]["OTIO_SCHEMA"], "Clip.2");
        assert_eq!(host[2]["source_range"]["start_time"]["value"], 12_000.0);
        assert_eq!(tracks[1]["kind"], "Audio");
    }

    #[test]
    fn overlaps_are_trimmed_not_written() {
        let mut s = session();
        s.clips = vec![clip(0, "a", 0, 0, 96_000), clip(0, "b", 48_000, 0, 96_000)];
        let v = otio(&s);
        let kids = v["tracks"]["children"][0]["children"].as_array().unwrap();
        assert_eq!(kids.len(), 2);
        // b starts where a ends, 48,000 samples into itself.
        assert_eq!(kids[1]["source_range"]["start_time"]["value"], 48_000.0);
        assert_eq!(kids[1]["source_range"]["duration"]["value"], 48_000.0);
    }

    #[test]
    fn paths_become_urls() {
        assert_eq!(
            target_url("C:\\Show Audio\\a#1.wav"),
            "file:///C:/Show%20Audio/a%231.wav"
        );
        assert_eq!(target_url("/home/steve/a.wav"), "file:///home/steve/a.wav");
        assert_eq!(target_url("media/a.wav"), "media/a.wav");
    }

    #[test]
    fn markers_ride_on_the_timeline() {
        let v = otio(&session());
        let m = &v["tracks"]["markers"][0];
        assert_eq!(m["name"], "Intro");
        assert_eq!(m["marked_range"]["start_time"]["value"], 1_000.0);
    }

    #[test]
    fn the_text_is_valid_json() {
        let text = otio_string(&session());
        let back: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(back["OTIO_SCHEMA"], "Timeline.1");
    }
}
