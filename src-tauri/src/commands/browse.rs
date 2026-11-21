// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Drive Manager commands: locations, folder listings, preview playback.

use std::path::PathBuf;

use tauri::async_runtime::spawn_blocking;
use tauri::State;

use crate::app_core::AppCore;
use crate::browse::{self, Entry, Location};
use crate::engine::{PreviewPlayer, PreviewStatus};

/// Drives first, then the user's common folders.
#[tauri::command]
pub async fn browse_locations(core: State<'_, AppCore>) -> Result<Vec<Location>, String> {
    let core = core.inner().clone();
    spawn_blocking(move || core.browse_locations())
        .await
        .map_err(|e| e.to_string())
}

/// Lists a folder for the Drive Manager.
#[tauri::command]
pub async fn browse_list(path: PathBuf, audio_only: bool) -> Result<Vec<Entry>, String> {
    spawn_blocking(move || browse::list_dir(&path, audio_only))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// Starts previewing a file, stopping any other preview.
#[tauri::command]
pub async fn preview_play(
    player: State<'_, PreviewPlayer>,
    transport: State<'_, crate::engine::Transport>,
    path: PathBuf,
    start: Option<f64>,
) -> Result<(), String> {
    // One thing plays at a time.
    transport.stop();
    player.play(&path, start.unwrap_or(0.0))
}

#[tauri::command]
pub fn preview_stop(player: State<'_, PreviewPlayer>) {
    player.stop();
}

#[tauri::command]
pub fn preview_status(player: State<'_, PreviewPlayer>) -> PreviewStatus {
    player.status()
}
