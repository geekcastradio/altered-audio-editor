// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The handle the core hands to any interface.
//!
//! Phase 0 of the move to a native interface: logic that today reaches for
//! the Tauri window moves behind this handle one piece at a time. A command
//! then becomes a thin wrapper that builds nothing and decides nothing. The
//! handle starts with the event sink; shared state joins it as commands move.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;

use crate::events::{EventSink, EventSinkExt};
use crate::peaks::{self, PeakSummary};

/// File in the config folder that holds a working-files folder the user picked.
pub const CACHE_FOLDER_SETTING: &str = "cache-folder.txt";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PeaksProgress {
    path: PathBuf,
    progress: f64,
}

/// The folders the core keeps its files in, worked out once at startup.
/// A folder that could not be found keeps the reason, so a command can
/// report it the way it always did.
#[derive(Clone, Debug)]
pub struct Dirs {
    pub config: Result<PathBuf, String>,
    pub backups: Result<PathBuf, String>,
    /// The user's Documents folder, when the system has one.
    pub documents: Option<PathBuf>,
    /// The user's home folder, used when there is no Documents folder.
    pub home: Option<PathBuf>,
    /// The app's own cache folder, the last place working files can go.
    pub app_cache: Result<PathBuf, String>,
    /// The user's common folders in the order the Drive Manager lists them
    /// (Home, Desktop, Documents, Music, Downloads). Missing ones are left out.
    pub places: Vec<(String, PathBuf)>,
}

impl Dirs {
    /// Both folders under one root. For tests and headless runs.
    pub fn under(root: &std::path::Path) -> Self {
        Self {
            config: Ok(root.join("config")),
            backups: Ok(root.join("backups")),
            documents: Some(root.join("documents")),
            home: Some(root.join("home")),
            app_cache: Ok(root.join("app-cache")),
            places: ["Home", "Desktop", "Documents", "Music", "Downloads"]
                .iter()
                .map(|n| (n.to_string(), root.join("home").join(n)))
                .collect(),
        }
    }
}

/// What core logic may use to reach the interface it runs under.
#[derive(Clone)]
pub struct AppCore {
    sink: Arc<dyn EventSink>,
    dirs: Dirs,
    version: String,
}

impl AppCore {
    pub fn new(sink: Arc<dyn EventSink>, dirs: Dirs) -> Self {
        Self {
            sink,
            dirs,
            version: String::new(),
        }
    }

    /// Sets the app version the core reports (the interface knows it, the core does not).
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// The app version, for files the core writes (backups carry it).
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The settings folder (shortcuts, preferences, layouts, templates).
    pub fn config_dir(&self) -> Result<PathBuf, String> {
        self.dirs.config.clone()
    }

    /// The folder settings backups are kept in.
    pub fn backups_dir(&self) -> Result<PathBuf, String> {
        self.dirs.backups.clone()
    }

    /// Where new libraries go by default: Documents/Altered Audio/Libraries
    /// (the home folder when there is no Documents folder).
    pub fn libraries_root(&self) -> Result<PathBuf, String> {
        let docs = self
            .dirs
            .documents
            .clone()
            .or_else(|| self.dirs.home.clone())
            .ok_or_else(|| "The Documents folder could not be found.".to_string())?;
        Ok(docs.join("Altered Audio").join("Libraries"))
    }

    /// Where working files go: the folder the user picked in Settings, else
    /// `Documents/Altered Audio/Cache`, else the app's own cache folder when
    /// Documents can't be found. A picked folder that is gone falls back to the default.
    pub fn cache_root(&self) -> Result<PathBuf, String> {
        if let Ok(cfg) = self.config_dir() {
            if let Ok(text) = std::fs::read_to_string(cfg.join(CACHE_FOLDER_SETTING)) {
                let p = PathBuf::from(text.trim());
                if !text.trim().is_empty() && p.is_dir() {
                    return Ok(p);
                }
            }
        }
        match &self.dirs.documents {
            Some(docs) => Ok(docs.join("Altered Audio").join("Cache")),
            None => self.dirs.app_cache.clone(),
        }
    }

    /// Where unsaved work is set aside every two minutes for crash recovery.
    pub fn autosave_dir(&self) -> Result<PathBuf, String> {
        Ok(self.config_dir()?.join("autosave"))
    }

    /// Default folder for new sessions: Documents/Altered Audio/Sessions
    /// (home/Documents when the system has no Documents folder).
    pub fn sessions_default_folder(&self) -> Result<PathBuf, String> {
        let docs = self
            .dirs
            .documents
            .clone()
            .or_else(|| self.dirs.home.as_ref().map(|h| h.join("Documents")))
            .ok_or_else(|| "The Documents folder could not be found.".to_string())?;
        Ok(docs.join("Altered Audio").join("Sessions"))
    }

    /// Drives first, then the user's common folders that exist. Blocks.
    pub fn browse_locations(&self) -> Vec<crate::browse::Location> {
        let mut out = crate::browse::roots();
        for (label, path) in &self.dirs.places {
            if path.is_dir() {
                out.push(crate::browse::Location {
                    path: path.clone(),
                    label: label.clone(),
                    kind: crate::browse::LocationKind::Place,
                });
            }
        }
        out
    }

    /// Builds a file's waveform peaks if they are not held already and returns
    /// their summary, sending `peaks-progress` events while it works. Blocks.
    pub fn peaks_ensure(&self, path: &std::path::Path) -> Result<PeakSummary, String> {
        // Waveform files older versions kept on disk are removed once per run: peaks live in memory now.
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            if let Ok(root) = self.cache_root() {
                let _ = std::fs::remove_dir_all(root.join("peaks"));
            }
        });
        peaks::ensure(path, |progress| {
            let _ = self.emit(
                "peaks-progress",
                PeaksProgress {
                    path: path.to_path_buf(),
                    progress,
                },
            );
            true
        })
        .map_err(|e| e.to_string())
    }

    /// The folder session templates are kept in.
    pub fn templates_dir(&self) -> Result<PathBuf, String> {
        Ok(self.config_dir()?.join("templates"))
    }

    /// The place events go.
    pub fn sink(&self) -> &dyn EventSink {
        self.sink.as_ref()
    }

    /// Sends one named event. Errors mean the interface is gone; callers ignore them.
    pub fn emit<T: Serialize>(&self, name: &str, payload: T) -> Result<(), String> {
        self.sink.emit_event(name, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::MemorySink;

    #[test]
    fn emits_through_the_sink() {
        let mem = Arc::new(MemorySink::default());
        let core = AppCore::new(mem.clone(), Dirs::under(std::path::Path::new("x")));
        core.emit("export-progress", 3).unwrap();
        let got = mem.0.lock().unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, "export-progress");
    }

    #[test]
    fn clones_share_one_sink() {
        let mem = Arc::new(MemorySink::default());
        let a = AppCore::new(mem.clone(), Dirs::under(std::path::Path::new("x")));
        let b = a.clone();
        a.emit("a", 1).unwrap();
        b.emit("b", 2).unwrap();
        assert_eq!(mem.0.lock().unwrap().len(), 2);
    }

    #[test]
    fn folders_come_from_the_dirs_it_was_built_with() {
        let core = AppCore::new(
            Arc::new(MemorySink::default()),
            Dirs::under(std::path::Path::new("root")),
        );
        assert_eq!(
            core.config_dir().unwrap(),
            PathBuf::from("root").join("config")
        );
        assert_eq!(
            core.backups_dir().unwrap(),
            PathBuf::from("root").join("backups")
        );
        assert_eq!(
            core.templates_dir().unwrap(),
            PathBuf::from("root").join("config").join("templates")
        );
        let broken = AppCore::new(
            Arc::new(MemorySink::default()),
            Dirs {
                config: Err("no settings folder".to_string()),
                backups: Err("no backups folder".to_string()),
                documents: None,
                home: None,
                app_cache: Err("no cache folder".to_string()),
                places: Vec::new(),
            },
        );
        assert_eq!(broken.config_dir().unwrap_err(), "no settings folder");
        assert_eq!(broken.templates_dir().unwrap_err(), "no settings folder");
    }

    #[test]
    fn browse_locations_list_only_folders_that_exist() {
        let root = std::env::temp_dir().join(format!("aae-places-{}", std::process::id()));
        std::fs::create_dir_all(root.join("home").join("Music")).unwrap();
        let core = AppCore::new(Arc::new(MemorySink::default()), Dirs::under(&root));
        let places: Vec<String> = core
            .browse_locations()
            .into_iter()
            .filter(|l| matches!(l.kind, crate::browse::LocationKind::Place))
            .map(|l| l.label)
            .collect();
        std::fs::remove_dir_all(&root).ok();
        assert_eq!(places, vec!["Music".to_string()]);
    }

    #[test]
    fn autosave_and_sessions_folders() {
        let core = AppCore::new(
            Arc::new(MemorySink::default()),
            Dirs::under(std::path::Path::new("root")),
        );
        assert_eq!(
            core.autosave_dir().unwrap(),
            PathBuf::from("root").join("config").join("autosave")
        );
        assert_eq!(
            core.sessions_default_folder().unwrap(),
            PathBuf::from("root")
                .join("documents")
                .join("Altered Audio")
                .join("Sessions")
        );
        let mut dirs = Dirs::under(std::path::Path::new("root"));
        dirs.documents = None;
        let no_docs = AppCore::new(Arc::new(MemorySink::default()), dirs);
        assert_eq!(
            no_docs.sessions_default_folder().unwrap(),
            PathBuf::from("root")
                .join("home")
                .join("Documents")
                .join("Altered Audio")
                .join("Sessions")
        );
        let mut none = Dirs::under(std::path::Path::new("root"));
        none.documents = None;
        none.home = None;
        none.config = Err("no settings folder".to_string());
        let none = AppCore::new(Arc::new(MemorySink::default()), none);
        assert!(none.sessions_default_folder().is_err());
        assert_eq!(none.autosave_dir().unwrap_err(), "no settings folder");
    }

    #[test]
    fn libraries_and_cache_go_under_documents() {
        let core = AppCore::new(
            Arc::new(MemorySink::default()),
            Dirs::under(std::path::Path::new("root")),
        );
        let docs = PathBuf::from("root")
            .join("documents")
            .join("Altered Audio");
        assert_eq!(core.libraries_root().unwrap(), docs.join("Libraries"));
        assert_eq!(core.cache_root().unwrap(), docs.join("Cache"));
    }

    #[test]
    fn libraries_fall_back_to_home_and_cache_to_the_app_cache() {
        let mut dirs = Dirs::under(std::path::Path::new("root"));
        dirs.documents = None;
        let core = AppCore::new(Arc::new(MemorySink::default()), dirs.clone());
        assert_eq!(
            core.libraries_root().unwrap(),
            PathBuf::from("root")
                .join("home")
                .join("Altered Audio")
                .join("Libraries")
        );
        assert_eq!(
            core.cache_root().unwrap(),
            PathBuf::from("root").join("app-cache")
        );
        dirs.home = None;
        let core = AppCore::new(Arc::new(MemorySink::default()), dirs);
        assert!(core.libraries_root().is_err());
    }

    #[test]
    fn a_picked_cache_folder_wins_only_while_it_exists() {
        let root = tempfile::tempdir().unwrap();
        let core = AppCore::new(Arc::new(MemorySink::default()), Dirs::under(root.path()));
        let cfg = core.config_dir().unwrap();
        std::fs::create_dir_all(&cfg).unwrap();
        let picked = root.path().join("fast-disk");
        std::fs::create_dir_all(&picked).unwrap();
        std::fs::write(
            cfg.join(CACHE_FOLDER_SETTING),
            picked.to_string_lossy().as_bytes(),
        )
        .unwrap();
        assert_eq!(core.cache_root().unwrap(), picked);
        std::fs::remove_dir(&picked).unwrap();
        assert!(core.cache_root().unwrap().ends_with("Cache"));
    }

    #[test]
    fn carries_the_version_it_is_given() {
        let core = AppCore::new(
            Arc::new(MemorySink::default()),
            Dirs::under(std::path::Path::new("x")),
        );
        assert_eq!(core.version(), "");
        assert_eq!(core.with_version("0.47.0").version(), "0.47.0");
    }
}
