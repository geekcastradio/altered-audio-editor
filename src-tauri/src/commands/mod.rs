// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Tauri command handlers. Keep these thin: parse input, call into the
//! core modules, return plain data.

pub mod app;
pub mod batch;
pub mod browse;
pub mod devices;
pub mod export;
pub mod keymap;
pub mod library;
pub mod links;
pub mod media;
pub mod peaks;
pub mod preferences;
pub mod quickpass;
pub mod record;
pub mod session;
pub mod spectral;
pub mod template;
pub mod transport;
pub mod wave;
pub mod workspace;
