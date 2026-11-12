// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Auto Mic Mix: works out, for each track of a multi-mic recording, when it
//! should be turned down because someone else is the one talking. Each mic
//! also hears the other speakers a little (bleed), so a plain gate on level
//! would hold open on bleed. Instead each track is measured against its own
//! quiet level, and a track stays open only while it is the speaker (or close
//! to the loudest one) once everything is compared fairly.
//!
//! The result is a gain curve per track as points (sample, dB), ready to
//! become a volume envelope. Nothing here touches audio.

/// What the user can set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// How far above its own quiet level a track must be to count as speaking.
    pub open_db: f32,
    /// A track stays open while within this many dB of the loudest track
    /// (both measured above their own quiet levels).
    pub margin_db: f32,
    /// How far a turned-down track is lowered (a positive number of dB).
    pub reduce_db: f32,
    /// Turning down fades over this long.
    pub fade_ms: f32,
    /// Turning up is faster, so the start of a word is not cut.
    pub attack_ms: f32,
    /// A track stays open this long after it stops being the speaker.
    pub hold_ms: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            open_db: 8.0,
            margin_db: 6.0,
            reduce_db: 12.0,
            fade_ms: 150.0,
            attack_ms: 15.0,
            hold_ms: 250.0,
        }
    }
}

/// One point of a gain curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub sample: usize,
    pub gain_db: f32,
}

const HOP_MS: f32 = 10.0;

fn db(x: f32) -> f32 {
    20.0 * x.max(1e-9).log10()
}

/// Level of each hop (RMS, in dB) for one track.
fn levels(x: &[f32], hop: usize, hops: usize) -> Vec<f32> {
    (0..hops)
        .map(|h| {
            let s = &x[(h * hop).min(x.len())..((h + 1) * hop).min(x.len())];
            if s.is_empty() {
                return -120.0;
            }
            db((s.iter().map(|v| v * v).sum::<f32>() / s.len() as f32).sqrt())
        })
        .collect()
}

/// The quiet level of a track: a low percentile of its hop levels, so
/// pauses set it even when the track is mostly speech.
fn quiet_level(levels: &[f32]) -> f32 {
    let mut v: Vec<f32> = levels.iter().copied().filter(|l| *l > -110.0).collect();
    if v.is_empty() {
        return -120.0;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 10]
}

/// Hop length in samples at `rate`.
pub fn hop_len(rate: u32) -> usize {
    ((rate as f32 * HOP_MS / 1000.0) as usize).max(1)
}

/// Level of every hop of `x` (RMS in dB), for audio that starts on a hop
/// boundary. Long recordings are measured a chunk at a time and the results
/// joined, so the audio never has to be in memory whole.
pub fn track_levels(x: &[f32], rate: u32) -> Vec<f32> {
    let hop = hop_len(rate);
    levels(x, hop, x.len().div_ceil(hop))
}

/// Gain curves from per-track hop levels (all tracks the same number of hops,
/// shorter ones padded with silence). `len` is the length in samples.
/// Returns None for fewer than two tracks. Points are only kept where the
/// gain moves by more than half a dB, plus the first and last.
pub fn mix_from_levels(
    lv: &[Vec<f32>],
    rate: u32,
    len: usize,
    s: &Settings,
) -> Option<Vec<Vec<Point>>> {
    if lv.len() < 2 || rate == 0 {
        return None;
    }
    let hop = hop_len(rate);
    let hops = lv.iter().map(Vec::len).max().unwrap_or(0).max(1);
    let at = |l: &[f32], h: usize| l.get(h).copied().unwrap_or(-120.0);
    let quiet: Vec<f32> = lv.iter().map(|l| quiet_level(l)).collect();

    let hold = (s.hold_ms / HOP_MS).round() as usize;
    // Per-hop smoothing: dB moved per hop when turning down and up.
    let down = s.reduce_db / (s.fade_ms / HOP_MS).max(1.0);
    let up = s.reduce_db / (s.attack_ms / HOP_MS).max(1.0);

    let mut out = Vec::new();
    for (i, li) in lv.iter().enumerate() {
        let mut open_for = 0usize;
        let mut gain = 0.0f32;
        let mut curve: Vec<Point> = Vec::new();
        let mut last_kept = f32::NAN;
        for h in 0..hops {
            let above = at(li, h) - quiet[i];
            let loudest = lv
                .iter()
                .enumerate()
                .map(|(j, l)| at(l, h) - quiet[j])
                .fold(f32::MIN, f32::max);
            let speaking = above >= s.open_db && above >= loudest - s.margin_db;
            if speaking {
                open_for = hold;
            } else {
                open_for = open_for.saturating_sub(1);
            }
            // Nobody speaking at all: leave everything open (room tone stays even).
            let anyone = loudest >= s.open_db;
            let target = if open_for > 0 || !anyone {
                0.0
            } else {
                -s.reduce_db
            };
            if target < gain {
                gain = (gain - down).max(target);
            } else {
                gain = (gain + up).min(target);
            }
            if last_kept.is_nan() || (gain - last_kept).abs() > 0.5 || h == hops - 1 {
                curve.push(Point {
                    sample: (h * hop).min(len),
                    gain_db: gain,
                });
                last_kept = gain;
            }
        }
        out.push(curve);
    }
    Some(out)
}

/// Gain curves for whole tracks in memory (used by the tests and small jobs).
pub fn auto_mix(tracks: &[&[f32]], rate: u32, s: &Settings) -> Option<Vec<Vec<Point>>> {
    let len = tracks.iter().map(|t| t.len()).max().unwrap_or(0);
    let lv: Vec<Vec<f32>> = tracks.iter().map(|t| track_levels(t, rate)).collect();
    mix_from_levels(&lv, rate, len, s)
}

/// Seconds each track spent as the speaker, by the same rule that opens a
/// track in the mix: well above its own quiet level and close to the loudest.
pub fn talk_seconds(lv: &[Vec<f32>], s: &Settings) -> Vec<f32> {
    let hops = lv.iter().map(Vec::len).max().unwrap_or(0);
    let at = |l: &[f32], h: usize| l.get(h).copied().unwrap_or(-120.0);
    let quiet: Vec<f32> = lv.iter().map(|l| quiet_level(l)).collect();
    let mut hops_talking = vec![0usize; lv.len()];
    for h in 0..hops {
        let loudest = lv
            .iter()
            .enumerate()
            .map(|(j, l)| at(l, h) - quiet[j])
            .fold(f32::MIN, f32::max);
        for (i, l) in lv.iter().enumerate() {
            let above = at(l, h) - quiet[i];
            if above >= s.open_db && above >= loudest - s.margin_db {
                hops_talking[i] += 1;
            }
        }
    }
    hops_talking
        .into_iter()
        .map(|n| n as f32 * HOP_MS / 1000.0)
        .collect()
}

/// The gain a curve gives at `sample` (held at the nearest earlier point).
pub fn gain_at(curve: &[Point], sample: usize) -> f32 {
    curve
        .iter()
        .take_while(|p| p.sample <= sample)
        .last()
        .map_or(0.0, |p| p.gain_db)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 16_000;

    /// A tone burst of `secs` at `amp` starting at `start` seconds, over a quiet floor.
    fn speech(total: f32, bursts: &[(f32, f32, f32)], floor: f32) -> Vec<f32> {
        let n = (total * RATE as f32) as usize;
        let mut x = vec![0.0f32; n];
        // Deterministic noise floor.
        let mut seed = 12345u32;
        for v in x.iter_mut() {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *v = ((seed >> 16) as f32 / 32768.0 - 1.0) * floor;
        }
        for &(start, secs, amp) in bursts {
            let a = (start * RATE as f32) as usize;
            let b = (((start + secs) * RATE as f32) as usize).min(n);
            for (k, v) in x.iter_mut().enumerate().take(b).skip(a) {
                *v += amp * (k as f32 * 0.31).sin();
            }
        }
        x
    }

    /// Two mics. A speaks 1 to 4 s, B speaks 5 to 8 s; each hears the other 18 dB down.
    fn two_mics() -> (Vec<f32>, Vec<f32>) {
        let bleed = 0.125;
        let a = speech(10.0, &[(1.0, 3.0, 0.5), (5.0, 3.0, 0.5 * bleed)], 0.002);
        let b = speech(10.0, &[(1.0, 3.0, 0.5 * bleed), (5.0, 3.0, 0.5)], 0.002);
        (a, b)
    }

    #[test]
    fn needs_two_tracks() {
        let (a, _) = two_mics();
        assert!(auto_mix(&[&a], RATE, &Settings::default()).is_none());
    }

    #[test]
    fn the_quiet_mic_is_turned_down_while_the_other_speaks() {
        let (a, b) = two_mics();
        let c = auto_mix(&[&a, &b], RATE, &Settings::default()).unwrap();
        let at = |s: f32| (s * RATE as f32) as usize;
        // A speaks at 2.5 s: A open, B down.
        assert!(gain_at(&c[0], at(2.5)) > -1.0);
        assert!(gain_at(&c[1], at(2.5)) <= -11.0);
        // B speaks at 6.5 s: the other way round.
        assert!(gain_at(&c[1], at(6.5)) > -1.0);
        assert!(gain_at(&c[0], at(6.5)) <= -11.0);
    }

    #[test]
    fn silence_everywhere_leaves_both_open() {
        let (a, b) = two_mics();
        let c = auto_mix(&[&a, &b], RATE, &Settings::default()).unwrap();
        let at = |s: f32| (s * RATE as f32) as usize;
        // 9.5 s: nobody talks. Both recover to open (after the hold and attack).
        assert!(gain_at(&c[0], at(9.5)) > -1.0);
        assert!(gain_at(&c[1], at(9.5)) > -1.0);
    }

    #[test]
    fn crosstalk_at_similar_level_keeps_both_open() {
        // Both talk at once at the same level: neither is turned down.
        let a = speech(6.0, &[(1.0, 4.0, 0.5)], 0.002);
        let b = speech(6.0, &[(1.0, 4.0, 0.45)], 0.002);
        let c = auto_mix(&[&a, &b], RATE, &Settings::default()).unwrap();
        let at = |s: f32| (s * RATE as f32) as usize;
        assert!(gain_at(&c[0], at(3.0)) > -1.0);
        assert!(gain_at(&c[1], at(3.0)) > -1.0);
    }

    #[test]
    fn the_start_of_a_word_is_not_cut() {
        let (a, b) = two_mics();
        let s = Settings::default();
        let c = auto_mix(&[&a, &b], RATE, &s).unwrap();
        // B starts speaking at 5 s. Within 30 ms B must be most of the way open.
        let at = |sec: f32| (sec * RATE as f32) as usize;
        assert!(gain_at(&c[1], at(5.03)) > -s.reduce_db * 0.5);
    }

    #[test]
    fn curves_are_sparse_and_end_on_the_last_sample() {
        let (a, b) = two_mics();
        let c = auto_mix(&[&a, &b], RATE, &Settings::default()).unwrap();
        for curve in &c {
            assert!(curve.len() < 120, "{} points", curve.len());
            assert!(curve.last().unwrap().sample <= a.len());
            assert!(curve.windows(2).all(|w| w[0].sample <= w[1].sample));
        }
    }

    #[test]
    fn talk_time_counts_each_speaker() {
        let (a, b) = two_mics();
        let lv: Vec<Vec<f32>> = [&a, &b].iter().map(|t| track_levels(t, RATE)).collect();
        let t = talk_seconds(&lv, &Settings::default());
        // Each speaks 3 s; allow a hop or two at the edges.
        assert!((t[0] - 3.0).abs() < 0.2, "{t:?}");
        assert!((t[1] - 3.0).abs() < 0.2, "{t:?}");
    }

    #[test]
    fn joined_chunks_give_the_same_levels_as_one_pass() {
        let (a, _) = two_mics();
        let whole = track_levels(&a, RATE);
        let hop = hop_len(RATE);
        let cut = hop * 200;
        let mut joined = track_levels(&a[..cut], RATE);
        joined.extend(track_levels(&a[cut..], RATE));
        assert_eq!(whole, joined);
    }
}
