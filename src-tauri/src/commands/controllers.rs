// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Hardware controllers (MIDI in). See `controllers.rs`.

use tauri::State;

use crate::controllers::{Controllers, Message};

/// The MIDI inputs connected now.
#[tauri::command]
pub fn controller_ports() -> Vec<String> {
    Controllers::ports()
}

/// Listens to exactly these inputs; returns the ones that opened.
#[tauri::command]
pub fn controller_open(names: Vec<String>, state: State<'_, Controllers>) -> Vec<String> {
    state.open(&names)
}

/// What the controllers sent since the last call.
#[tauri::command]
pub fn controller_events(state: State<'_, Controllers>) -> Vec<Message> {
    state.events()
}
