// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! In-app updates. The app only talks to GitHub when the user picks
//! Help > Check for Updates, or has turned on the weekly check (off by
//! default). It reads one signed `latest.json` from the project's
//! releases, and installs only after a click in the update dialog.
//! The signature is checked against the public key in `tauri.conf.json`.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_updater::{Update, UpdaterExt};

/// The update found by the last check, kept until it is installed.
#[derive(Default)]
pub struct PendingUpdate(Mutex<Option<Update>>);

/// What the update dialog shows.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current: String,
    pub notes: Option<String>,
    /// False for the portable zip and for systems without update files,
    /// which get a link to the download page instead.
    pub can_install: bool,
}

/// True when this copy was put in place by the Windows installer, which
/// leaves `uninstall.exe` beside the program. The portable zip has none,
/// so an update would install a second copy instead of replacing it.
fn can_install() -> bool {
    #[cfg(windows)]
    {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("uninstall.exe").is_file()))
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Asks GitHub whether a newer version exists. `None` means up to date.
#[tauri::command]
pub async fn update_check(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<Option<UpdateInfo>, String> {
    let found = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;
    let info = found.as_ref().map(|u| UpdateInfo {
        version: u.version.clone(),
        current: u.current_version.clone(),
        notes: u.body.clone().filter(|n| !n.trim().is_empty()),
        can_install: can_install(),
    });
    *pending.0.lock().map_err(|e| e.to_string())? = found;
    Ok(info)
}

/// Downloads the update found by the last check, installs it and
/// restarts the app. The frontend asks about unsaved work first.
#[tauri::command]
pub async fn update_install(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<(), String> {
    if !can_install() {
        return Err("This copy cannot update itself. Download the new version from the releases page.".into());
    }
    let update = pending
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .take()
        .ok_or_else(|| "Check for updates first.".to_string())?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())?;
    app.restart()
}
