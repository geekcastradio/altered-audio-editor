// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Interface preferences in the app config folder.

use std::collections::BTreeMap;

use serde_json::Value;
use tauri::AppHandle;

use crate::session::preferences;

fn config_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    crate::paths::config_dir(app)
}

#[tauri::command]
pub fn preferences_load(app: AppHandle) -> Result<BTreeMap<String, Value>, String> {
    Ok(preferences::load_preferences(&config_dir(&app)?))
}

#[tauri::command]
pub fn preferences_save(app: AppHandle, values: BTreeMap<String, Value>) -> Result<(), String> {
    preferences::save_preferences(&config_dir(&app)?, values).map_err(|e| e.to_string())
}
