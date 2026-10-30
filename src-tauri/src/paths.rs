// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Where the app keeps its settings. Normally the per-user app config
//! folder, which an uninstall or an upgrade leaves alone. A file named
//! `portable` next to the program switches to a `data` folder beside it
//! instead, for a stick or a self-managed install.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

/// The `data` folder beside the program when portable mode is on.
pub fn portable_root() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    portable_root_for(exe.parent()?)
}

/// Portable root for a program living in `exe_dir`, when it has a `portable` marker.
pub fn portable_root_for(exe_dir: &Path) -> Option<PathBuf> {
    exe_dir
        .join("portable")
        .exists()
        .then(|| exe_dir.join("data"))
}

/// The folder holding settings, presets, shortcuts, layouts and templates.
pub fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Some(root) = portable_root() {
        return Ok(root.join("config"));
    }
    app.path().app_config_dir().map_err(|e| e.to_string())
}

/// The folder for backups: `Documents/Altered Audio Editor/Backups`, or
/// `data/Backups` in portable mode, or inside the config folder when
/// Documents can't be found.
pub fn backups_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Some(root) = portable_root() {
        return Ok(root.join("Backups"));
    }
    match app.path().document_dir() {
        Ok(docs) => Ok(docs.join("Altered Audio Editor").join("Backups")),
        Err(_) => Ok(config_dir(app)?.join("Backups")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_portable_marker_switches_to_a_data_folder() {
        let dir = tempfile::tempdir().unwrap();
        assert!(portable_root_for(dir.path()).is_none());
        std::fs::write(dir.path().join("portable"), b"").unwrap();
        assert_eq!(portable_root_for(dir.path()), Some(dir.path().join("data")));
    }
}
