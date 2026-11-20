// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Builds a session from a parsed `.rpp` project (`.rpp`).
//!
//! The parser in `rpp` reads the project; this turns it into tracks, clips and markers. Item
//! positions come in seconds and become samples at the project rate. File names are tried next to
//! the project first, then as stored. A clip whose file cannot be found is still placed, so the
//! edit survives and the file can be relinked.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

use crate::codecs::probe;
use crate::rpp;
use crate::session::model::{
    new_id, ChannelLayout, Clip, ClipStretch, Fade, Id, Marker, MarkerKind, MediaRef, Session,
    Track, STRETCH_MAX, STRETCH_MIN,
};

#[derive(Debug, Error)]
pub enum RppError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Parse(String),
}

/// What came across, for the notice shown after import.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RppReport {
    pub tracks: usize,
    pub clips: usize,
    pub markers: usize,
    /// Clips whose speed was changed in the project.
    pub stretched: usize,
    /// Things the project had that are not brought in (plug-ins, sends, envelopes and so on).
    pub left_out: Vec<String>,
    /// Media files that could not be found. Their clips are still placed.
    pub missing_files: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct RppImported {
    pub session: Session,
    pub report: RppReport,
    pub missing: Vec<Id>,
}

fn db(gain: f64) -> f64 {
    if gain.is_finite() && gain > 0.0 {
        (20.0 * gain.log10()).clamp(-96.0, 24.0)
    } else {
        -96.0
    }
}

fn samples(seconds: f64, rate: u32) -> u64 {
    if seconds.is_finite() && seconds > 0.0 {
        (seconds * f64::from(rate)).round() as u64
    } else {
        0
    }
}

pub(crate) fn candidates(file: &str, base: &Path) -> Vec<PathBuf> {
    let unified = file.replace('\\', "/");
    let mut out = Vec::new();
    let p = PathBuf::from(&unified);
    if p.is_absolute() || unified.chars().nth(1) == Some(':') {
        out.push(p);
        // A path from another computer: its file name next to the project.
        if let Some(name) = unified.rsplit('/').next() {
            out.push(base.join(name));
            out.push(base.join("Media").join(name));
        }
    } else {
        out.push(base.join(&unified));
        if let Some(name) = unified.rsplit('/').next() {
            out.push(base.join(name));
        }
    }
    out
}

pub(crate) fn media_for(
    file: &str,
    base: &Path,
    rate: u32,
    channels: u16,
    min_len: u64,
) -> (MediaRef, bool) {
    for p in candidates(file, base) {
        if p.is_file() {
            if let Ok(info) = probe(&p) {
                return (
                    MediaRef {
                        id: new_id(),
                        path: p,
                        relative_path: None,
                        sample_rate: info.sample_rate,
                        channels: info.channels,
                        length: info.length,
                    },
                    true,
                );
            }
        }
    }
    (
        MediaRef {
            id: new_id(),
            path: PathBuf::from(file.replace('\\', "/")),
            relative_path: None,
            sample_rate: rate,
            channels,
            length: min_len,
        },
        false,
    )
}

/// Turns a parsed project into a session. `base` is the folder the project sits in.
pub fn build(project: &rpp::Project, name: &str, base: &Path) -> RppImported {
    let rate = project.sample_rate.unwrap_or(48_000).clamp(8_000, 384_000);
    let mut session = Session::new(name, rate);
    let mut report = RppReport {
        left_out: project.warnings.clone(),
        ..RppReport::default()
    };
    let mut media_ids: HashMap<String, Id> = HashMap::new();
    let mut missing: Vec<Id> = Vec::new();

    for t in &project.tracks {
        let mut track = Track::new(
            if t.name.trim().is_empty() {
                format!("Track {}", session.tracks.len() + 1)
            } else {
                t.name.clone()
            },
            ChannelLayout::Stereo,
        );
        track.volume_db = db(t.volume);
        track.pan = t.pan.clamp(-1.0, 1.0);
        track.muted = t.mute;
        track.solo = t.solo;
        for it in &t.items {
            if it.file.is_empty() || it.length <= 0.0 {
                continue;
            }
            let rate_factor = if it.play_rate.is_finite() && it.play_rate > 0.0 {
                it.play_rate
            } else {
                1.0
            };
            let media_id = match media_ids.get(&it.file) {
                Some(id) => *id,
                None => {
                    // The source length needed to cover this item, until the real file says.
                    let need = samples(it.source_offset + it.length * rate_factor, rate);
                    let (m, found) = media_for(&it.file, base, rate, 2, need);
                    if !found {
                        missing.push(m.id);
                        report.missing_files.push(m.path.clone());
                    }
                    let id = m.id;
                    session.media.push(m);
                    media_ids.insert(it.file.clone(), id);
                    id
                }
            };
            let (m_rate, m_len) = session
                .media
                .iter()
                .find(|m| m.id == media_id)
                .map_or((rate, u64::MAX), |m| (m.sample_rate, m.length));
            let source_in = samples(it.source_offset, m_rate);
            let mut source_out = source_in + samples(it.length * rate_factor, m_rate).max(1);
            if m_len != u64::MAX && !missing.contains(&media_id) {
                source_out = source_out.min(m_len);
            } else if missing.contains(&media_id) {
                if let Some(m) = session.media.iter_mut().find(|m| m.id == media_id) {
                    m.length = m.length.max(source_out);
                }
            }
            if source_out <= source_in {
                continue;
            }
            let stretch = if (rate_factor - 1.0).abs() > 1e-6 {
                report.stretched += 1;
                Some(ClipStretch {
                    ratio: (1.0 / rate_factor).clamp(STRETCH_MIN, STRETCH_MAX),
                    ..ClipStretch::default()
                })
            } else {
                None
            };
            let fade_of = |secs: f64| Fade {
                length: samples(secs, rate),
                ..Fade::default()
            };
            track.clips.push(Clip {
                id: new_id(),
                name: if it.name.is_empty() {
                    "Clip".into()
                } else {
                    it.name.clone()
                },
                media_id,
                start: samples(it.position, rate),
                source_in,
                source_out,
                gain_db: db(it.volume),
                muted: it.mute,
                locked: false,
                color: None,
                fade_in: fade_of(it.fade_in),
                fade_out: fade_of(it.fade_out),
                envelopes: Vec::new(),
                effects: Default::default(),
                group: None,
                looped: None,
                stretch,
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

    for m in &project.markers {
        let start = samples(m.position, rate);
        let end = m.end.map_or(start, |e| samples(e, rate));
        session.markers.push(Marker {
            id: new_id(),
            name: m.name.clone(),
            kind: MarkerKind::Cue,
            position: start,
            duration: end.saturating_sub(start),
            description: String::new(),
            chapter: None,
            contracted_duration: None,
            color: None,
        });
    }
    session.markers.sort_by_key(|m| m.position);
    report.markers = session.markers.len();

    RppImported {
        session,
        report,
        missing,
    }
}

/// Reads a `.rpp` file and builds a session.
pub fn import(path: &Path) -> Result<RppImported, RppError> {
    let bytes = std::fs::read(path)?;
    let text = String::from_utf8_lossy(&bytes);
    let project = rpp::parse(&text).map_err(|e| RppError::Parse(e.to_string()))?;
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Imported Project".into());
    Ok(build(
        &project,
        &name,
        path.parent().unwrap_or(Path::new("")),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(file: &str, pos: f64, len: f64, off: f64, rate: f64) -> rpp::Item {
        rpp::Item {
            name: "Intro".into(),
            file: file.into(),
            position: pos,
            length: len,
            source_offset: off,
            play_rate: rate,
            volume: 1.0,
            fade_in: 0.5,
            fade_out: 0.0,
            mute: false,
        }
    }

    fn project() -> rpp::Project {
        rpp::Project {
            sample_rate: Some(48_000),
            tracks: vec![rpp::Track {
                name: "Host".into(),
                volume: 0.5,
                pan: -0.25,
                mute: false,
                solo: true,
                items: vec![
                    item("C:\\Shows\\gone.wav", 2.0, 10.0, 1.0, 1.0),
                    item("C:\\Shows\\gone.wav", 12.0, 5.0, 0.0, 1.25),
                ],
            }],
            markers: vec![
                rpp::Marker {
                    name: "Ad".into(),
                    position: 30.0,
                    end: Some(60.0),
                },
                rpp::Marker {
                    name: "Start".into(),
                    position: 1.0,
                    end: None,
                },
            ],
            warnings: vec!["FX chains".into()],
        }
    }

    #[test]
    fn seconds_become_samples_at_the_project_rate() {
        let r = build(&project(), "Ep", Path::new("Z:/nowhere"));
        let c = &r.session.tracks[0].clips;
        assert_eq!(c[0].start, 96_000);
        assert_eq!(c[0].source_in, 48_000);
        assert_eq!(c[0].source_out, 48_000 + 480_000);
        assert_eq!(c[0].fade_in.length, 24_000);
    }

    #[test]
    fn mixer_values_and_stretch_carry_over() {
        let r = build(&project(), "Ep", Path::new("Z:/nowhere"));
        let t = &r.session.tracks[0];
        assert!((t.volume_db + 6.0206).abs() < 0.01);
        assert!(t.solo);
        let s = t.clips[1].stretch.expect("stretched");
        assert!((s.ratio - 0.8).abs() < 1e-9);
        assert_eq!(r.report.stretched, 1);
        // Plays 5 s at 1.25x, so it reads 6.25 s of the file.
        assert_eq!(t.clips[1].source_out - t.clips[1].source_in, 300_000);
    }

    #[test]
    fn missing_files_are_placed_once_and_reported() {
        let r = build(&project(), "Ep", Path::new("Z:/nowhere"));
        assert_eq!(r.session.media.len(), 1);
        assert_eq!(r.missing.len(), 1);
        assert_eq!(r.report.missing_files.len(), 1);
        assert!(r.session.media[0].length >= 480_000 + 48_000);
    }

    #[test]
    fn markers_and_regions_are_sorted_with_lengths() {
        let r = build(&project(), "Ep", Path::new("Z:/nowhere"));
        assert_eq!(r.session.markers[0].name, "Start");
        assert_eq!(r.session.markers[0].duration, 0);
        assert_eq!(r.session.markers[1].duration, 30 * 48_000);
        assert_eq!(r.report.left_out, vec!["FX chains".to_string()]);
    }

    #[test]
    fn relative_and_foreign_paths_are_tried_next_to_the_project() {
        let c = candidates("Media\\a.wav", Path::new("/p"));
        assert_eq!(c[0], PathBuf::from("/p/Media/a.wav"));
        let c = candidates("C:\\Old\\b.wav", Path::new("/p"));
        assert!(c.contains(&PathBuf::from("/p/b.wav")));
    }
}
