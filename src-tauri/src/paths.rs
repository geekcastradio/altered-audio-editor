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

/// Name of the settings folder in the per-user config area (Roaming on
/// Windows). Older versions used a folder named after the app identifier.
const SETTINGS_FOLDER: &str = "Altered Audio";

/// The folder holding settings, presets, shortcuts, layouts, templates and
/// recovery copies: `Altered Audio` in the per-user config area, or
/// `data/config` in portable mode.
pub fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Some(root) = portable_root() {
        return Ok(root.join("config"));
    }
    app.path()
        .config_dir()
        .map(|d| d.join(SETTINGS_FOLDER))
        .map_err(|e| e.to_string())
}

/// Moves settings from the folders older versions used (named after the
/// app identifier) into [`config_dir`]. Run once, early at startup, before
/// anything reads settings. Nothing is overwritten, and an old file is only
/// removed after its copy is checked. Failures leave the old folder alone.
pub fn migrate_settings(app: &AppHandle) {
    if portable_root().is_some() {
        return;
    }
    let Ok(new) = config_dir(app) else { return };
    let resolver = app.path();
    let mut olds: Vec<PathBuf> = Vec::new();
    for old in [resolver.app_config_dir(), resolver.app_data_dir()]
        .into_iter()
        .flatten()
    {
        if old != new && !olds.contains(&old) {
            olds.push(old);
        }
    }
    for old in olds {
        if let Err(e) = move_settings(&old, &new) {
            eprintln!("Could not move settings from {}: {e}", old.display());
        }
    }
}

/// Every file under `dir`, as paths relative to `base`.
fn files_under(dir: &Path, base: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            files_under(&path, base, out)?;
        } else if let Ok(rel) = path.strip_prefix(base) {
            out.push(rel.to_path_buf());
        }
    }
    Ok(())
}

/// Removes empty folders under `dir`, then `dir` itself when it is empty.
fn prune_empty(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                prune_empty(&entry.path());
            }
        }
    }
    let _ = std::fs::remove_dir(dir);
}

/// Copies every file from `old` into `new` (never over an existing file),
/// compares each copy with its source, and only then removes the sources.
/// Returns how many files moved.
pub fn move_settings(old: &Path, new: &Path) -> std::io::Result<usize> {
    if !old.is_dir() {
        return Ok(0);
    }
    let mut files = Vec::new();
    files_under(old, old, &mut files)?;
    if files.is_empty() {
        prune_empty(old);
        return Ok(0);
    }
    std::fs::create_dir_all(new)?;
    let mut copied: Vec<PathBuf> = Vec::new();
    for rel in &files {
        let src = old.join(rel);
        let dest = new.join(rel);
        if dest.exists() {
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut part = dest.clone().into_os_string();
        part.push(".part");
        let part = PathBuf::from(part);
        std::fs::copy(&src, &part)?;
        std::fs::rename(&part, &dest)?;
        if std::fs::read(&src)? != std::fs::read(&dest)? {
            let _ = std::fs::remove_file(&dest);
            return Err(std::io::Error::other(format!(
                "the copy of {} did not match",
                rel.display()
            )));
        }
        copied.push(src);
    }
    for src in &copied {
        let _ = std::fs::remove_file(src);
    }
    prune_empty(old);
    if !copied.is_empty() {
        let note = format!(
            "Settings moved here from {} by Altered Audio Editor.\r\n",
            old.display()
        );
        let _ = std::fs::write(new.join("moved-settings.txt"), note);
    }
    Ok(copied.len())
}

/// The visible folder in Documents that holds libraries, sessions, backups
/// and working files. Older versions put backups and the cache in a folder
/// named `Altered Audio Editor`; [`migrate_documents`] moves those over.
const DOCS_FOLDER: &str = "Altered Audio";
const OLD_DOCS_FOLDER: &str = "Altered Audio Editor";

/// The folder for backups: `Documents/Altered Audio/Backups`, or
/// `data/Backups` in portable mode, or inside the config folder when
/// Documents can't be found.
pub fn backups_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Some(root) = portable_root() {
        return Ok(root.join("Backups"));
    }
    match app.path().document_dir() {
        Ok(docs) => Ok(docs.join(DOCS_FOLDER).join("Backups")),
        Err(_) => Ok(config_dir(app)?.join("Backups")),
    }
}

/// Moves `Backups` and `Cache` from the old `Altered Audio Editor` folder in
/// Documents into `Altered Audio`. Run early at startup. Files are renamed,
/// not copied, so a large cache moves at once; nothing is overwritten and
/// nothing is deleted except folders left empty.
pub fn migrate_documents(app: &AppHandle) {
    if portable_root().is_some() {
        return;
    }
    let Ok(docs) = app.path().document_dir() else {
        return;
    };
    let (old, new) = (docs.join(OLD_DOCS_FOLDER), docs.join(DOCS_FOLDER));
    for name in ["Backups", "Cache"] {
        if let Err(e) = move_folder(&old.join(name), &new.join(name)) {
            eprintln!("Could not move {name} from {}: {e}", old.display());
        }
    }
    // Only removes the old folder when nothing else is in it.
    let _ = std::fs::remove_dir(&old);
}

/// Moves a folder to a new place by renaming. When the new place already
/// exists, files are moved into it one by one and never over an existing
/// file; anything that cannot move stays where it is.
pub fn move_folder(old: &Path, new: &Path) -> std::io::Result<usize> {
    if !old.is_dir() {
        return Ok(0);
    }
    if !new.exists() {
        if let Some(parent) = new.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut count = Vec::new();
        files_under(old, old, &mut count)?;
        std::fs::rename(old, new)?;
        return Ok(count.len());
    }
    let mut files = Vec::new();
    files_under(old, old, &mut files)?;
    let mut moved = 0;
    for rel in files {
        let dest = new.join(&rel);
        if dest.exists() {
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(old.join(&rel), &dest)?;
        moved += 1;
    }
    prune_empty(old);
    Ok(moved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_move_over_with_their_folders_and_the_old_ones_go() {
        let d = tempfile::tempdir().unwrap();
        let (old, new) = (d.path().join("old"), d.path().join("new"));
        std::fs::create_dir_all(old.join("templates")).unwrap();
        std::fs::write(old.join("preferences.json"), b"{\"a\":1}").unwrap();
        std::fs::write(old.join("templates").join("t.json"), b"t").unwrap();
        assert_eq!(move_settings(&old, &new).unwrap(), 2);
        assert_eq!(
            std::fs::read(new.join("preferences.json")).unwrap(),
            b"{\"a\":1}"
        );
        assert_eq!(
            std::fs::read(new.join("templates").join("t.json")).unwrap(),
            b"t"
        );
        assert!(new.join("moved-settings.txt").exists());
        assert!(!old.exists());
    }

    #[test]
    fn a_file_already_in_the_new_folder_is_never_overwritten_and_its_old_copy_stays() {
        let d = tempfile::tempdir().unwrap();
        let (old, new) = (d.path().join("old"), d.path().join("new"));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(old.join("preferences.json"), b"old").unwrap();
        std::fs::write(new.join("preferences.json"), b"new").unwrap();
        std::fs::write(old.join("keymap.json"), b"k").unwrap();
        assert_eq!(move_settings(&old, &new).unwrap(), 1);
        assert_eq!(std::fs::read(new.join("preferences.json")).unwrap(), b"new");
        assert_eq!(std::fs::read(new.join("keymap.json")).unwrap(), b"k");
        assert!(old.join("preferences.json").exists());
        assert!(!old.join("keymap.json").exists());
    }

    #[test]
    fn nothing_to_move_is_not_an_error() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            move_settings(&d.path().join("missing"), &d.path().join("new")).unwrap(),
            0
        );
        assert!(!d.path().join("new").exists());
        let empty = d.path().join("empty");
        std::fs::create_dir_all(empty.join("sub")).unwrap();
        assert_eq!(move_settings(&empty, &d.path().join("new")).unwrap(), 0);
        assert!(!empty.exists());
    }

    #[test]
    fn a_folder_moves_whole_or_merges_without_overwriting() {
        let d = tempfile::tempdir().unwrap();
        let (old, new) = (
            d.path().join("old").join("Cache"),
            d.path().join("new").join("Cache"),
        );
        std::fs::create_dir_all(old.join("peaks")).unwrap();
        std::fs::write(old.join("peaks").join("a.bin"), b"a").unwrap();
        assert_eq!(move_folder(&old, &new).unwrap(), 1);
        assert_eq!(
            std::fs::read(new.join("peaks").join("a.bin")).unwrap(),
            b"a"
        );
        assert!(!old.exists());

        std::fs::create_dir_all(old.join("peaks")).unwrap();
        std::fs::write(old.join("peaks").join("a.bin"), b"older").unwrap();
        std::fs::write(old.join("peaks").join("b.bin"), b"b").unwrap();
        assert_eq!(move_folder(&old, &new).unwrap(), 1);
        assert_eq!(
            std::fs::read(new.join("peaks").join("a.bin")).unwrap(),
            b"a"
        );
        assert_eq!(
            std::fs::read(new.join("peaks").join("b.bin")).unwrap(),
            b"b"
        );
        assert!(old.join("peaks").join("a.bin").exists());
        assert_eq!(move_folder(&d.path().join("none"), &new).unwrap(), 0);
    }

    #[test]
    fn a_portable_marker_switches_to_a_data_folder() {
        let dir = tempfile::tempdir().unwrap();
        assert!(portable_root_for(dir.path()).is_none());
        std::fs::write(dir.path().join("portable"), b"").unwrap();
        assert_eq!(portable_root_for(dir.path()), Some(dir.path().join("data")));
    }
}
