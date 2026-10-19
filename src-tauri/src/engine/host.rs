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

use std::sync::atomic::{AtomicBool, Ordering};

static USE_ASIO: AtomicBool = AtomicBool::new(false);

/// True when this build can talk to ASIO drivers.
pub const ASIO_BUILT: bool = cfg!(all(target_os = "windows", feature = "asio"));

/// Picks ASIO (true) or the system driver (false) from the next stream opened.
pub fn set_asio(on: bool) {
    USE_ASIO.store(on && ASIO_BUILT, Ordering::Relaxed);
}

pub fn asio() -> bool {
    USE_ASIO.load(Ordering::Relaxed)
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
    #[cfg(all(target_os = "windows", feature = "asio"))]
    {
        use cpal::traits::{DeviceTrait, HostTrait};
        if let Ok(h) = cpal::host_from_id(cpal::HostId::Asio) {
            if let Ok(devs) = h.output_devices() {
                return devs
                    .filter_map(|d| d.description().ok().map(|x| x.name().to_string()))
                    .collect();
            }
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
}
