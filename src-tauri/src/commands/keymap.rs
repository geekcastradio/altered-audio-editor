// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The user's shortcut changes, kept in the app config folder.

use std::collections::BTreeMap;

use tauri::State;

use crate::app_core::AppCore;

use crate::session::keymap;

#[tauri::command]
pub fn keymap_load(core: State<'_, AppCore>) -> Result<BTreeMap<String, Vec<String>>, String> {
    keymap::load_user_keymap(&core.config_dir()?).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn keymap_save(
    core: State<'_, AppCore>,
    bindings: BTreeMap<String, Vec<String>>,
) -> Result<(), String> {
    keymap::save_user_keymap(&core.config_dir()?, bindings).map_err(|e| e.to_string())
}
