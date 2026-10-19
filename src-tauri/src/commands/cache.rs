// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Settings > Media and Cache: where the app keeps its working files
//! and how much room they take. Only the waveform drawings are cleared
//! from here: they are rebuilt on demand. Edit results, untitled files
//! and recordings stay, since an open file or session may still use them.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize)]
pub struct CacheFolder {
    pub name: String,
    pub bytes: u64,
    /// Safe to clear from Settings.
    pub clearable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheInfo {
    pub path: PathBuf,
    pub folders: Vec<CacheFolder>,
}

fn size(p: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(p) else {
        return 0;
    };
    entries
        .flatten()
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => size(&e.path()),
            Ok(m) => m.len(),
            Err(_) => 0,
        })
        .sum()
}

/// Folders the app writes, with what each one holds.
const FOLDERS: [(&str, bool); 5] = [
    ("peaks", true),
    ("spectral", false),
    ("wave-fx", false),
    ("untitled", false),
    ("recordings", false),
];

fn cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_cache_dir().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cache_info(app: AppHandle) -> Result<CacheInfo, String> {
    let dir = cache_dir(&app)?;
    spawn_blocking(move || CacheInfo {
        folders: FOLDERS
            .iter()
            .map(|(n, c)| CacheFolder {
                name: (*n).to_string(),
                bytes: size(&dir.join(n)),
                clearable: *c,
            })
            .collect(),
        path: dir,
    })
    .await
    .map_err(|e| e.to_string())
}

/// Deletes the waveform drawings. Returns the bytes freed.
#[tauri::command]
pub async fn cache_clear_peaks(app: AppHandle) -> Result<u64, String> {
    let dir = cache_dir(&app)?.join("peaks");
    spawn_blocking(move || {
        let freed = size(&dir);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
        }
        Ok(freed)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_add_up_through_folders() {
        let d = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(d.path().join("a/b")).unwrap();
        std::fs::write(d.path().join("a/x"), [0u8; 10]).unwrap();
        std::fs::write(d.path().join("a/b/y"), [0u8; 5]).unwrap();
        assert_eq!(size(&d.path().join("a")), 15);
        assert_eq!(size(&d.path().join("missing")), 0);
    }
}
