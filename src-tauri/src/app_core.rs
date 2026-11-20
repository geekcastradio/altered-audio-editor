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

/// The folders the core keeps its files in, worked out once at startup.
/// A folder that could not be found keeps the reason, so a command can
/// report it the way it always did.
#[derive(Clone, Debug)]
pub struct Dirs {
    pub config: Result<PathBuf, String>,
    pub backups: Result<PathBuf, String>,
}

impl Dirs {
    /// Both folders under one root. For tests and headless runs.
    pub fn under(root: &std::path::Path) -> Self {
        Self {
            config: Ok(root.join("config")),
            backups: Ok(root.join("backups")),
        }
    }
}

/// What core logic may use to reach the interface it runs under.
#[derive(Clone)]
pub struct AppCore {
    sink: Arc<dyn EventSink>,
    dirs: Dirs,
}

impl AppCore {
    pub fn new(sink: Arc<dyn EventSink>, dirs: Dirs) -> Self {
        Self { sink, dirs }
    }

    /// The settings folder (shortcuts, preferences, layouts, templates).
    pub fn config_dir(&self) -> Result<PathBuf, String> {
        self.dirs.config.clone()
    }

    /// The folder settings backups are kept in.
    pub fn backups_dir(&self) -> Result<PathBuf, String> {
        self.dirs.backups.clone()
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
        let broken = AppCore::new(
            Arc::new(MemorySink::default()),
            Dirs {
                config: Err("no settings folder".to_string()),
                backups: Err("no backups folder".to_string()),
            },
        );
        assert_eq!(broken.config_dir().unwrap_err(), "no settings folder");
    }
}
