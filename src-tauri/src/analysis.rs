// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Number formats and scales behind the Analysis panel: levels, frequencies,
//! times, the log frequency axis, histogram bars and the Hum Cut settings a
//! hum finding calls for. This is the Rust copy of the matching functions in
//! `src/app/analysis.ts`; both are tested against
//! `shared-fixtures/analysis.json`. The finding texts (`findings`) stay in
//! TypeScript for now, since they group digits by the machine's locale.
//!
//! Like the TypeScript, the scales assume real numbers: a NaN position is not
//! handled the same way (Rust's `min` and `max` skip a NaN).

use crate::js_compat::{js_number_to_string, js_round, js_to_fixed};

/// "-12.3 dB", or "-inf dB" for silence and missing values.
pub fn fmt_db(v: Option<f64>, floor: f64) -> String {
    match v {
        Some(v) if v.is_finite() && v > floor => format!("{} dB", js_to_fixed(v, 1)),
        _ => "-inf dB".to_string(),
    }
}

/// The floor `fmt_db` uses when none is given.
pub const DEFAULT_DB_FLOOR: f64 = -150.0;

/// A sample value, six places.
pub fn fmt_sample(v: f64) -> String {
    js_to_fixed(v, 6)
}

/// Left edge of a point on a log frequency axis, 0 to 1.
pub fn log_x(hz: f64, low: f64, high: f64) -> f64 {
    let lo = low.ln();
    ((hz.max(low).ln() - lo) / (high.ln() - lo)).clamp(0.0, 1.0)
}

/// Vertical position for a level between `top` and `bottom` dB, 0 (top) to 1
/// (bottom).
pub fn db_y(db: f64, top: f64, bottom: f64) -> f64 {
    ((top - db) / (top - bottom)).clamp(0.0, 1.0)
}

/// Histogram bars scaled to the largest bin, 0 to 1, on a log scale so quiet
/// detail still shows.
pub fn histogram_bars(bins: &[f64]) -> Vec<f64> {
    let top = bins.iter().fold(1.0_f64, |a, &b| a.max(b));
    bins.iter()
        .map(|&n| {
            if n <= 0.0 {
                0.0
            } else {
                (1.0 + n).log10() / (1.0 + top).log10()
            }
        })
        .collect()
}

/// Level at the left edge of histogram bin `i`: 0 dB for the first.
pub fn bin_db(i: usize) -> f64 {
    -(i as f64)
}

/// The frequencies labelled on the axis.
pub const FREQ_TICKS: [f64; 10] = [
    20.0, 50.0, 100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 20000.0,
];

/// A frequency as labelled on the axis: "500", "1.5k".
pub fn fmt_hz(hz: f64) -> String {
    if hz >= 1000.0 {
        format!("{}k", js_number_to_string(hz / 1000.0))
    } else {
        js_number_to_string(hz)
    }
}

/// Seconds as m:ss.mmm for a list of places to look.
pub fn fmt_at(seconds: f64) -> String {
    let ms = js_round(seconds.max(0.0) * 1000.0);
    let m = (ms / 60000.0).floor();
    let s = ((ms % 60000.0) / 1000.0).floor();
    let rem = ms % 1000.0;
    format!("{m}:{s:02}.{rem:03}")
}

/// A tone found in a hum finding.
#[derive(Debug, Clone, PartialEq)]
pub struct HumTone {
    pub hz: f64,
    pub excess_db: f64,
}

/// A hum finding: the mains frequency, how strong it is and the tones that
/// stand out.
#[derive(Debug, Clone, PartialEq)]
pub struct Hum {
    pub base_hz: f64,
    pub strength_db: f64,
    pub tones: Vec<HumTone>,
}

/// An effect that deals with a finding, with the settings it calls for.
#[derive(Debug, Clone, PartialEq)]
pub struct HumCutFix {
    pub label: String,
    pub effect: String,
    pub mains: f64,
    pub harmonics: f64,
}

/// Hum Cut settings for a hum finding: the mains frequency it found and enough
/// overtones to reach the highest tone that stands out (at least two, at most
/// ten).
pub fn hum_cut_fix(h: &Hum) -> HumCutFix {
    let loud = h
        .tones
        .iter()
        .filter(|t| t.excess_db >= 8.0)
        .map(|t| js_round(t.hz / h.base_hz))
        .fold(1.0_f64, f64::max);
    let harmonics = loud.clamp(2.0, 10.0);
    HumCutFix {
        label: format!("Add Hum Cut at {} Hz", js_number_to_string(h.base_hz)),
        effect: "altaudio.hum_cut".to_string(),
        mains: if h.base_hz == 50.0 { 1.0 } else { 0.0 },
        harmonics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_and_missing_values_show_as_inf() {
        assert_eq!(fmt_db(None, DEFAULT_DB_FLOOR), "-inf dB");
        assert_eq!(fmt_db(Some(-150.0), DEFAULT_DB_FLOOR), "-inf dB");
        assert_eq!(fmt_db(Some(f64::NAN), DEFAULT_DB_FLOOR), "-inf dB");
        assert_eq!(fmt_db(Some(-12.34), DEFAULT_DB_FLOOR), "-12.3 dB");
    }

    #[test]
    fn times_round_up_into_the_next_minute() {
        assert_eq!(fmt_at(59.9996), "1:00.000");
        assert_eq!(fmt_at(-3.0), "0:00.000");
        assert_eq!(fmt_at(125.0005), "2:05.001");
    }

    #[test]
    fn hz_labels() {
        assert_eq!(fmt_hz(500.0), "500");
        assert_eq!(fmt_hz(1500.0), "1.5k");
        assert_eq!(fmt_hz(20000.0), "20k");
    }

    #[test]
    fn hum_harmonics_stay_between_two_and_ten() {
        let h = Hum {
            base_hz: 60.0,
            strength_db: 5.0,
            tones: vec![],
        };
        assert_eq!(hum_cut_fix(&h).harmonics, 2.0);
    }
}
