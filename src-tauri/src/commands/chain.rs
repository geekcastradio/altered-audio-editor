// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Process chains: pick a chain file and check it against the action list.

use std::path::PathBuf;

use serde::Serialize;
use serde_json::{Map, Value};
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use crate::chain::{self, Chain};

/// A chain read from a file, with what is wrong with it and each step's settings filled in.
#[derive(Debug, Serialize)]
pub struct ChainLoaded {
    pub chain: Chain,
    pub problems: Vec<ChainProblem>,
    /// One entry per step: its settings with defaults filled in. Empty for a step that has a problem.
    pub resolved: Vec<Map<String, Value>>,
}

#[derive(Debug, Serialize)]
pub struct ChainProblem {
    pub step: Option<usize>,
    pub message: String,
}

/// Opens the picker for process chain files.
#[tauri::command]
pub async fn chain_pick_open(app: AppHandle) -> Result<Option<PathBuf>, String> {
    let parent = app.get_webview_window("main");
    spawn_blocking(move || {
        let mut d = app
            .dialog()
            .file()
            .set_title("Run Process Chain")
            .add_filter("Process chain (.json)", &["json"]);
        if let Some(w) = parent.as_ref() {
            d = d.set_parent(w);
        }
        d.blocking_pick_file()
            .and_then(|f| f.as_path().map(|p| p.to_path_buf()))
    })
    .await
    .map_err(|e| e.to_string())
}

/// Reads a chain file and checks it.
#[tauri::command]
pub async fn chain_load(path: PathBuf) -> Result<ChainLoaded, String> {
    spawn_blocking(move || load(&path))
        .await
        .map_err(|e| e.to_string())?
}

fn load(path: &std::path::Path) -> Result<ChainLoaded, String> {
    let len = std::fs::metadata(path)
        .map_err(|e| format!("Could not read the file: {e}"))?
        .len();
    if len > 1_000_000 {
        return Err("This file is too large to be a process chain.".into());
    }
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("Could not read the file: {e}"))?;
    let text = text.trim_start_matches('\u{feff}');
    let chain: Chain = serde_json::from_str(text)
        .map_err(|e| format!("This is not a readable process chain: {e}"))?;
    let problems: Vec<ChainProblem> = chain::validate(&chain)
        .into_iter()
        .map(|p| ChainProblem {
            step: p.step,
            message: p.message,
        })
        .collect();
    let resolved = chain
        .steps
        .iter()
        .map(|s| chain::resolved(s).unwrap_or_default())
        .collect();
    Ok(ChainLoaded {
        chain,
        problems,
        resolved,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_good_file_loads_with_defaults_filled_in() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("c.json");
        std::fs::write(
            &p,
            r#"{"version":1,"name":"Cleanup","steps":[{"action":"multitrack.close-dead-space","params":{"pause_seconds":0.4}}]}"#,
        )
        .unwrap();
        let l = load(&p).unwrap();
        assert!(l.problems.is_empty());
        assert_eq!(l.resolved[0]["pause_seconds"], serde_json::json!(0.4));
    }

    #[test]
    fn a_bad_file_says_what_is_wrong() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("c.json");
        std::fs::write(&p, "not json").unwrap();
        assert!(load(&p).unwrap_err().contains("not a readable"));
        std::fs::write(
            &p,
            r#"{"version":1,"name":"x","steps":[{"action":"nope"}]}"#,
        )
        .unwrap();
        let l = load(&p).unwrap();
        assert_eq!(l.problems.len(), 1);
        assert!(l.resolved[0].is_empty());
    }
}
