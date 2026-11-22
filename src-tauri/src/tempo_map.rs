// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The session's tempo map: the tempo and time signature it starts with, and changes at
//! later bars. Beats here are quarter notes, as in MIDI clips; a bar of 6/8 is 3 of them.
//! The Rust copy of `src/stores/tempo.ts` (the core's own `Tempo` in `session/model.rs`
//! does the same sums for playback), tested against `shared-fixtures/keys-tempo-midi.json`.
//! Numbers are floats throughout, as they are in the interface.

use crate::js_compat::{is_js_space, js_number_to_string, js_round, js_trim};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TempoChange {
    pub id: String,
    /// Bars from the session start where it takes over (1 is the second bar).
    pub bar: f64,
    pub bpm: f64,
    pub beats_per_bar: f64,
    pub beat_unit: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tempo {
    /// Quarter notes per minute.
    pub bpm: f64,
    /// The top of the time signature.
    pub beats_per_bar: f64,
    /// The bottom of the time signature (4 when left out).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beat_unit: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changes: Option<Vec<TempoChange>>,
}

pub const BPM_MIN: f64 = 20.0;
pub const BPM_MAX: f64 = 400.0;
/// Bottoms a time signature can have.
pub const BEAT_UNITS: [f64; 6] = [1.0, 2.0, 4.0, 8.0, 16.0, 32.0];

/// One stretch of the map with one tempo and time signature.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Segment {
    /// Bars from the session start where it begins.
    pub bar: f64,
    /// Quarter notes from the start where it begins.
    pub beat: f64,
    /// Seconds from the start where it begins.
    pub seconds: f64,
    pub bpm: f64,
    pub beats_per_bar: f64,
    pub beat_unit: f64,
    /// The change it comes from; None for the start.
    pub id: Option<String>,
}

fn clamp_bpm(b: f64) -> f64 {
    if b.is_finite() {
        BPM_MAX.min(BPM_MIN.max(b))
    } else {
        120.0
    }
}

/// Quarter notes in one bar of a segment.
pub fn bar_beats(beats_per_bar: f64, beat_unit: Option<f64>) -> f64 {
    beats_per_bar.max(1.0) * 4.0 / beat_unit.unwrap_or(4.0).max(1.0)
}

/// Quarter notes in one beat of its time signature (0.5 in 6/8).
pub fn unit_beats(beat_unit: Option<f64>) -> f64 {
    4.0 / beat_unit.unwrap_or(4.0).max(1.0)
}

/// The stretches of the map in order. Never empty.
pub fn segments(t: &Tempo) -> Vec<Segment> {
    let mut out = vec![Segment {
        bar: 0.0,
        beat: 0.0,
        seconds: 0.0,
        bpm: clamp_bpm(t.bpm),
        beats_per_bar: t.beats_per_bar.max(1.0),
        beat_unit: t.beat_unit.unwrap_or(4.0).max(1.0),
        id: None,
    }];
    let mut prev = out[0].clone();
    for c in t.changes.as_deref().unwrap_or(&[]) {
        if c.bar <= prev.bar {
            continue;
        }
        let beat =
            prev.beat + (c.bar - prev.bar) * bar_beats(prev.beats_per_bar, Some(prev.beat_unit));
        let seg = Segment {
            bar: c.bar,
            beat,
            seconds: prev.seconds + ((beat - prev.beat) * 60.0) / prev.bpm,
            bpm: clamp_bpm(c.bpm),
            beats_per_bar: c.beats_per_bar.max(1.0),
            beat_unit: c.beat_unit.max(1.0),
            id: Some(c.id.clone()),
        };
        prev = seg.clone();
        out.push(seg);
    }
    out
}

fn last_where(segs: Vec<Segment>, ok: impl Fn(&Segment) -> bool) -> Segment {
    let mut it = segs.into_iter();
    let mut seg = it.next().expect("segments is never empty");
    for s in it {
        if !ok(&s) {
            break;
        }
        seg = s;
    }
    seg
}

pub fn segment_at_beat(t: &Tempo, beats: f64) -> Segment {
    last_where(segments(t), |s| s.beat <= beats)
}

pub fn segment_at_seconds(t: &Tempo, secs: f64) -> Segment {
    last_where(segments(t), |s| s.seconds <= secs)
}

pub fn segment_at_bar(t: &Tempo, bar: f64) -> Segment {
    last_where(segments(t), |s| s.bar <= bar)
}

/// Quarter notes to seconds.
pub fn beats_to_seconds(t: &Tempo, beats: f64) -> f64 {
    let b = beats.max(0.0);
    let s = segment_at_beat(t, b);
    s.seconds + ((b - s.beat) * 60.0) / s.bpm
}

/// Seconds to quarter notes.
pub fn seconds_to_beats(t: &Tempo, secs: f64) -> f64 {
    let s = segment_at_seconds(t, secs);
    s.beat + ((secs - s.seconds) * s.bpm) / 60.0
}

/// Where a bar starts (bars from 0), in quarter notes.
pub fn bar_start(t: &Tempo, bar: f64) -> f64 {
    let s = segment_at_bar(t, bar);
    s.beat + (bar - s.bar) * bar_beats(s.beats_per_bar, Some(s.beat_unit))
}

/// Where a time in quarter notes falls: the bar (from 0), how far into it in beats of its
/// time signature, and the stretch of the map it is in.
#[derive(Debug, Clone, PartialEq)]
pub struct BarAt {
    pub bar: f64,
    pub beat: f64,
    pub seg: Segment,
}

pub fn bar_at(t: &Tempo, beats: f64) -> BarAt {
    let seg = segment_at_beat(t, beats);
    let into = (beats - seg.beat).max(0.0);
    let per = bar_beats(seg.beats_per_bar, Some(seg.beat_unit));
    // A hair of slack so a bar line computed a little short counts as the new bar.
    let bars = (into / per + 1e-9).floor();
    BarAt {
        bar: seg.bar + bars,
        beat: ((into - bars * per) / unit_beats(Some(seg.beat_unit))).max(0.0),
        seg,
    }
}

/// The nearest bar line to a time in quarter notes, as a bar number from 0.
pub fn nearest_bar(t: &Tempo, beats: f64) -> f64 {
    let bar = bar_at(t, beats).bar;
    let a = bar_start(t, bar);
    let b = bar_start(t, bar + 1.0);
    if beats - a <= b - beats {
        bar
    } else {
        bar + 1.0
    }
}

/// "120" or "92.5".
pub fn bpm_text(bpm: f64) -> String {
    js_number_to_string(js_round(bpm * 100.0) / 100.0)
}

/// "4/4", "6/8".
pub fn meter_text(beats_per_bar: f64, beat_unit: Option<f64>) -> String {
    format!(
        "{}/{}",
        js_number_to_string(beats_per_bar),
        js_number_to_string(beat_unit.unwrap_or(4.0))
    )
}

/// Reads "6/8" or "3" (over 4). None when it is not a time signature.
pub fn parse_meter(text: &str) -> Option<(f64, f64)> {
    let s = js_trim(text);
    let digits = |s: &str| s.bytes().take_while(u8::is_ascii_digit).count();
    let n = digits(s);
    if n == 0 || n > 2 {
        return None;
    }
    let top: f64 = s[..n].parse().ok()?;
    let rest = s[n..].trim_start_matches(is_js_space);
    let bottom = if rest.is_empty() {
        4.0
    } else {
        let after = rest.strip_prefix('/')?.trim_start_matches(is_js_space);
        let m = digits(after);
        if m == 0 || m > 2 || m != after.len() {
            return None;
        }
        after.parse().ok()?
    };
    if !(1.0..=32.0).contains(&top) || !BEAT_UNITS.contains(&bottom) {
        return None;
    }
    Some((top, bottom))
}

/// New values for a stretch of the map; left as None, a value stays.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Values {
    pub bpm: Option<f64>,
    pub beats_per_bar: Option<f64>,
    pub beat_unit: Option<f64>,
}

/// The map with the stretch `seg_id` (None for the start, else a change) given new values.
pub fn edit_segment(t: &Tempo, seg_id: Option<&str>, v: &Values) -> Tempo {
    match seg_id {
        None => {
            let mut o = t.clone();
            if let Some(b) = v.bpm {
                o.bpm = b;
            }
            if let Some(b) = v.beats_per_bar {
                o.beats_per_bar = b;
            }
            if let Some(u) = v.beat_unit {
                o.beat_unit = Some(u);
            }
            o.changes = Some(t.changes.clone().unwrap_or_default());
            o
        }
        Some(id) => {
            let mut o = t.clone();
            let mut list = t.changes.clone().unwrap_or_default();
            for c in list.iter_mut().filter(|c| c.id == id) {
                if let Some(b) = v.bpm {
                    c.bpm = b;
                }
                if let Some(b) = v.beats_per_bar {
                    c.beats_per_bar = b;
                }
                if let Some(u) = v.beat_unit {
                    c.beat_unit = u;
                }
            }
            o.changes = Some(list);
            o
        }
    }
}

fn by_bar(list: &mut [TempoChange]) {
    list.sort_by(|a, b| {
        a.bar
            .partial_cmp(&b.bar)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// The map with a change at `bar` (from 1), taking the values in effect there; the existing
/// one if there is one already. Gives the map and the id of the change at that bar.
pub fn add_change(t: &Tempo, bar: f64, id: &str) -> (Tempo, String) {
    let at = js_round(bar).max(1.0);
    if let Some(existing) = t
        .changes
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .find(|c| c.bar == at)
    {
        return (t.clone(), existing.id.clone());
    }
    let s = segment_at_bar(t, at);
    let c = TempoChange {
        id: id.to_string(),
        bar: at,
        bpm: s.bpm,
        beats_per_bar: s.beats_per_bar,
        beat_unit: s.beat_unit,
    };
    let mut list = t.changes.clone().unwrap_or_default();
    list.push(c);
    by_bar(&mut list);
    (
        Tempo {
            changes: Some(list),
            ..t.clone()
        },
        id.to_string(),
    )
}

/// The map with a change moved to another bar (from 1). Moving onto another change replaces it.
pub fn move_change(t: &Tempo, id: &str, bar: f64) -> Tempo {
    let at = js_round(bar).max(1.0);
    let mut list: Vec<TempoChange> = t
        .changes
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter(|c| c.id == id || c.bar != at)
        .cloned()
        .map(|mut c| {
            if c.id == id {
                c.bar = at;
            }
            c
        })
        .collect();
    by_bar(&mut list);
    Tempo {
        changes: Some(list),
        ..t.clone()
    }
}

pub fn remove_change(t: &Tempo, id: &str) -> Tempo {
    let list: Vec<TempoChange> = t
        .changes
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter(|c| c.id != id)
        .cloned()
        .collect();
    Tempo {
        changes: Some(list),
        ..t.clone()
    }
}
