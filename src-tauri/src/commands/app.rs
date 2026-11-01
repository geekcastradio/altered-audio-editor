// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

use serde::Serialize;

use crate::session::{Document, Session};

/// Basic facts about the running core, shown in the status bar.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub session_schema_version: u32,
}

/// Builds the app info without needing a running Tauri app.
pub fn current_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        session_schema_version: Session::CURRENT_VERSION,
    }
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    current_app_info()
}

/// Files the app was started with (File Explorer's Open or Open with),
/// handed to the interface once, when it is ready.
#[derive(Default)]
pub struct LaunchPaths(pub std::sync::Mutex<Vec<String>>);

/// The arguments that name existing files, relative ones taken from `cwd`.
pub fn file_args<I: IntoIterator<Item = String>>(args: I, cwd: &std::path::Path) -> Vec<String> {
    args.into_iter()
        .filter(|a| !a.starts_with('-'))
        .map(|a| cwd.join(a))
        .filter(|p| p.is_file())
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}

/// The files the app was started with; empty after the first call.
#[tauri::command]
pub fn launch_paths(state: tauri::State<'_, LaunchPaths>) -> Vec<String> {
    std::mem::take(&mut *state.0.lock().unwrap_or_else(|e| e.into_inner()))
}

/// Free space, in bytes, on the drive that holds `path` (or the Documents folder when none is given).
/// None when the drive cannot be asked, so the status bar just leaves the figure out.
#[tauri::command]
pub fn disk_free(app: tauri::AppHandle, path: Option<String>) -> Option<u64> {
    use tauri::Manager;
    let mut dir = path
        .filter(|p| !p.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| app.path().document_dir().ok())?;
    // A file or a folder that is gone still lives on some drive: walk up to one that exists.
    while !dir.is_dir() {
        if !dir.pop() {
            return None;
        }
    }
    fs4::available_space(&dir).ok()
}

/// Minimizes the main window. Called from Window > Minimize.
#[tauri::command]
pub fn app_minimize(window: tauri::WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

/// Quits the app. Called from File > Exit.
#[tauri::command]
pub fn app_exit(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_package_version_and_schema_version() {
        let info = current_app_info();
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.session_schema_version, Session::CURRENT_VERSION);
    }

    #[test]
    fn only_arguments_naming_existing_files_are_taken() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Show.altaudio"), b"x").unwrap();
        let got = file_args(
            ["Show.altaudio", "--flag", "missing.altaudio"].map(String::from),
            dir.path(),
        );
        assert_eq!(got.len(), 1);
        assert!(got[0].ends_with("Show.altaudio"));
    }

    #[test]
    fn launch_paths_are_handed_over_once() {
        let state = LaunchPaths(std::sync::Mutex::new(vec!["a".into()]));
        assert_eq!(std::mem::take(&mut *state.0.lock().unwrap()), ["a"]);
        assert!(state.0.lock().unwrap().is_empty());
    }

    #[test]
    fn serializes_as_camel_case_for_the_frontend() {
        let json = serde_json::to_value(current_app_info()).unwrap();
        assert!(json.get("sessionSchemaVersion").is_some());
    }
}

/// Sets the window title, for example "Ep 440 * - Altered Audio Editor".
#[tauri::command]
pub fn app_set_title(app: tauri::AppHandle, title: String) -> Result<(), String> {
    use tauri::Manager;
    match app.get_webview_window("main") {
        Some(w) => w.set_title(&title).map_err(|e| e.to_string()),
        None => Ok(()),
    }
}
