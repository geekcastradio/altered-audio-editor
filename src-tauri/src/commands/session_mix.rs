// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Insert Session as Clip: another saved session mixed down to one file.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use tauri::async_runtime::spawn_blocking;

use crate::export::mixdown::{self, MixdownRequest};
use crate::session::file;

/// What the page needs after the mix is written.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMix {
    pub path: PathBuf,
    pub seconds: f64,
    /// Media files the session uses that could not be found (they play as silence).
    pub missing: usize,
    /// Highest sample before conversion; above 1.0 the mix goes over 0 dBFS.
    pub peak: f32,
}

/// Mixes the saved session at `session_path` (everything: its tracks, buses and Master)
/// to a new 32-bit float WAV in `folder` (created if needed), named after `name`
/// without overwriting anything. The open session is not touched.
#[tauri::command]
pub async fn session_mix_copy(
    session_path: PathBuf,
    folder: PathBuf,
    name: String,
) -> Result<SessionMix, String> {
    spawn_blocking(move || -> Result<SessionMix, String> {
        let opened = file::open(&session_path).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&folder)
            .map_err(|e| format!("Could not create {}: {e}", folder.display()))?;
        let path = mixdown::unique_path(&folder, &name, "wav");
        let request = MixdownRequest {
            bit_depth: Some(32),
            ..MixdownRequest::default()
        };
        let stop = AtomicBool::new(false);
        let r = mixdown::mixdown(&opened.session, &request, &path, &stop, |_, _| {})
            .map_err(|e| e.to_string())?;
        Ok(SessionMix {
            path,
            seconds: r.frames as f64 / f64::from(r.sample_rate.max(1)),
            missing: opened.missing.len(),
            peak: r.peak,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
