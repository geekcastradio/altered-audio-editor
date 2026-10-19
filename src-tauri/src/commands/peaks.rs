// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Waveform peaks for the Editor. Building runs off the UI thread and
//! reports progress as `peaks-progress` events.

use std::path::PathBuf;

use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::ipc::Response;
use tauri::{AppHandle, Emitter, Manager};

use crate::peaks::{self, PeakSummary};

fn cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("peaks"))
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    path: PathBuf,
    progress: f64,
}

/// Makes sure the peak cache for a file exists and returns its summary.
#[tauri::command]
pub async fn peaks_ensure(app: AppHandle, path: PathBuf) -> Result<PeakSummary, String> {
    let dir = cache_dir(&app)?;
    spawn_blocking(move || {
        let p = path.clone();
        peaks::ensure(&dir, &path, |progress| {
            let _ = app.emit(
                "peaks-progress",
                Progress {
                    path: p.clone(),
                    progress,
                },
            );
            true
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// One level of peaks as raw little-endian i16 pairs (min, max per channel).
#[tauri::command]
pub async fn peaks_level(app: AppHandle, path: PathBuf, level: usize) -> Result<Response, String> {
    let dir = cache_dir(&app)?;
    spawn_blocking(move || {
        // A cleared cache (Settings > Media and Cache) is rebuilt here.
        peaks::read_level(&dir, &path, level)
            .or_else(|_| {
                peaks::ensure(&dir, &path, |_| true)
                    .and_then(|_| peaks::read_level(&dir, &path, level))
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
    .map(Response::new)
}
