// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Check Loudness rules: the targets, the broadcast regions, the pass or fail
//! verdict with its advice, and the export prediction. The Rust copy of the
//! pure parts of `src/app/loudness.ts`; both are tested against
//! `shared-fixtures/loudness-recording.json`. Reading the computer's time zone
//! and the custom target held in settings stay in TypeScript.

use crate::js_compat::{js_number_to_string, js_round, js_to_fixed};

/// Within this many LU of the target passes; within twice it is close.
pub const TOLERANCE_LU: f64 = 1.0;
pub const DEFAULT_REGION: &str = "us";

/// What a scan of the mix measured (the fields the verdict reads).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Report {
    pub integrated: f64,
    pub integrated_mono: f64,
    pub true_peak: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub lufs: f64,
    /// Measure the mix folded to mono (for mono shows).
    pub mono: bool,
    /// Highest allowed true peak, dBTP.
    pub peak: f64,
}

pub struct Region {
    pub id: &'static str,
    pub name: &'static str,
    pub standard: &'static str,
    pub lufs: f64,
    pub peak: f64,
}

pub const REGIONS: [Region; 6] = [
    Region {
        id: "us",
        name: "United States",
        standard: "ATSC A/85",
        lufs: -24.0,
        peak: -2.0,
    },
    Region {
        id: "canada",
        name: "Canada",
        standard: "ATSC A/85",
        lufs: -24.0,
        peak: -2.0,
    },
    Region {
        id: "europe",
        name: "Europe",
        standard: "EBU R128",
        lufs: -23.0,
        peak: -1.0,
    },
    Region {
        id: "uk",
        name: "United Kingdom",
        standard: "EBU R128",
        lufs: -23.0,
        peak: -1.0,
    },
    Region {
        id: "australia",
        name: "Australia and New Zealand",
        standard: "OP-59",
        lufs: -24.0,
        peak: -2.0,
    },
    Region {
        id: "japan",
        name: "Japan",
        standard: "ARIB TR-B32",
        lufs: -24.0,
        peak: -1.0,
    },
];

const CANADIAN_ZONES: [&str; 30] = [
    "Toronto",
    "Vancouver",
    "Edmonton",
    "Winnipeg",
    "Halifax",
    "St_Johns",
    "Regina",
    "Montreal",
    "Moncton",
    "Glace_Bay",
    "Goose_Bay",
    "Iqaluit",
    "Whitehorse",
    "Yellowknife",
    "Dawson",
    "Dawson_Creek",
    "Fort_Nelson",
    "Cambridge_Bay",
    "Inuvik",
    "Rankin_Inlet",
    "Resolute",
    "Swift_Current",
    "Thunder_Bay",
    "Nipigon",
    "Rainy_River",
    "Atikokan",
    "Blanc-Sablon",
    "Pangnirtung",
    "Creston",
    "Coral_Harbour",
];

/// The broadcast region a time zone suggests, for when none has been chosen.
pub fn region_for_zone(zone: Option<&str>) -> &'static str {
    let z = zone.unwrap_or("");
    let mut parts = z.split('/');
    let area = parts.next().unwrap_or("");
    let place = parts.next().unwrap_or("");
    if area == "America" {
        return if CANADIAN_ZONES.contains(&place) {
            "canada"
        } else {
            DEFAULT_REGION
        };
    }
    if area == "Europe" {
        return if ["London", "Belfast", "Jersey", "Guernsey", "Isle_of_Man"].contains(&place) {
            "uk"
        } else {
            "europe"
        };
    }
    if area == "Australia" || z == "Pacific/Auckland" || z == "Pacific/Chatham" {
        return "australia";
    }
    if z == "Asia/Tokyo" {
        return "japan";
    }
    DEFAULT_REGION
}

/// The region with this id; the first (United States) when unknown.
pub fn region_by_id(id: Option<&str>) -> &'static Region {
    REGIONS
        .iter()
        .find(|r| Some(r.id) == id)
        .unwrap_or(&REGIONS[0])
}

/// The targets, with broadcast set by the chosen region and the user's own last.
pub fn targets_for(region_id: &str, custom_lufs: f64, custom_peak: f64) -> Vec<Target> {
    let r = region_by_id(Some(region_id));
    let t = |id: &str, name: String, lufs: f64, mono: bool, peak: f64| Target {
        id: id.to_string(),
        name,
        lufs,
        mono,
        peak,
    };
    vec![
        t(
            "podcast-stereo",
            "Podcast, stereo (-16 LUFS)".to_string(),
            -16.0,
            false,
            -1.0,
        ),
        t(
            "podcast-mono",
            "Podcast, mono (-19 LUFS)".to_string(),
            -19.0,
            true,
            -1.0,
        ),
        t(
            "spotify",
            "Louder platforms (-14 LUFS)".to_string(),
            -14.0,
            false,
            -1.0,
        ),
        t(
            "broadcast",
            format!(
                "Broadcast, {} ({} LUFS)",
                r.name,
                js_number_to_string(r.lufs)
            ),
            r.lufs,
            false,
            r.peak,
        ),
        t(
            "custom",
            format!(
                "Custom ({} LUFS, {} dBTP)",
                js_number_to_string(custom_lufs),
                js_number_to_string(custom_peak)
            ),
            custom_lufs,
            false,
            custom_peak,
        ),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Pass,
    Close,
    Fail,
}

impl Grade {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Close => "close",
            Self::Fail => "fail",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub measured: f64,
    /// Loudness minus target, LU. Positive is too loud.
    pub offset: f64,
    pub loudness: Grade,
    pub peak: Grade,
    /// Master gain change that would hit the target, dB.
    pub gain_to_target: f64,
    /// True when that gain would keep true peaks within the limit.
    pub gain_is_safe: bool,
    pub advice: String,
}

fn tenth(x: f64) -> f64 {
    js_round(x * 10.0) / 10.0
}

fn fixed1(x: f64) -> String {
    js_to_fixed(x, 1)
}

fn grade_of(offset: f64) -> Grade {
    if offset.abs() <= TOLERANCE_LU {
        Grade::Pass
    } else if offset.abs() <= 2.0 * TOLERANCE_LU {
        Grade::Close
    } else {
        Grade::Fail
    }
}

const SILENT: &str = "The mix is silent or too short to measure.";

/// The pass or fail verdict for a scan against a target.
pub fn evaluate(r: Report, t: &Target) -> Verdict {
    let measured = if t.mono {
        r.integrated_mono
    } else {
        r.integrated
    };
    if !measured.is_finite() {
        return Verdict {
            measured,
            offset: 0.0,
            loudness: Grade::Fail,
            peak: Grade::Pass,
            gain_to_target: 0.0,
            gain_is_safe: false,
            advice: SILENT.to_string(),
        };
    }
    let offset = tenth(measured - t.lufs);
    let loudness = grade_of(offset);
    let peak = if r.true_peak <= t.peak {
        Grade::Pass
    } else if r.true_peak <= t.peak + 0.5 {
        Grade::Close
    } else {
        Grade::Fail
    };
    let gain_to_target = tenth(t.lufs - measured);
    let peak_after = r.true_peak + gain_to_target;
    let gain_is_safe = peak_after <= t.peak;
    let advice = if loudness == Grade::Pass && peak == Grade::Pass {
        "Ready to publish at this target.".to_string()
    } else if loudness != Grade::Pass && gain_is_safe {
        format!(
            "{} the Master by {} dB to hit the target.",
            if gain_to_target > 0.0 {
                "Raise"
            } else {
                "Lower"
            },
            fixed1(gain_to_target.abs())
        )
    } else if loudness != Grade::Pass {
        format!(
            "Raising the Master {} dB would push peaks to {} dBTP. Limit the voice tracks harder first (a limiter around -3 to -6 dB on vocals), then check again.",
            fixed1(gain_to_target),
            fixed1(peak_after)
        )
    } else {
        format!(
            "Peaks reach {} dBTP, over the {} dBTP limit. Lower the loudest moments or limit them, then check again.",
            fixed1(r.true_peak),
            js_number_to_string(t.peak)
        )
    };
    Verdict {
        measured,
        offset,
        loudness,
        peak,
        gain_to_target,
        gain_is_safe,
        advice,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prediction {
    /// Gain the export applies to reach the target, dB. 0 when left as mixed.
    pub gain: f64,
    pub loudness: f64,
    pub true_peak: f64,
    /// The gain stopped short of the target to keep peaks under its limit.
    pub limited_by_peak: bool,
    /// How far the file lands from the target, LU. None without a target.
    pub offset: Option<f64>,
    pub grade: Grade,
    pub summary: String,
}

/// What an export will come out as, from a scan of the mix. `mono` is true when
/// the file is written as mono.
pub fn predict_export(r: Report, t: Option<&Target>, mono: bool) -> Prediction {
    let measured = if mono {
        r.integrated_mono
    } else {
        r.integrated
    };
    if !measured.is_finite() {
        return Prediction {
            gain: 0.0,
            loudness: measured,
            true_peak: r.true_peak,
            limited_by_peak: false,
            offset: None,
            grade: Grade::Fail,
            summary: SILENT.to_string(),
        };
    }
    let Some(t) = t else {
        return Prediction {
            gain: 0.0,
            loudness: measured,
            true_peak: r.true_peak,
            limited_by_peak: false,
            offset: None,
            grade: Grade::Pass,
            summary: format!(
                "Exports as mixed: {} LUFS, peaks {} dBTP.",
                fixed1(measured),
                fixed1(r.true_peak)
            ),
        };
    };
    let want = t.lufs - measured;
    let room = t.peak - r.true_peak;
    let limited_by_peak = want > room;
    let gain = tenth(want.min(room));
    let loudness = measured + gain;
    let true_peak = r.true_peak + gain;
    let offset = tenth(loudness - t.lufs);
    let grade = grade_of(offset);
    let mv = if gain.abs() < 0.05 {
        "left as it is".to_string()
    } else {
        format!(
            "{} {} dB",
            if gain > 0.0 { "raised" } else { "lowered" },
            fixed1(gain.abs())
        )
    };
    let base = format!(
        "Measures {} LUFS with peaks at {} dBTP. The export is {mv} and lands at {} LUFS, peaks {} dBTP.",
        fixed1(measured),
        fixed1(r.true_peak),
        fixed1(loudness),
        fixed1(true_peak)
    );
    let why = if limited_by_peak {
        format!(
            " That is {} LU under the {} target: the peaks reach the {} dBTP limit first. Limit the loudest moments to get closer.",
            fixed1(offset.abs()),
            js_number_to_string(t.lufs),
            js_number_to_string(t.peak)
        )
    } else if grade == Grade::Pass {
        " On target.".to_string()
    } else {
        String::new()
    };
    Prediction {
        gain,
        loudness,
        true_peak,
        limited_by_peak,
        offset: Some(offset),
        grade,
        summary: format!("{base}{why}"),
    }
}
