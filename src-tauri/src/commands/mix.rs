// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Multi-mic tools: Auto Mic Mix (turn each mic down while someone else is
//! talking) and Talk Time (how long each voice spoke). Both read the audio a
//! minute at a time, so a two-hour show does not have to fit in memory. The
//! interface turns the volume points into one undoable edit.

use serde::{Deserialize, Serialize};
use tauri::async_runtime::spawn_blocking;
use tauri::State;

use super::effects::{isolate, render};
use super::session::OpenSession;
use crate::dsp::automix::{hop_len, mix_from_levels, talk_seconds, track_levels, Settings};
use crate::session::model::{EnvelopeCurve, EnvelopePoint, Id, Session};

/// Longest tracks the analysis will take, in tracks.
const MAX_TRACKS: usize = 16;
/// How many hops are read at a time (one minute at 10 ms hops).
const CHUNK_HOPS: usize = 6_000;

/// The settings the dialog offers; anything left out takes the default.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MixOptions {
    pub open_db: Option<f32>,
    pub margin_db: Option<f32>,
    pub reduce_db: Option<f32>,
    pub fade_ms: Option<f32>,
    pub attack_ms: Option<f32>,
    pub hold_ms: Option<f32>,
}

impl MixOptions {
    fn settings(&self) -> Settings {
        let d = Settings::default();
        Settings {
            open_db: self.open_db.unwrap_or(d.open_db).clamp(2.0, 30.0),
            margin_db: self.margin_db.unwrap_or(d.margin_db).clamp(0.0, 24.0),
            reduce_db: self.reduce_db.unwrap_or(d.reduce_db).clamp(1.0, 40.0),
            fade_ms: self.fade_ms.unwrap_or(d.fade_ms).clamp(10.0, 1000.0),
            attack_ms: self.attack_ms.unwrap_or(d.attack_ms).clamp(2.0, 500.0),
            hold_ms: self.hold_ms.unwrap_or(d.hold_ms).clamp(0.0, 2000.0),
        }
    }
}

/// One track's result.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackMix {
    pub track: Id,
    pub name: String,
    /// Volume points in session samples, ready for a Volume envelope.
    pub points: Vec<EnvelopePoint>,
    /// Seconds this voice was the one talking.
    pub talk_seconds: f32,
    /// Its share of everyone's talk time, 0 to 1.
    pub share: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MixAnalysis {
    pub tracks: Vec<TrackMix>,
    pub total_seconds: f32,
}

/// Where the last clip on any track ends.
fn session_end(session: &Session) -> u64 {
    let rate = session.sample_rate;
    session
        .tracks
        .iter()
        .flat_map(|t| &t.clips)
        .map(|c| {
            let src = session
                .media
                .iter()
                .find(|m| m.id == c.media_id)
                .map_or(rate, |m| m.sample_rate);
            c.start + c.timeline_length(src, rate)
        })
        .max()
        .unwrap_or(0)
}

/// Hop levels for one track, read a chunk at a time.
fn levels_for(session: &Session, track: Id, end: u64) -> Result<Vec<f32>, String> {
    let rate = session.sample_rate;
    let chunk = (hop_len(rate) * CHUNK_HOPS) as u64;
    let only = isolate(session, Some(track), None);
    let mut out = Vec::new();
    let mut at = 0u64;
    while at < end {
        let to = (at + chunk).min(end);
        let stereo = render(&only, at, to)?;
        let mono: Vec<f32> = stereo
            .as_chunks::<2>()
            .0
            .iter()
            .map(|f| (f[0] + f[1]) * 0.5)
            .collect();
        out.extend(track_levels(&mono, rate));
        at = to;
    }
    Ok(out)
}

/// Works out the mix and the talk time for these tracks (two or more).
#[tauri::command]
pub async fn mix_analyze(
    open: State<'_, OpenSession>,
    tracks: Vec<Id>,
    options: Option<MixOptions>,
) -> Result<MixAnalysis, String> {
    let session = open.0.lock().map_err(|e| e.to_string())?.session.clone();
    spawn_blocking(move || {
        if tracks.len() < 2 {
            return Err("Pick at least two tracks, one for each mic.".into());
        }
        if tracks.len() > MAX_TRACKS {
            return Err(format!("That is more than {MAX_TRACKS} tracks."));
        }
        let rate = session.sample_rate;
        let end = session_end(&session);
        if end == 0 {
            return Err("There is no audio to work from.".into());
        }
        let settings = options.unwrap_or_default().settings();
        let mut lv = Vec::new();
        let mut names = Vec::new();
        for id in &tracks {
            let t = session
                .tracks
                .iter()
                .find(|t| t.id == *id)
                .ok_or("A picked track is gone.")?;
            names.push(t.name.clone());
            lv.push(levels_for(&session, *id, end)?);
        }
        let curves = mix_from_levels(&lv, rate, end as usize, &settings)
            .ok_or("Pick at least two tracks, one for each mic.")?;
        let talk = talk_seconds(&lv, &settings);
        let total: f32 = talk.iter().sum();
        let tracks = tracks
            .iter()
            .zip(names)
            .zip(curves)
            .zip(talk)
            .map(|(((id, name), curve), secs)| TrackMix {
                track: *id,
                name,
                points: curve
                    .into_iter()
                    .map(|p| EnvelopePoint {
                        position: p.sample as u64,
                        value: f64::from(p.gain_db),
                        curve: EnvelopeCurve::Linear,
                    })
                    .collect(),
                talk_seconds: secs,
                share: if total > 0.0 { secs / total } else { 0.0 },
            })
            .collect();
        Ok(MixAnalysis {
            tracks,
            total_seconds: end as f32 / rate as f32,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_are_clamped_and_default() {
        let s = MixOptions::default().settings();
        assert_eq!(s, Settings::default());
        let wild = MixOptions {
            reduce_db: Some(500.0),
            hold_ms: Some(-5.0),
            ..Default::default()
        }
        .settings();
        assert_eq!(wild.reduce_db, 40.0);
        assert_eq!(wild.hold_ms, 0.0);
    }

    #[test]
    fn an_empty_session_has_no_end() {
        assert_eq!(session_end(&Session::new("t", 48_000)), 0);
    }
}
