// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Changing the pitch of a voice over time: automatic correction toward a
//! key and scale (Pitch Tune), and a pitch bend drawn over a selection
//! (Pitch Curve). Both share one engine, a pitch-synchronous overlap-add:
//! the pitch is tracked, the audio is cut into one-period grains around
//! each pitch mark, and the grains are laid back at the new spacing. Each
//! grain keeps its own spectrum, so the voice keeps its character (the
//! formants stay where they were) while the pitch moves. Parts with no
//! clear pitch (breaths, consonants, silence) are left as they are.
//!
//! It is made for one voice at a time and for small corrections. Large
//! moves (over about 7 semitones) and mixes of several sounds do not work
//! well and are limited. Written from the textbook method, with the
//! pitch tracker the Show Pitch line already uses.

use super::spectral::pitch_track;

/// How often the pitch is read, in seconds.
const HOP_SECONDS: f64 = 0.010;
/// The shortest stretch of clear pitch that is worked on, in tracker frames.
const MIN_RUN: usize = 4;
/// Fade between the original and the reworked audio at the ends of a stretch, in seconds.
const EDGE_SECONDS: f64 = 0.006;
/// The most a bend or a correction may move the pitch, in cents.
pub const MAX_CENTS: f64 = 700.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scale {
    Chromatic,
    Major,
    Minor,
    MajorPentatonic,
    MinorPentatonic,
}

impl Scale {
    /// Semitones above the key that belong to the scale.
    fn steps(self) -> &'static [u8] {
        match self {
            Scale::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
            Scale::Minor => &[0, 2, 3, 5, 7, 8, 10],
            Scale::MajorPentatonic => &[0, 2, 4, 7, 9],
            Scale::MinorPentatonic => &[0, 3, 5, 7, 10],
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuneSettings {
    /// The key's note, 0 (C) to 11 (B).
    pub key: u8,
    pub scale: Scale,
    /// 0 to 1: how much of the distance to the nearest scale note is corrected.
    pub strength: f64,
    /// How fast the correction follows, in milliseconds (0 is instant, robotic).
    pub speed_ms: f64,
    /// Notes further off than this are left alone, in cents.
    pub max_cents: f64,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Edit {
    Tune(TuneSettings),
    /// Points of (position in the selection 0 to 1, shift in cents), joined by straight lines.
    Bend {
        points: Vec<(f64, f64)>,
    },
}

/// What was done, for the interface to report.
#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    /// Tracker frames in the selection.
    pub frames: usize,
    /// Frames with a clear pitch.
    pub voiced: usize,
    /// Frames whose pitch was moved by more than 5 cents.
    pub moved: usize,
}

/// Midi note number (fractional) of a frequency.
fn midi(hz: f64) -> f64 {
    69.0 + 12.0 * (hz / 440.0).log2()
}

/// The pitch per tracker frame in Hz (0 where there is none), median-filtered so a single
/// wrong octave does not move anything, and with very short stretches dropped.
pub fn contour(mono: &[f32], rate: u32) -> (Vec<f64>, f64) {
    let hop = (f64::from(rate) * HOP_SECONDS).round().max(1.0);
    let columns = ((mono.len() as f64 / hop).floor() as usize).max(1);
    let real_hop = mono.len() as f64 / columns as f64;
    let raw = pitch_track(mono, 0, rate, 0, mono.len() as u64, columns);
    let mut f0: Vec<f64> = raw.iter().map(|&v| f64::from(v)).collect();
    // Median over five frames of the voiced ones nearby.
    let src = f0.clone();
    for (i, o) in f0.iter_mut().enumerate() {
        if src[i] <= 0.0 {
            continue;
        }
        let lo = i.saturating_sub(2);
        let hi = (i + 3).min(src.len());
        let mut w: Vec<f64> = src[lo..hi].iter().copied().filter(|v| *v > 0.0).collect();
        w.sort_by(|a, b| a.total_cmp(b));
        *o = w[w.len() / 2];
    }
    // Drop runs that are too short to be a voice.
    let mut i = 0;
    while i < f0.len() {
        if f0[i] <= 0.0 {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < f0.len() && f0[j] > 0.0 {
            j += 1;
        }
        if j - i < MIN_RUN {
            f0[i..j].fill(0.0);
        }
        i = j;
    }
    (f0, real_hop)
}

/// The shift (as a pitch ratio) wanted for each frame, 1.0 where nothing is to change.
fn ratios(f0: &[f64], hop_ms: f64, edit: &Edit) -> Vec<f64> {
    let mut out = vec![1.0; f0.len()];
    match edit {
        Edit::Tune(t) => {
            let strength = t.strength.clamp(0.0, 1.0);
            let max = t.max_cents.clamp(0.0, MAX_CENTS);
            // One-pole smoothing of the shift, in cents, restarted at each stretch of pitch.
            let a = if t.speed_ms <= 0.0 {
                0.0
            } else {
                (-hop_ms / t.speed_ms).exp()
            };
            let mut smooth: Option<f64> = None;
            for (i, &hz) in f0.iter().enumerate() {
                if hz <= 0.0 {
                    smooth = None;
                    continue;
                }
                let m = midi(hz);
                let rel = (m - f64::from(t.key)).rem_euclid(12.0);
                let base = m - rel;
                // The nearest scale note, looking one octave either way.
                let target = [-12.0, 0.0, 12.0]
                    .iter()
                    .flat_map(|o| {
                        t.scale
                            .steps()
                            .iter()
                            .map(move |s| base + o + f64::from(*s))
                    })
                    .min_by(|x, y| (x - m).abs().total_cmp(&(y - m).abs()))
                    .unwrap_or(m);
                let mut want = (target - m) * 100.0 * strength;
                if want.abs() > max {
                    want = 0.0;
                }
                let s = match smooth {
                    None => want,
                    Some(prev) => a * prev + (1.0 - a) * want,
                };
                smooth = Some(s);
                out[i] = 2f64.powf(s / 1200.0);
            }
        }
        Edit::Bend { points } => {
            let mut pts: Vec<(f64, f64)> = points
                .iter()
                .map(|(x, c)| (x.clamp(0.0, 1.0), c.clamp(-MAX_CENTS, MAX_CENTS)))
                .collect();
            pts.sort_by(|a, b| a.0.total_cmp(&b.0));
            let n = f0.len().max(1) as f64;
            for (i, &hz) in f0.iter().enumerate() {
                if hz <= 0.0 || pts.is_empty() {
                    continue;
                }
                let x = (i as f64 + 0.5) / n;
                let cents = match pts.iter().position(|p| p.0 >= x) {
                    None => pts[pts.len() - 1].1,
                    Some(0) => pts[0].1,
                    Some(k) => {
                        let (x0, c0) = pts[k - 1];
                        let (x1, c1) = pts[k];
                        if x1 > x0 {
                            c0 + (c1 - c0) * (x - x0) / (x1 - x0)
                        } else {
                            c1
                        }
                    }
                };
                out[i] = 2f64.powf(cents / 1200.0);
            }
        }
    }
    out
}

fn hann(len: usize) -> Vec<f32> {
    (0..len)
        .map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * (i as f32 + 0.5) / len as f32).cos())
        .collect()
}

/// Pitch marks inside samples `a..b`: one near each period's strongest peak.
fn marks(mono: &[f32], a: usize, b: usize, period_at: &dyn Fn(usize) -> f64) -> Vec<usize> {
    let peak = |lo: usize, hi: usize| -> usize {
        let hi = hi.min(mono.len()).max(lo + 1);
        (lo..hi)
            .max_by(|&x, &y| mono[x].abs().total_cmp(&mono[y].abs()))
            .unwrap_or(lo)
    };
    let mut out = Vec::new();
    let t0 = period_at(a).max(2.0);
    let mut m = peak(a, a + t0 as usize);
    while m < b {
        out.push(m);
        let t = period_at(m).max(2.0);
        let want = m as f64 + t;
        let slack = (t * 0.25).max(1.0);
        let lo = (want - slack).max(0.0) as usize;
        let next = peak(lo, (want + slack) as usize + 1);
        if next <= m {
            break;
        }
        m = next;
    }
    out
}

/// Applies `edit` to `audio`, `n` channels interleaved. The pitch is read from the average of
/// the channels, and the same grains are moved in every channel. The length does not change.
pub fn apply(audio: &[f32], n: usize, rate: u32, edit: &Edit) -> (Vec<f32>, Report) {
    let n = n.max(1);
    let frames = audio.len() / n;
    let mut report = Report::default();
    if frames < 1024 {
        return (audio.to_vec(), report);
    }
    let mono: Vec<f32> = (0..frames)
        .map(|i| audio[i * n..i * n + n].iter().sum::<f32>() / n as f32)
        .collect();
    let (f0, hop) = contour(&mono, rate);
    let ratio = ratios(&f0, hop * 1000.0 / f64::from(rate), edit);
    report.frames = f0.len();
    report.voiced = f0.iter().filter(|v| **v > 0.0).count();
    report.moved = f0
        .iter()
        .zip(&ratio)
        .filter(|(h, r)| **h > 0.0 && (1200.0 * r.log2()).abs() > 5.0)
        .count();

    let frame_of = |s: usize| ((s as f64 / hop) as usize).min(f0.len() - 1);
    let hz_at = |s: usize| f0[frame_of(s)];
    let period_at = |s: usize| {
        let hz = hz_at(s);
        if hz > 0.0 {
            f64::from(rate) / hz
        } else {
            f64::from(rate) / 150.0
        }
    };

    let mut wet = vec![0f32; audio.len()];
    let mut win = vec![0f32; frames];
    let mut mask = vec![0f32; frames];
    let edge = ((f64::from(rate) * EDGE_SECONDS) as usize).max(1);

    let mut i = 0;
    while i < f0.len() {
        if f0[i] <= 0.0 || (ratio[i] - 1.0).abs() < 1e-4 && !matches!(edit, Edit::Tune(_)) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < f0.len() && f0[j] > 0.0 {
            j += 1;
        }
        let a = (i as f64 * hop) as usize;
        let b = ((j as f64 * hop) as usize).min(frames);
        let mk = marks(&mono, a, b, &period_at);
        if mk.len() >= 3 {
            // Grains are cut at the original marks and laid at the new spacing.
            let mut k = 0usize;
            let mut s = mk[0] as f64;
            while s < b as f64 && s >= 0.0 {
                let sp = s as usize;
                while k + 1 < mk.len() && (mk[k + 1] as f64 - s).abs() < (mk[k] as f64 - s).abs() {
                    k += 1;
                }
                let m = mk[k];
                let t_in = period_at(m).round().max(2.0) as usize;
                let w = hann(2 * t_in);
                for (g, wv) in w.iter().enumerate() {
                    let src = m as i64 + g as i64 - t_in as i64;
                    let dst = sp as i64 + g as i64 - t_in as i64;
                    if src < 0 || src >= frames as i64 || dst < 0 || dst >= frames as i64 {
                        continue;
                    }
                    let (src, dst) = (src as usize, dst as usize);
                    for c in 0..n {
                        wet[dst * n + c] += audio[src * n + c] * wv;
                    }
                    win[dst] += wv;
                }
                let r = ratio[frame_of(sp)].max(0.25);
                if (r - 1.0).abs() < 1e-4 && k + 1 < mk.len() {
                    // No change wanted here: step from mark to mark, which gives the audio back as it was.
                    s = mk[k + 1] as f64;
                } else {
                    s += (f64::from(rate) / (hz_at(sp).max(40.0) * r)).max(2.0);
                }
            }
            // The stretch this covers, with short fades at its ends.
            for (x, mv) in mask.iter_mut().enumerate().take(b).skip(a) {
                let up = ((x - a) as f32 / edge as f32).min(1.0);
                let down = ((b - 1 - x) as f32 / edge as f32).min(1.0);
                *mv = mv.max(up.min(down));
            }
        }
        i = j;
    }

    let mut out = audio.to_vec();
    for x in 0..frames {
        if mask[x] > 0.0 && win[x] > 1e-3 {
            for c in 0..n {
                let dry = audio[x * n + c];
                let new = wet[x * n + c] / win[x];
                out[x * n + c] = dry + mask[x] * (new - dry);
            }
        }
    }
    (out, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 48_000;

    /// A voice-like tone: eight harmonics falling off as 1/k.
    fn voice(freq: f64, secs: f64) -> Vec<f32> {
        let n = (secs * f64::from(RATE)) as usize;
        (0..n)
            .map(|i| {
                let t = i as f64 / f64::from(RATE);
                let mut v = 0.0;
                for k in 1..=8 {
                    v += (std::f64::consts::TAU * freq * f64::from(k) * t).sin() / f64::from(k);
                }
                (v * 0.25) as f32
            })
            .collect()
    }

    /// The pitch of `x` between `from` and `to` seconds near `near` Hz, by the lag that
    /// repeats the waveform best (the tracker itself is tuned for real voices, not
    /// for the smeared edges a pure test tone gets).
    fn pitch_of(x: &[f32], from: f64, to: f64, near: f64) -> f64 {
        let r = f64::from(RATE);
        let seg = &x[(from * r) as usize..(to * r) as usize];
        let lo = (r / (near * 1.2)) as usize;
        let hi = (r / (near / 1.2)) as usize;
        let mut best = (lo, f64::MIN);
        let mut scores = Vec::new();
        for lag in lo..=hi {
            let (mut num, mut da, mut db) = (0f64, 0f64, 0f64);
            for i in 0..seg.len() - hi {
                num += f64::from(seg[i]) * f64::from(seg[i + lag]);
                da += f64::from(seg[i]).powi(2);
                db += f64::from(seg[i + lag]).powi(2);
            }
            let c = num / (da * db).sqrt().max(1e-12);
            scores.push(c);
            if c > best.1 {
                best = (lag, c);
            }
        }
        // Parabolic refinement around the best lag.
        let k = best.0 - lo;
        let frac = if k > 0 && k + 1 < scores.len() {
            let (l, m, h) = (scores[k - 1], scores[k], scores[k + 1]);
            let d = l - 2.0 * m + h;
            if d.abs() > 1e-12 {
                0.5 * (l - h) / d
            } else {
                0.0
            }
        } else {
            0.0
        };
        r / (best.0 as f64 + frac)
    }

    fn rms(x: &[f32]) -> f64 {
        (x.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
    }

    fn tune(strength: f64, scale: Scale, key: u8) -> Edit {
        Edit::Tune(TuneSettings {
            key,
            scale,
            strength,
            speed_ms: 0.0,
            max_cents: 600.0,
        })
    }

    #[test]
    fn a_flat_note_is_brought_up_to_the_nearest_chromatic_note() {
        // A3 is 220 Hz; 213.8 Hz is about 50 cents flat of it and 50 sharp of G#3.
        let x = voice(215.0, 1.5);
        let (y, rep) = apply(&x, 1, RATE, &tune(1.0, Scale::Chromatic, 0));
        assert_eq!(y.len(), x.len());
        assert!(rep.voiced > rep.frames / 2 && rep.moved > 0);
        let hz = pitch_of(&y, 0.4, 1.2, 220.0);
        assert!((hz - 220.0).abs() < 220.0 * 0.01, "pitch after {hz}");
        // The level stays about the same.
        let db = 20.0 * (rms(&y) / rms(&x)).log10();
        assert!(db.abs() < 1.5, "level changed {db} dB");
    }

    #[test]
    fn strength_zero_leaves_the_audio_as_it_was() {
        let x = voice(215.0, 1.0);
        let (y, _) = apply(&x, 1, RATE, &tune(0.0, Scale::Chromatic, 0));
        let worst = x
            .iter()
            .zip(&y)
            .map(|(a, b)| (a - b).abs())
            .fold(0f32, f32::max);
        assert!(worst < 0.02, "worst difference {worst}");
    }

    #[test]
    fn the_scale_decides_which_notes_attract() {
        // 207 Hz is about G#3. In C major the nearest note is G3 (196) or A3 (220); G#3 is closer to A3 by a hair.
        let x = voice(205.0, 1.5);
        let (y, _) = apply(&x, 1, RATE, &tune(1.0, Scale::Major, 0));
        let hz = pitch_of(&y, 0.4, 1.2, 207.0);
        let semis = midi(hz).rem_euclid(12.0);
        // Pitch classes of C major: 0 2 4 5 7 9 11.
        assert!(
            [0.0, 2.0, 4.0, 5.0, 7.0, 9.0, 11.0]
                .iter()
                .any(|n| (semis - n).abs() < 0.2 || (semis - n).abs() > 11.8),
            "pitch class {semis}"
        );
    }

    #[test]
    fn a_bend_moves_the_pitch_along_its_line() {
        let x = voice(200.0, 2.0);
        let edit = Edit::Bend {
            points: vec![(0.0, 0.0), (1.0, 200.0)],
        };
        let (y, _) = apply(&x, 1, RATE, &edit);
        let first = pitch_of(&y, 0.2, 0.5, 205.0);
        let last = pitch_of(&y, 1.5, 1.8, 225.0);
        // Between 25 and 175 cents is 150 cents, a factor of 1.09.
        let r = last / first;
        assert!(r > 1.06 && r < 1.12, "ratio {r}");
        assert_eq!(y.len(), x.len());
    }

    #[test]
    fn noise_has_no_pitch_and_is_not_touched() {
        let mut s = 0x1234_5678_u64;
        let x: Vec<f32> = (0..RATE as usize)
            .map(|_| {
                s = s
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((s >> 33) as f32 / (1u64 << 31) as f32 - 0.5) * 0.3
            })
            .collect();
        let (y, rep) = apply(&x, 1, RATE, &tune(1.0, Scale::Chromatic, 0));
        assert_eq!(rep.moved, 0);
        assert_eq!(x, y);
    }

    #[test]
    fn every_channel_moves_together_and_the_length_holds() {
        let m = voice(215.0, 1.0);
        let st: Vec<f32> = m.iter().flat_map(|v| [*v, *v * 0.5]).collect();
        let (y, _) = apply(&st, 2, RATE, &tune(1.0, Scale::Chromatic, 0));
        assert_eq!(y.len(), st.len());
        let left: Vec<f32> = y.iter().step_by(2).copied().collect();
        let right: Vec<f32> = y.iter().skip(1).step_by(2).copied().collect();
        let ratio = rms(&right) / rms(&left);
        assert!((ratio - 0.5).abs() < 0.05, "channel balance {ratio}");
    }
}
