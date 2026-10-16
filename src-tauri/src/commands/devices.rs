// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The computer's audio inputs, for each track's input picker, plus on
//! Windows each output as a 'System audio' input. Only names and
//! channel counts are read; nothing is opened or recorded.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

use crate::engine::record::SYSTEM_AUDIO_PREFIX;
use tauri::async_runtime::spawn_blocking;

#[derive(Debug, Clone, Serialize)]
pub struct AudioInput {
    /// Device name as the system reports it.
    pub name: String,
    /// Input channels on the device.
    pub channels: u16,
    pub is_default: bool,
}

fn list() -> Vec<AudioInput> {
    let host = cpal::default_host();
    let default = host
        .default_input_device()
        .and_then(|d| d.description().ok())
        .map(|d| d.name().to_string());
    let Ok(devices) = host.input_devices() else {
        return Vec::new();
    };
    let mut out: Vec<AudioInput> = devices
        .filter_map(|d| {
            let name = d.description().ok()?.name().to_string();
            let channels = d.default_input_config().map(|c| c.channels()).unwrap_or(0);
            Some(AudioInput {
                is_default: default.as_deref() == Some(name.as_str()),
                name,
                channels,
            })
        })
        .filter(|d| d.channels > 0)
        .collect();
    out.sort_by_key(|d| (!d.is_default, d.name.to_lowercase()));
    // Windows can record what an output plays, such as a call app.
    if cfg!(windows) {
        if let Ok(outputs) = host.output_devices() {
            let mut sys: Vec<AudioInput> = outputs
                .filter_map(|d| {
                    let name = d.description().ok()?.name().to_string();
                    let channels = d.default_output_config().ok()?.channels();
                    Some(AudioInput {
                        name: format!("{SYSTEM_AUDIO_PREFIX}{name}"),
                        channels,
                        is_default: false,
                    })
                })
                .collect();
            sys.sort_by_key(|d| d.name.to_lowercase());
            out.extend(sys);
        }
    }
    out
}

/// The audio inputs available now. Empty when there are none.
#[tauri::command]
pub async fn audio_inputs() -> Result<Vec<AudioInput>, String> {
    spawn_blocking(list).await.map_err(|e| e.to_string())
}

/// Opens the system's own sound settings, where inputs are set up.
/// Local only: it starts the system settings app, nothing on the web.
#[tauri::command]
pub fn open_sound_settings() -> Result<(), String> {
    use std::process::Command;
    #[cfg(target_os = "windows")]
    let tries: &[(&str, &[&str])] = &[
        ("explorer", &["ms-settings:sound"]),
        ("control", &["mmsys.cpl"]),
    ];
    #[cfg(target_os = "macos")]
    let tries: &[(&str, &[&str])] = &[(
        "open",
        &["x-apple.systempreferences:com.apple.preference.sound"],
    )];
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let tries: &[(&str, &[&str])] = &[
        ("pavucontrol", &[]),
        ("gnome-control-center", &["sound"]),
        ("systemsettings", &["kcm_pulseaudio"]),
    ];
    for (cmd, args) in tries {
        if Command::new(cmd).args(*args).spawn().is_ok() {
            return Ok(());
        }
    }
    Err("Could not open the system's sound settings.".into())
}
