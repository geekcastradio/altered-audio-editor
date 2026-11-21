// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Audiobook delivery check, in Rust.
//!
//! This mirrors `src/app/delivery.ts`. Both are tested against
//! `shared-fixtures/delivery.json`, so the two copies cannot drift. When the
//! interface moves to Rust, the TypeScript copy goes away.
//!
//! The three numbers narration is judged on besides file format: average level
//! (RMS) between -23 and -18 dBFS, peaks no higher than -3 dBFS, and a noise
//! floor no louder than -60 dBFS. A pass is a good sign, not a guarantee.

use serde::Serialize;

pub const RMS_MIN: f64 = -23.0;
pub const RMS_MAX: f64 = -18.0;
pub const PEAK_MAX: f64 = -3.0;
pub const FLOOR_MAX: f64 = -60.0;

/// The middle of the RMS band, where gain advice aims.
const RMS_AIM: f64 = (RMS_MIN + RMS_MAX) / 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    Pass,
    Close,
    Fail,
}

/// The numbers the check reads from a loudness report. A value that could not
/// be measured is `None`; a peak that is not finite counts as not measured.
#[derive(Clone, Copy, Debug, Default)]
pub struct Report {
    pub rms: Option<f64>,
    pub noise_floor: Option<f64>,
    pub sample_peak: Option<f64>,
    pub silent_seconds: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Rule {
    pub id: &'static str,
    pub name: &'static str,
    /// The measured value in dB, or None when it could not be measured.
    pub value: Option<f64>,
    /// The limit in words, for the card.
    pub limit: String,
    pub grade: Grade,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Verdict {
    pub rules: Vec<Rule>,
    /// Every rule passes and the file has no digital silence.
    pub pass: bool,
    /// Master gain that would put the average level mid-band, dB; None when not measurable.
    #[serde(rename = "gainToTarget")]
    pub gain_to_target: Option<f64>,
    /// That gain keeps peaks at or under the limit.
    #[serde(rename = "gainIsSafe")]
    pub gain_is_safe: bool,
    /// What to do next, most important first.
    pub advice: Vec<String>,
}

/// JavaScript's `Math.round`: halves go toward positive infinity.
fn js_round(x: f64) -> f64 {
    let f = x.floor();
    if x - f >= 0.5 {
        f + 1.0
    } else {
        f
    }
}

/// JavaScript's `Number.prototype.toFixed(1)`: the nearest tenth of the exact
/// value, and an exact tie goes up in size (away from zero for negatives).
fn js_to_fixed1(x: f64) -> String {
    if x < 0.0 {
        return format!("-{}", js_to_fixed1(-x));
    }
    let exact = format!("{x:.60}");
    let (whole, frac) = exact.split_once('.').unwrap_or((&exact, ""));
    let mut digits = frac.bytes();
    let tenth = digits.next().map_or(0, |d| u128::from(d - b'0'));
    let rest: Vec<u8> = digits.collect();
    let tie = rest.first() == Some(&b'5') && rest[1..].iter().all(|&d| d == b'0');
    if !tie {
        return format!("{x:.1}");
    }
    let n = whole.parse::<u128>().unwrap_or(0) * 10 + tenth + 1;
    format!("{}.{}", n / 10, n % 10)
}

fn round1(v: f64) -> f64 {
    js_round(v * 10.0) / 10.0
}

/// Grades a value against a band: inside passes, within 1 dB outside is close.
fn band(v: f64, lo: f64, hi: f64) -> Grade {
    if v >= lo && v <= hi {
        Grade::Pass
    } else if v >= lo - 1.0 && v <= hi + 1.0 {
        Grade::Close
    } else {
        Grade::Fail
    }
}

fn graded(value: Option<f64>, lo: f64, hi: f64) -> Grade {
    value.map_or(Grade::Fail, |v| band(v, lo, hi))
}

pub fn acx_check(r: &Report) -> Verdict {
    let rms = r.rms;
    let floor = r.noise_floor;
    let peak = r.sample_peak.filter(|p| p.is_finite());
    let rules = vec![
        Rule {
            id: "rms",
            name: "Average level (RMS)",
            value: rms,
            limit: format!("{RMS_MIN} to {RMS_MAX} dBFS"),
            grade: graded(rms, RMS_MIN, RMS_MAX),
        },
        Rule {
            id: "peak",
            name: "Peak",
            value: peak,
            limit: format!("{PEAK_MAX} dBFS or lower"),
            grade: graded(peak, f64::NEG_INFINITY, PEAK_MAX),
        },
        Rule {
            id: "floor",
            name: "Noise floor",
            value: floor,
            limit: format!("{FLOOR_MAX} dBFS or lower"),
            grade: graded(floor, f64::NEG_INFINITY, FLOOR_MAX),
        },
    ];
    let silent = r.silent_seconds.unwrap_or(0.0) > 0.05;
    let gain = rms.map(|v| round1(RMS_AIM - v));
    let gain_is_safe = matches!((gain, peak), (Some(g), Some(p)) if p + g <= PEAK_MAX);
    let grade_of = |id: &str| rules.iter().find(|x| x.id == id).map(|x| x.grade);
    let mut advice: Vec<String> = Vec::new();
    match (rms, gain) {
        (Some(_), Some(g)) if grade_of("rms") != Some(Grade::Pass) => {
            let amount = js_to_fixed1(g.abs());
            if gain_is_safe {
                let dir = if g > 0.0 { "Raise" } else { "Lower" };
                advice.push(format!(
                    "{dir} the Master {amount} dB to bring the average level to about {RMS_AIM} dBFS."
                ));
            } else {
                let dir = if g > 0.0 { "Raising" } else { "Lowering" };
                advice.push(format!(
                    "{dir} the level {amount} dB would push peaks over {PEAK_MAX} dBFS. Compress and limit the voice first (a Voice Chain with a limiter near {} dBFS), then check again.",
                    PEAK_MAX - 0.5
                ));
            }
        }
        (None, _) => advice.push("The file is silent or too short to measure.".to_string()),
        _ => {}
    }
    if let Some(p) = peak.filter(|_| grade_of("peak") != Some(Grade::Pass)) {
        advice.push(format!(
            "Peaks reach {} dBFS. Limit the loudest moments to {} dBFS or lower. Lowering peaks also lowers the average, so check the level again.",
            js_to_fixed1(p),
            PEAK_MAX - 0.5
        ));
    }
    match floor {
        None => advice.push("No quiet gap was found to measure the noise floor. Leave a second or so of room tone before the first line and after the last.".to_string()),
        Some(f) if grade_of("floor") != Some(Grade::Pass) => advice.push(format!(
            "The room tone sits at {} dBFS. Learn a noise print from a quiet stretch and use the Noise Reducer, or re-record somewhere quieter.",
            js_to_fixed1(f)
        )),
        _ => {}
    }
    if silent {
        advice.push("The file has stretches of digital silence. Replace them with room tone (Effects > Fill with Room Tone); silence that is truly empty is rejected.".to_string());
    }
    if advice.is_empty() {
        advice.push("Levels meet the audiobook numbers. Export as MP3, 192 kbps constant bit rate, 44.1 kHz.".to_string());
    }
    let pass = rules.iter().all(|x| x.grade == Grade::Pass) && !silent;
    Verdict {
        rules,
        pass,
        gain_to_target: gain,
        gain_is_safe,
        advice,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_round_sends_halves_up() {
        assert_eq!(js_round(2.5), 3.0);
        assert_eq!(js_round(-2.5), -2.0);
        assert_eq!(js_round(-7.5), -7.0);
        assert_eq!(js_round(0.4), 0.0);
        assert_eq!(js_round(-0.6), -1.0);
    }

    #[test]
    fn js_to_fixed1_matches_javascript() {
        assert_eq!(js_to_fixed1(0.25), "0.3");
        assert_eq!(js_to_fixed1(2.25), "2.3");
        assert_eq!(js_to_fixed1(-2.25), "-2.3");
        assert_eq!(js_to_fixed1(59.25), "59.3");
        assert_eq!(js_to_fixed1(3.0), "3.0");
        assert_eq!(js_to_fixed1(5.5), "5.5");
        assert_eq!(js_to_fixed1(0.05), "0.1");
        assert_eq!(js_to_fixed1(0.04), "0.0");
        assert_eq!(js_to_fixed1(-0.04), "-0.0");
        assert_eq!(js_to_fixed1(9.95), "9.9");
    }
}
