// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Time display helpers. Positions are sample counts. The default display is
//! Decimal: m:ss.ddd, or h:mm:ss.ddd past one hour. This is the Rust copy of
//! `src/lib/time.ts`; both are tested against `shared-fixtures/time.json`.

use crate::js_compat::js_round;
use std::fmt;

/// The sample rate was zero, negative, NaN or infinite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InvalidSampleRate(pub f64);

impl fmt::Display for InvalidSampleRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid sample rate: {}", self.0)
    }
}

impl std::error::Error for InvalidSampleRate {}

/// Convert a sample position to seconds.
pub fn samples_to_seconds(samples: f64, sample_rate: f64) -> Result<f64, InvalidSampleRate> {
    if !sample_rate.is_finite() || sample_rate <= 0.0 {
        return Err(InvalidSampleRate(sample_rate));
    }
    Ok(samples / sample_rate)
}

/// Format a sample position as Decimal time, for example 1:05.250.
pub fn format_decimal(samples: f64, sample_rate: f64) -> Result<String, InvalidSampleRate> {
    let sign = if samples < 0.0 { "-" } else { "" };
    // Round to whole milliseconds first so 59.9996 s shows as 1:00.000.
    let total_ms = js_round(samples_to_seconds(samples, sample_rate)?.abs() * 1000.0) as u64;
    let ms = total_ms % 1000;
    let total_seconds = total_ms / 1000;
    let s = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let m = total_minutes % 60;
    let h = total_minutes / 60;
    if h > 0 {
        Ok(format!("{sign}{h}:{m:02}:{s:02}.{ms:03}"))
    } else {
        Ok(format!("{sign}{m}:{s:02}.{ms:03}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_up_into_the_next_minute() {
        assert_eq!(format_decimal(2_879_981.0, 48_000.0).unwrap(), "1:00.000");
    }

    #[test]
    fn hours_show_when_past_one() {
        assert_eq!(
            format_decimal(158_760_000.0, 44_100.0).unwrap(),
            "1:00:00.000"
        );
    }

    #[test]
    fn bad_rates_are_errors() {
        for r in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(format_decimal(1.0, r).is_err());
        }
    }
}
