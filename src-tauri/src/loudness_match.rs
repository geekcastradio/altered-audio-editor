// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The arithmetic behind Loudness Match: the gain change that brings a track
//! or clip to a loudness target, its loudest sample, its true peak or its
//! average level, and the warning when that change would push peaks too high.
//! This is the Rust copy of the pure rules in `src/app/loudness-match.ts`;
//! both are tested against `shared-fixtures/loudness-match.json`. Reading the
//! session to build the items and turning rows into clip gain changes stay in
//! TypeScript until the session moves.

use crate::js_compat::js_round;

/// Clip gain limits, matching the Clip Gain command.
pub const GAIN_MIN: f64 = -96.0;
pub const GAIN_MAX: f64 = 24.0;

/// Default tolerance in dB: an item already this close to the goal is left alone.
pub const DEFAULT_TOLERANCE: f64 = 0.5;

/// Where Peak, True Peak and RMS matching start, in dB below full scale.
pub const BASIS_LEVEL_PEAK: f64 = -1.0;
pub const BASIS_LEVEL_TRUE_PEAK: f64 = -1.0;
pub const BASIS_LEVEL_RMS: f64 = -18.0;

/// What is being matched: a track or a group of clips that change together.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchItem {
    pub key: String,
    pub label: String,
    pub clip_ids: Vec<String>,
}

/// One item with what was measured and the change that would be made.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchRow {
    pub item: MatchItem,
    /// The level the match measures (loudness, peak, true peak or RMS).
    pub measured: f64,
    pub true_peak: f64,
    /// Gain change in dB, rounded to 0.1.
    pub change: f64,
    pub peak_after: f64,
    /// False when the item is silent or too short to measure.
    pub measurable: bool,
    /// True when the change would push peaks over the limit.
    pub peak_warning: bool,
}

/// What a loudness measurement reports, as far as matching needs it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoudnessReading {
    pub integrated: f64,
    pub integrated_mono: f64,
    pub true_peak: f64,
}

/// A loudness target: its goal, whether it measures the mix folded to mono,
/// and the highest true peak it allows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoudnessTarget {
    pub lufs: f64,
    pub mono: bool,
    pub peak: f64,
}

/// Peak and average level of one channel, in dB below full scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelLevels {
    pub peak_db: f64,
    pub rms_db: f64,
}

/// The levels of both channels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StereoLevels {
    pub left: ChannelLevels,
    pub right: ChannelLevels,
}

/// Which level a peak or RMS match works on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelBasis {
    Peak,
    Rms,
}

/// A value rounded to 0.1, as JavaScript's `Math.round(x * 10) / 10`.
fn round1(x: f64) -> f64 {
    js_round(x * 10.0) / 10.0
}

/// The gain change to move `measured` to `goal`, rounded to 0.1 dB, or 0 when
/// already within `tolerance`.
pub fn change_for(goal: f64, measured: f64, tolerance: f64) -> f64 {
    if (goal - measured).abs() <= tolerance.max(0.0) {
        return 0.0;
    }
    round1(goal - measured)
}

fn unmeasurable(item: &MatchItem, measured: f64, true_peak: f64, peak_after: f64) -> MatchRow {
    MatchRow {
        item: item.clone(),
        measured,
        true_peak,
        change: 0.0,
        peak_after,
        measurable: false,
        peak_warning: false,
    }
}

/// The row for matching to a loudness target.
pub fn match_row(
    item: &MatchItem,
    r: &LoudnessReading,
    t: &LoudnessTarget,
    tolerance: f64,
) -> MatchRow {
    let measured = if t.mono {
        r.integrated_mono
    } else {
        r.integrated
    };
    if !measured.is_finite() {
        return unmeasurable(item, measured, r.true_peak, r.true_peak);
    }
    let change = change_for(t.lufs, measured, tolerance);
    let peak_after = round1(r.true_peak + change);
    MatchRow {
        item: item.clone(),
        measured,
        true_peak: r.true_peak,
        change,
        peak_after,
        measurable: true,
        peak_warning: change != 0.0 && peak_after > t.peak,
    }
}

/// Loudest sample and average (RMS) level of both channels together, in dB
/// below full scale. Returns `(peak_db, rms_db)`.
pub fn levels_of(r: &StereoLevels) -> (f64, f64) {
    let peak_db = r.left.peak_db.max(r.right.peak_db);
    let power = (10f64.powf(r.left.rms_db / 10.0) + 10f64.powf(r.right.rms_db / 10.0)) / 2.0;
    let rms_db = if power > 0.0 {
        10.0 * power.log10()
    } else {
        f64::NEG_INFINITY
    };
    (peak_db, rms_db)
}

/// The row for matching by peak or RMS: the change that puts the item at
/// `level` dB. A change that would push the loudest sample past full scale is
/// flagged.
pub fn level_row(
    item: &MatchItem,
    basis: LevelBasis,
    level: f64,
    r: &StereoLevels,
    tolerance: f64,
) -> MatchRow {
    let (peak_db, rms_db) = levels_of(r);
    let measured = match basis {
        LevelBasis::Peak => peak_db,
        LevelBasis::Rms => rms_db,
    };
    if !measured.is_finite() || measured <= -120.0 {
        return unmeasurable(item, measured, peak_db, peak_db);
    }
    let change = change_for(level, measured, tolerance);
    let peak_after = round1(peak_db + change);
    MatchRow {
        item: item.clone(),
        measured,
        true_peak: peak_db,
        change,
        peak_after,
        measurable: true,
        peak_warning: change != 0.0 && peak_after > 0.0,
    }
}

/// The row for matching by true peak (the loudest point of the reconstructed
/// waveform, which can sit above the loudest sample): the change that puts it
/// at `level` dBTP.
pub fn true_peak_row(
    item: &MatchItem,
    level: f64,
    r: &LoudnessReading,
    tolerance: f64,
) -> MatchRow {
    let measured = r.true_peak;
    if !measured.is_finite() || measured <= -120.0 {
        return unmeasurable(item, measured, measured, measured);
    }
    let change = change_for(level, measured, tolerance);
    let peak_after = round1(measured + change);
    MatchRow {
        item: item.clone(),
        measured,
        true_peak: measured,
        change,
        peak_after,
        measurable: true,
        peak_warning: change != 0.0 && peak_after > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item() -> MatchItem {
        MatchItem {
            key: "t".into(),
            label: "Track".into(),
            clip_ids: vec!["a".into()],
        }
    }

    #[test]
    fn already_close_is_left_alone() {
        assert_eq!(change_for(-16.0, -16.4, DEFAULT_TOLERANCE), 0.0);
        assert_eq!(change_for(-16.0, -17.0, DEFAULT_TOLERANCE), 1.0);
        assert_eq!(change_for(-16.0, -17.0, -3.0), 1.0);
    }

    #[test]
    fn silence_cannot_be_matched() {
        let r = LoudnessReading {
            integrated: f64::NEG_INFINITY,
            integrated_mono: f64::NEG_INFINITY,
            true_peak: f64::NEG_INFINITY,
        };
        let t = LoudnessTarget {
            lufs: -16.0,
            mono: false,
            peak: -1.0,
        };
        let row = match_row(&item(), &r, &t, 0.0);
        assert!(!row.measurable && row.change == 0.0);
    }

    #[test]
    fn a_boost_that_pushes_peaks_over_is_flagged() {
        let r = LoudnessReading {
            integrated: -20.0,
            integrated_mono: -23.0,
            true_peak: -3.0,
        };
        let t = LoudnessTarget {
            lufs: -16.0,
            mono: false,
            peak: -1.0,
        };
        let row = match_row(&item(), &r, &t, 0.0);
        assert_eq!(
            (row.change, row.peak_after, row.peak_warning),
            (4.0, 1.0, true)
        );
    }
}
