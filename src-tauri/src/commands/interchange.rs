// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! File > Export > Edit Lists: writes the open session's clips and markers as CMX3600 EDLs (one
//! per track), a marker EDL for DaVinci Resolve, or an OpenTimelineIO file, so the cut can move to
//! another editor. Fades, automation and effects do not travel in these formats.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::async_runtime::spawn_blocking;
use tauri::State;

use crate::commands::session::OpenSession;
use crate::export::edl::{self, EdlOptions};
use crate::export::interchange::{InClip, InMarker, InSession};
use crate::export::otio;
use crate::export::timecode::{timecode_to_frames, FrameRate};
use crate::session::model::{Session, TrackKind};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InterchangeKind {
    /// One CMX3600 EDL per track.
    Edl,
    /// DaVinci Resolve marker EDL.
    MarkerEdl,
    /// OpenTimelineIO JSON.
    Otio,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterchangeRequest {
    pub kind: InterchangeKind,
    /// `23.976`, `24`, `25`, `29.97`, `29.97df` or `30`.
    pub rate: String,
    /// Timecode of the session start, for example `01:00:00:00`.
    pub start: String,
    /// Resolve marker color name. Only the marker EDL uses it.
    pub color: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterchangeResult {
    /// Full paths of the files written.
    pub files: Vec<PathBuf>,
    /// Things the format could not carry.
    pub warnings: Vec<String>,
}

fn parse_rate(text: &str) -> Option<FrameRate> {
    match text {
        "23.976" => Some(FrameRate::F23_976),
        "24" => Some(FrameRate::F24),
        "25" => Some(FrameRate::F25),
        "29.97" => Some(FrameRate::F29_97Ndf),
        "29.97df" => Some(FrameRate::F29_97Df),
        "30" => Some(FrameRate::F30),
        _ => None,
    }
}

/// Plain file-name text: no separators or characters Windows refuses.
fn safe_name(text: &str) -> String {
    let s: String = text
        .chars()
        .map(|c| {
            if c.is_control() || "\\/:*?\"<>|".contains(c) {
                ' '
            } else {
                c
            }
        })
        .collect();
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.is_empty() {
        "Session".into()
    } else {
        s
    }
}

/// A path in `dir` for `name.ext` that does not exist yet: `name.ext`, `name 2.ext`, and so on.
fn free_path(dir: &Path, name: &str, ext: &str) -> PathBuf {
    let first = dir.join(format!("{name}.{ext}"));
    if !first.exists() {
        return first;
    }
    (2..10_000)
        .map(|n| dir.join(format!("{name} {n}.{ext}")))
        .find(|p| !p.exists())
        .unwrap_or(first)
}

/// Builds the plain values the writers take. Audio tracks only, counted from zero in order.
/// Muted clips are left out and counted in the warnings.
pub fn build_in_session(session: &Session) -> (InSession, Vec<String>) {
    let rate = session.sample_rate;
    let mut warnings = Vec::new();
    let mut track_names = Vec::new();
    let mut clips = Vec::new();
    let (mut muted, mut looped, mut reversed) = (0usize, 0usize, 0usize);
    for track in session
        .tracks
        .iter()
        .filter(|t| matches!(t.kind, TrackKind::Audio))
    {
        let index = track_names.len();
        track_names.push(track.name.clone());
        for clip in &track.clips {
            if clip.muted {
                muted += 1;
                continue;
            }
            let Some(media) = session.media.iter().find(|m| m.id == clip.media_id) else {
                warnings.push(format!(
                    "{}: its audio file is missing, left out",
                    clip.name
                ));
                continue;
            };
            let src_rate = if media.sample_rate == 0 {
                rate
            } else {
                media.sample_rate
            };
            let to_session = |n: u64| -> u64 {
                if src_rate == rate {
                    n
                } else {
                    ((u128::from(n) * u128::from(rate) + u128::from(src_rate) / 2)
                        / u128::from(src_rate)) as u64
                }
            };
            if clip.looped.is_some() {
                looped += 1;
            }
            if clip.reversed {
                reversed += 1;
            }
            clips.push(InClip {
                track: index,
                name: clip.name.clone(),
                path: media.path.to_string_lossy().into_owned(),
                start: clip.start,
                source_start: to_session(clip.source_in),
                length: clip.timeline_length(src_rate, rate),
                media_length: Some(to_session(media.length)),
                stretch: clip.stretch_ratio(),
            });
        }
    }
    if muted > 0 {
        warnings.push(format!("{muted} muted clip(s) left out."));
    }
    if looped > 0 {
        warnings.push(format!(
            "{looped} looped clip(s) are written as one region the length of the loop."
        ));
    }
    if reversed > 0 {
        warnings.push(format!("{reversed} reversed clip(s) are written forwards."));
    }
    let markers = session
        .markers
        .iter()
        .map(|m| InMarker {
            name: m.name.clone(),
            start: m.position,
            length: m.duration,
        })
        .collect();
    (
        InSession {
            title: session.name.clone(),
            sample_rate: rate,
            track_names,
            clips,
            markers,
        },
        warnings,
    )
}

/// Writes the chosen interchange file(s) for the open session into `folder`. Existing files are
/// never overwritten: a numbered name is used instead.
#[tauri::command]
pub async fn export_interchange(
    open: State<'_, OpenSession>,
    folder: PathBuf,
    request: InterchangeRequest,
) -> Result<InterchangeResult, String> {
    let session = open.0.lock().map_err(|e| e.to_string())?.session.clone();
    spawn_blocking(move || {
        if !folder.is_dir() {
            return Err("That folder is not there.".to_string());
        }
        let rate = parse_rate(&request.rate).ok_or("Pick a frame rate.")?;
        let start_frames = timecode_to_frames(&request.start, rate)
            .ok_or("The start time must look like 01:00:00:00.")?;
        let (input, mut warnings) = build_in_session(&session);
        let base = safe_name(&session.name);
        let opts = EdlOptions { rate, start_frames };
        let mut written = Vec::new();
        let mut write = |name: String, ext: &str, text: &str| -> Result<(), String> {
            let path = free_path(&folder, &name, ext);
            std::fs::write(&path, text.as_bytes())
                .map_err(|e| format!("Could not write {}: {e}", path.display()))?;
            written.push(path);
            Ok(())
        };
        match request.kind {
            InterchangeKind::Edl => {
                let (files, report) = edl::edls(&input, &opts);
                warnings.extend(report.warnings);
                for f in files {
                    let label = input
                        .track_names
                        .get(f.track)
                        .cloned()
                        .unwrap_or_else(|| format!("Track {}", f.track + 1));
                    write(format!("{base} - {}", safe_name(&label)), "edl", &f.text)?;
                }
            }
            InterchangeKind::MarkerEdl => {
                if input.markers.is_empty() {
                    return Err("The session has no markers.".to_string());
                }
                let (text, report) = edl::marker_edl(&input, &opts, &request.color);
                warnings.extend(report.warnings);
                write(format!("{base} markers"), "edl", &text)?;
            }
            InterchangeKind::Otio => {
                if input.clips.is_empty() {
                    return Err("The session has no clips.".to_string());
                }
                write(base, "otio", &otio::otio_string(&input))?;
            }
        }
        if written.is_empty() {
            return Err("There was nothing to write.".to_string());
        }
        Ok(InterchangeResult {
            files: written,
            warnings,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_lose_path_characters() {
        assert_eq!(safe_name("Ep 12: A/B?"), "Ep 12 A B");
        assert_eq!(safe_name("  "), "Session");
    }

    #[test]
    fn rates_parse() {
        assert_eq!(parse_rate("29.97df"), Some(FrameRate::F29_97Df));
        assert_eq!(parse_rate("60"), None);
    }
}
