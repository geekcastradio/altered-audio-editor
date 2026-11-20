// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Interface preferences in the app config folder.

use std::collections::BTreeMap;

use serde_json::Value;
use tauri::State;

use crate::app_core::AppCore;

use crate::session::preferences;

#[tauri::command]
pub fn preferences_load(core: State<'_, AppCore>) -> Result<BTreeMap<String, Value>, String> {
    Ok(preferences::load_preferences(&core.config_dir()?))
}

#[tauri::command]
pub fn preferences_save(
    core: State<'_, AppCore>,
    values: BTreeMap<String, Value>,
) -> Result<(), String> {
    preferences::save_preferences(&core.config_dir()?, values).map_err(|e| e.to_string())
}
