// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Which audio driver the app uses: the system's own (WASAPI on Windows,
//! Core Audio on macOS, ALSA on Linux) or ASIO on Windows. ASIO is only
//! there in builds made with the `asio` feature (the release builds); the
//! system driver is always the fallback, so a missing or busy ASIO
//! driver never leaves the app without sound.
//!
//! ASIO is a trademark of Steinberg Media Technologies GmbH. The ASIO
//! SDK is used under its GPLv3 license option.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static USE_ASIO: AtomicBool = AtomicBool::new(false);
/// The buffer size asked for, in frames. 0 leaves it to the driver.
static BUFFER_FRAMES: AtomicU32 = AtomicU32::new(0);

/// True when this build can talk to ASIO drivers.
pub const ASIO_BUILT: bool = cfg!(all(target_os = "windows", feature = "asio"));

/// Picks ASIO (true) or the system driver (false) from the next stream opened.
pub fn set_asio(on: bool) {
    USE_ASIO.store(on && ASIO_BUILT, Ordering::Relaxed);
}

pub fn asio() -> bool {
    USE_ASIO.load(Ordering::Relaxed)
}

/// Asks for a buffer of `frames` from the next stream opened (0: the driver's own).
pub fn set_buffer_frames(frames: u32) {
    BUFFER_FRAMES.store(frames.min(8192), Ordering::Relaxed);
}

pub fn buffer_frames() -> u32 {
    BUFFER_FRAMES.load(Ordering::Relaxed)
}

/// The buffer to open a stream with. Only a device that reports its range is asked
/// for a size, held inside that range; otherwise the driver's own size is kept.
pub fn pick_buffer(want: u32, supported: &cpal::SupportedBufferSize) -> cpal::BufferSize {
    match supported {
        cpal::SupportedBufferSize::Range { min, max } if want > 0 && min <= max => {
            cpal::BufferSize::Fixed(want.clamp(*min, *max))
        }
        _ => cpal::BufferSize::Default,
    }
}

/// `config` with the chosen buffer size, when the device allows one.
pub fn with_buffer(
    mut config: cpal::StreamConfig,
    supported: &cpal::SupportedBufferSize,
) -> cpal::StreamConfig {
    config.buffer_size = pick_buffer(buffer_frames(), supported);
    config
}

/// A pointer to the setting, added to an error that may come from a buffer the device refused.
pub fn buffer_hint() -> &'static str {
    if buffer_frames() > 0 {
        " If you set a buffer size in Settings > Audio Devices, set it back to Driver default."
    } else {
        ""
    }
}

/// The host to open devices on now.
pub fn host() -> cpal::Host {
    host_for(asio())
}

/// The ASIO host when `asio` (and it is built in and loads), else the system one.
#[allow(unused_variables)]
pub fn host_for(asio: bool) -> cpal::Host {
    #[cfg(all(target_os = "windows", feature = "asio"))]
    if asio {
        if let Ok(h) = cpal::host_from_id(cpal::HostId::Asio) {
            return h;
        }
    }
    cpal::default_host()
}

/// Names of the ASIO drivers installed, or empty when ASIO is not built
/// in or none are installed.
pub fn asio_drivers() -> Vec<String> {
    // Read the names from the registry. Asking each driver to load itself
    // can crash the whole app when a driver misbehaves, so nothing is
    // loaded here; a driver is only opened when the user picks ASIO.
    #[cfg(all(target_os = "windows", feature = "asio"))]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        if let Ok(out) = std::process::Command::new("reg")
            .args(["query", r"HKLM\SOFTWARE\ASIO"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut names: Vec<String> = text
                .lines()
                .filter_map(|l| {
                    l.trim()
                        .rsplit_once('\\')
                        .map(|(_, n)| n.trim().to_string())
                })
                .filter(|n| !n.is_empty() && !n.eq_ignore_ascii_case("ASIO"))
                .collect();
            names.sort();
            names.dedup();
            return names;
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asio_stays_off_when_not_built() {
        set_asio(true);
        assert_eq!(asio(), ASIO_BUILT);
        set_asio(false);
        assert!(!asio());
    }

    #[test]
    fn buffer_requests_stay_inside_the_device_range() {
        use cpal::{BufferSize, SupportedBufferSize};
        let range = SupportedBufferSize::Range {
            min: 128,
            max: 1024,
        };
        assert_eq!(pick_buffer(0, &range), BufferSize::Default);
        assert_eq!(pick_buffer(64, &range), BufferSize::Fixed(128));
        assert_eq!(pick_buffer(256, &range), BufferSize::Fixed(256));
        assert_eq!(pick_buffer(4096, &range), BufferSize::Fixed(1024));
        assert_eq!(
            pick_buffer(256, &SupportedBufferSize::Unknown),
            BufferSize::Default
        );
    }

    #[test]
    fn buffer_frames_are_capped() {
        set_buffer_frames(1_000_000);
        assert_eq!(buffer_frames(), 8192);
        assert!(!buffer_hint().is_empty());
        set_buffer_frames(0);
        assert_eq!(buffer_hint(), "");
    }
}
