// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Edit > Settings > Backup and Restore. Thin wrappers over `crate::backup`.

use std::path::PathBuf;

use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

use crate::backup::{self, Manifest, Mode, Options, Report, SnapshotInfo};
use crate::paths;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub config_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub portable: bool,
    pub options: Options,
    pub snapshots: Vec<SnapshotInfo>,
}

fn version(app: &AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub async fn backup_info(app: AppHandle) -> Result<BackupInfo, String> {
    let config = paths::config_dir(&app)?;
    let dir = paths::backups_dir(&app)?;
    spawn_blocking(move || BackupInfo {
        portable: paths::portable_root().is_some(),
        options: backup::load_options(&config),
        snapshots: backup::list(&dir),
        config_dir: config,
        backups_dir: dir,
    })
    .await
    .map_err(|e| e.to_string())
}

/// Asks where to save a backup. None when cancelled.
#[tauri::command]
pub async fn backup_pick_save(app: AppHandle) -> Result<Option<PathBuf>, String> {
    let parent: Option<WebviewWindow> = app.get_webview_window("main");
    let start = paths::backups_dir(&app).ok();
    spawn_blocking(move || {
        let name = format!(
            "settings-{}.{}",
            chrono::Local::now().format("%Y-%m-%d"),
            backup::EXT
        );
        let mut d = app
            .dialog()
            .file()
            .set_title("Export Settings Backup")
            .set_file_name(name)
            .add_filter("Settings backup", &[backup::EXT]);
        if let Some(dir) = start.filter(|p| p.is_dir()) {
            d = d.set_directory(dir);
        }
        if let Some(w) = parent.as_ref() {
            d = d.set_parent(w);
        }
        d.blocking_save_file()
            .and_then(|f| f.as_path().map(|p| p.to_path_buf()))
    })
    .await
    .map_err(|e| e.to_string())
}

/// Asks which backup to open. None when cancelled.
#[tauri::command]
pub async fn backup_pick_open(app: AppHandle) -> Result<Option<PathBuf>, String> {
    let parent: Option<WebviewWindow> = app.get_webview_window("main");
    spawn_blocking(move || {
        let mut d = app
            .dialog()
            .file()
            .set_title("Import Settings Backup")
            .add_filter("Settings backup", &[backup::EXT]);
        if let Some(w) = parent.as_ref() {
            d = d.set_parent(w);
        }
        d.blocking_pick_file()
            .and_then(|f| f.as_path().map(|p| p.to_path_buf()))
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backup_export(
    app: AppHandle,
    dest: PathBuf,
    include_secrets: bool,
) -> Result<Manifest, String> {
    let config = paths::config_dir(&app)?;
    let v = version(&app);
    spawn_blocking(move || backup::create(&config, &dest, &v, "manual", include_secrets))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn backup_inspect(path: PathBuf) -> Result<Manifest, String> {
    spawn_blocking(move || backup::read_manifest(&path))
        .await
        .map_err(|e| e.to_string())?
}

/// Brings a backup in. Replace first takes a snapshot of what is there now, so it can be undone.
#[tauri::command]
pub async fn backup_import(
    app: AppHandle,
    path: PathBuf,
    mode: Mode,
    categories: Option<Vec<String>>,
) -> Result<Report, String> {
    let config = paths::config_dir(&app)?;
    let dir = paths::backups_dir(&app)?;
    let v = version(&app);
    spawn_blocking(move || {
        backup::read_manifest(&path)?;
        let options = backup::load_options(&config);
        backup::snapshot(&config, &dir, &v, "before-restore", true, &options)?;
        backup::restore(&path, &config, mode, categories.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn backup_snapshot_now(app: AppHandle) -> Result<Option<PathBuf>, String> {
    let config = paths::config_dir(&app)?;
    let dir = paths::backups_dir(&app)?;
    let v = version(&app);
    spawn_blocking(move || {
        backup::snapshot(
            &config,
            &dir,
            &v,
            "manual",
            true,
            &backup::load_options(&config),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn backup_set_options(app: AppHandle, options: Options) -> Result<(), String> {
    let config = paths::config_dir(&app)?;
    spawn_blocking(move || {
        backup::save_options(
            &config,
            &Options {
                keep: options.keep.clamp(1, 100),
                ..options
            },
        )
    })
    .await
    .map_err(|e| e.to_string())?
}
