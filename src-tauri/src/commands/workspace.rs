// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Panel layout persistence. The layout lives in the app config folder,
//! never in browser storage.

use std::path::PathBuf;

use serde_json::Value;
use tauri::State;

use crate::app_core::AppCore;

use crate::session::workspace;

/// Returns the saved layout, or null when none has been saved.
#[tauri::command]
pub fn workspace_load(core: State<'_, AppCore>) -> Result<Option<Value>, String> {
    workspace::load_user_layout(&core.config_dir()?).map_err(|e| e.to_string())
}

/// Saves the current layout.
#[tauri::command]
pub fn workspace_save(core: State<'_, AppCore>, layout: Value) -> Result<(), String> {
    workspace::save_user_layout(&core.config_dir()?, layout).map_err(|e| e.to_string())
}

/// Forgets the saved layout.
#[tauri::command]
pub fn workspace_clear(core: State<'_, AppCore>) -> Result<(), String> {
    workspace::clear_user_layout(&core.config_dir()?).map_err(|e| e.to_string())
}

/// Saved named workspaces, without their layouts.
#[tauri::command]
pub fn workspace_list(core: State<'_, AppCore>) -> Result<Vec<workspace::WorkspaceInfo>, String> {
    Ok(workspace::list_named(&core.config_dir()?))
}

/// One named workspace's saved layout, or null when it has none yet.
#[tauri::command]
pub fn workspace_get(core: State<'_, AppCore>, id: String) -> Result<Option<Value>, String> {
    workspace::load_named(&core.config_dir()?, &id)
        .map(|w| w.map(|w| w.layout))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn workspace_put(
    core: State<'_, AppCore>,
    id: String,
    name: String,
    built_in: bool,
    layout: Value,
) -> Result<(), String> {
    let mut w = workspace::Workspace::new(id, name, layout);
    w.built_in = built_in;
    workspace::save_named(&core.config_dir()?, w).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn workspace_delete(core: State<'_, AppCore>, id: String) -> Result<(), String> {
    workspace::delete_named(&core.config_dir()?, &id).map_err(|e| e.to_string())
}
