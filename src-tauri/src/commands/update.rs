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

/// Whether `exe` sits inside a macOS app bundle (`Something.app/Contents/MacOS/...`).
/// The updater replaces the whole bundle, so a bare binary (a development
/// build) has nothing to replace.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn inside_app_bundle(exe: &std::path::Path) -> bool {
    exe.ancestors()
        .skip(1)
        .any(|a| a.extension().is_some_and(|e| e == "app"))
}

/// Whether this copy can replace itself.
/// - Windows: put in place by the installer, which leaves `uninstall.exe`
///   beside the program. The portable zip has none, so an update would
///   install a second copy instead of replacing it.
/// - macOS: running from an app bundle.
/// - Linux: running as an AppImage (the `.deb` is updated by the system).
fn can_install() -> bool {
    #[cfg(windows)]
    {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("uninstall.exe").is_file()))
            .unwrap_or(false)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::current_exe()
            .map(|exe| inside_app_bundle(&exe))
            .unwrap_or(false)
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("APPIMAGE").is_some_and(|p| !p.is_empty())
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
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
        return Err(
            "This copy cannot update itself. Download the new version from the releases page."
                .into(),
        );
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

#[cfg(test)]
mod tests {
    use super::inside_app_bundle;
    use std::path::Path;

    #[test]
    fn a_binary_inside_an_app_bundle_can_update_and_a_loose_one_cannot() {
        assert!(inside_app_bundle(Path::new(
            "/Applications/Altered Audio Editor.app/Contents/MacOS/altered-audio-editor"
        )));
        assert!(!inside_app_bundle(Path::new(
            "/Users/x/src/target/debug/altered-audio-editor"
        )));
        // The file itself being called x.app does not count; only a folder around it does.
        assert!(!inside_app_bundle(Path::new("/tmp/thing.app")));
    }
}
