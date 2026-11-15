// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Reduce Bleed: one clip's audio, copied and cleaned against another mic's track. The original
//! file is never changed. The interface points the clip at the copy, so undo brings it back.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::async_runtime::spawn_blocking;
use tauri::State;

use super::effects::{isolate, render};
use super::session::OpenSession;
use crate::bleed_copy::clean_wav;
use crate::dsp::bleed::Settings;
use crate::export::{mixdown, region};
use crate::session::model::Id;

/// What the interface can set. Anything left out takes the default.
#[derive(Debug, Clone, Deserialize)]
pub struct BleedSettings {
    pub filter_ms: Option<f32>,
    pub step: Option<f32>,
    pub max_bleed_db: Option<f32>,
    pub quiet_db: Option<f32>,
}

impl BleedSettings {
    fn settings(&self) -> Settings {
        let d = Settings::default();
        Settings {
            filter_ms: self.filter_ms.unwrap_or(d.filter_ms).clamp(2.0, 100.0),
            step: self.step.unwrap_or(d.step).clamp(0.01, 1.0),
            max_bleed_db: self
                .max_bleed_db
                .unwrap_or(d.max_bleed_db)
                .clamp(-40.0, 0.0),
            quiet_db: self.quiet_db.unwrap_or(d.quiet_db).clamp(-120.0, -20.0),
        }
    }
}

#[derive(Serialize)]
pub struct BleedCopy {
    pub path: PathBuf,
    /// How much quieter the clip became overall, in dB.
    pub removed_db: f32,
}

/// Stereo frames to mono.
fn mono(stereo: &[f32]) -> Vec<f32> {
    stereo
        .as_chunks::<2>()
        .0
        .iter()
        .map(|f| (f[0] + f[1]) * 0.5)
        .collect()
}

/// Copies the clip's audio to a new file in `folder` and takes the reference track's sound out
/// of it. The clip must play its file at normal speed, at the session's sample rate.
#[tauri::command]
pub async fn bleed_reduce_copy(
    open: State<'_, OpenSession>,
    clip_id: Id,
    reference_track: Id,
    folder: PathBuf,
    name: String,
    settings: BleedSettings,
) -> Result<BleedCopy, String> {
    let session = open.0.lock().map_err(|e| e.to_string())?.session.clone();
    spawn_blocking(move || {
        let (track, clip) = session
            .tracks
            .iter()
            .find_map(|t| t.clips.iter().find(|c| c.id == clip_id).map(|c| (t, c)))
            .ok_or("The clip is gone.")?;
        if track.id == reference_track {
            return Err("A mic cannot be cleaned against itself.".to_string());
        }
        if clip.looped.is_some() || clip.stretch.is_some() {
            return Err(format!(
                "\"{}\" is looped or stretched. Reduce Bleed needs a clip that plays its file at normal speed.",
                clip.name
            ));
        }
        let media = session
            .media
            .iter()
            .find(|m| m.id == clip.media_id)
            .ok_or("The file for this clip is missing from the session.")?;
        if media.sample_rate != session.sample_rate {
            return Err(format!(
                "\"{}\" is at {} Hz but the session is at {} Hz. Reduce Bleed needs them to match.",
                clip.name, media.sample_rate, session.sample_rate
            ));
        }
        if !session.tracks.iter().any(|t| t.id == reference_track) {
            return Err("The other mic's track is gone.".into());
        }
        let length = clip.timeline_length(media.sample_rate, session.sample_rate);
        let reference = mono(&render(
            &isolate(&session, Some(reference_track), None),
            clip.start,
            clip.start + length,
        )?);
        std::fs::create_dir_all(&folder)
            .map_err(|e| format!("Could not create {}: {e}", folder.display()))?;
        let path = mixdown::unique_path(&folder, &name, "wav");
        region::copy_region(&media.path, clip.source_in, clip.source_out, &path)
            .map_err(|e| e.to_string())?;
        match clean_wav(&path, &reference, &settings.settings()) {
            Ok(removed_db) => Ok(BleedCopy { path, removed_db }),
            Err(e) => {
                let _ = std::fs::remove_file(&path);
                Err(e)
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
