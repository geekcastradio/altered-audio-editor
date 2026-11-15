// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Bleed reduction for multi-mic recordings. Each mic also hears the other
//! speakers a little. The other mic's track is a clean copy of that sound, so
//! an adaptive filter can learn how the room turns the other speaker into what
//! this mic hears, and subtract exactly that. Ordinary signal processing, no
//! model and nothing to download.
//!
//! The filter only learns while the other speaker is talking and this mic's own
//! speaker is not (otherwise it would learn to cancel the voice it should keep).
//! The reference is read a little ahead of the sample being cleaned, so a small
//! timing offset between the two recordings does not matter. Tracks that are
//! far out of sync should be lined up with Sync Tracks first.
//!
//! Nothing here touches files: it takes samples and returns samples.

/// What the user can set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// How long a room response the filter can follow, in milliseconds.
    pub filter_ms: f32,
    /// How fast it learns (0.01 slow and steady to 1.0 fast and jumpy).
    pub step: f32,
    /// The loudest bleed expected, in dB below the other speaker's own level.
    /// Anything louder than this at the mic counts as that mic's own voice.
    pub max_bleed_db: f32,
    /// A reference quieter than this (dBFS, average over the filter window)
    /// is treated as silence: nothing is learned and nothing is removed.
    pub quiet_db: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            filter_ms: 12.0,
            step: 0.1,
            max_bleed_db: -6.0,
            quiet_db: -60.0,
        }
    }
}

/// What one pass did.
#[derive(Debug, Clone, PartialEq)]
pub struct Cleaned {
    pub samples: Vec<f32>,
    /// How much quieter the track became overall, in dB (0 when nothing changed).
    pub removed_db: f32,
}

/// The filter length in samples for a rate, kept sensible.
pub fn taps_for(rate: u32, filter_ms: f32) -> usize {
    ((rate as f32 * filter_ms / 1000.0).round() as usize).clamp(16, 8192)
}

/// Takes the other mic's sound out of `target`. Both are the same length
/// (the shorter one decides). `reference` is the other mic's own track.
pub fn reduce(target: &[f32], reference: &[f32], rate: u32, s: &Settings) -> Cleaned {
    let len = target.len().min(reference.len());
    let taps = taps_for(rate, s.filter_ms);
    let ahead = taps / 4;
    // Reference padded so every window p[n + ahead .. n + ahead + taps] exists.
    let mut p = vec![0f32; taps - 1];
    p.extend_from_slice(&reference[..len]);
    p.resize(taps - 1 + len + ahead + taps, 0.0);

    let mut h = vec![0f32; taps];
    let mut out = Vec::with_capacity(len);
    let quiet_power = 10f64.powf(s.quiet_db as f64 / 10.0);
    let ceiling = 10f64.powf(s.max_bleed_db as f64 / 20.0);
    let mu = s.step.clamp(0.001, 1.0) as f64;

    // Running power of the reference window and of the target over the same span.
    let mut ref_energy: f64 = p[ahead..ahead + taps]
        .iter()
        .map(|v| (*v as f64).powi(2))
        .sum();
    let mut tgt_energy: f64 = 0.0;
    let (mut in_sum, mut out_sum) = (0f64, 0f64);

    for n in 0..len {
        if n > 0 {
            let enter = p[n + ahead + taps - 1] as f64;
            let leave = p[n + ahead - 1] as f64;
            ref_energy = (ref_energy + enter * enter - leave * leave).max(0.0);
        }
        let d = target[n];
        let dd = d as f64;
        tgt_energy += dd * dd;
        if n >= taps {
            let old = target[n - taps] as f64;
            tgt_energy = (tgt_energy - old * old).max(0.0);
        }
        let window = &p[n + ahead..n + ahead + taps];
        let active = ref_energy / taps as f64 > quiet_power;
        let e = if active {
            let y: f32 = h.iter().zip(window).map(|(a, b)| a * b).sum();
            let e = d - y;
            // Learn only while the other speaker is on and this mic is mostly bleed.
            let own_voice = tgt_energy.sqrt() > ceiling * ref_energy.sqrt();
            if !own_voice {
                let g = (mu * e as f64 / (ref_energy + 1e-6 * taps as f64)) as f32;
                for (w, x) in h.iter_mut().zip(window) {
                    *w += g * x;
                }
            }
            e
        } else {
            d
        };
        in_sum += dd * dd;
        out_sum += (e as f64) * (e as f64);
        out.push(e);
    }
    let removed_db = if in_sum > 0.0 && out_sum > 0.0 {
        (10.0 * (in_sum / out_sum).log10()) as f32
    } else {
        0.0
    };
    Cleaned {
        samples: out,
        removed_db,
    }
}

/// Cleans two mics against each other. Each is cleaned using the other's
/// original track, so neither pass learns from the other's result.
pub fn reduce_pair(a: &[f32], b: &[f32], rate: u32, s: &Settings) -> (Cleaned, Cleaned) {
    (reduce(a, b, rate, s), reduce(b, a, rate, s))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 16_000;

    /// A repeatable stand-in for a voice: filtered noise in bursts.
    struct Voice {
        state: u64,
        lp: f32,
    }
    impl Voice {
        fn new(seed: u64) -> Self {
            Self {
                state: seed,
                lp: 0.0,
            }
        }
        fn next(&mut self) -> f32 {
            self.state = self
                .state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let n = ((self.state >> 33) as f32 / (1u64 << 31) as f32) * 2.0 - 1.0;
            self.lp += 0.5 * (n - self.lp);
            self.lp
        }
    }

    /// A talker that is on during the given second ranges.
    fn talker(seed: u64, secs: usize, on: &[(usize, usize)]) -> Vec<f32> {
        let mut v = Voice::new(seed);
        (0..secs * RATE as usize)
            .map(|i| {
                let t = i / RATE as usize;
                let s = v.next();
                if on.iter().any(|&(a, b)| t >= a && t < b) {
                    s
                } else {
                    0.0
                }
            })
            .collect()
    }

    /// What the other speaker sounds like in this mic: delayed, a little room.
    fn through_room(x: &[f32]) -> Vec<f32> {
        let taps: [(usize, f32); 4] = [(20, 0.25), (55, 0.10), (130, -0.05), (210, 0.03)];
        (0..x.len())
            .map(|n| {
                taps.iter()
                    .map(|&(d, g)| if n >= d { g * x[n - d] } else { 0.0 })
                    .sum()
            })
            .collect()
    }

    fn mic(own: &[f32], other: &[f32]) -> Vec<f32> {
        own.iter()
            .zip(through_room(other))
            .map(|(a, b)| a + b)
            .collect()
    }

    fn power(x: &[f32]) -> f64 {
        x.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / x.len().max(1) as f64
    }

    fn db(r: f64) -> f64 {
        10.0 * r.log10()
    }

    fn seg(x: &[f32], a: usize, b: usize) -> &[f32] {
        &x[a * RATE as usize..b * RATE as usize]
    }

    #[test]
    fn the_other_speakers_bleed_drops_a_lot_once_the_filter_has_learned() {
        let a = talker(1, 20, &[(5, 10), (10, 15)]);
        let b = talker(2, 20, &[(0, 5), (10, 20)]);
        let m1 = mic(&a, &b);
        let r = reduce(&m1, &b, RATE, &Settings::default());
        // 15 to 20 s: only B talks, so everything in mic 1 is bleed.
        let before = power(seg(&m1, 15, 20));
        let after = power(seg(&r.samples, 15, 20));
        assert!(
            db(before / after) > 18.0,
            "only {:.1} dB",
            db(before / after)
        );
    }

    #[test]
    fn the_wearers_own_voice_survives_while_both_talk() {
        let a = talker(1, 20, &[(5, 10), (10, 15)]);
        let b = talker(2, 20, &[(0, 5), (10, 20)]);
        let m1 = mic(&a, &b);
        let r = reduce(&m1, &b, RATE, &Settings::default());
        // 10 to 15 s: both talk. Compare with the true own voice.
        let err_in: Vec<f32> = seg(&m1, 10, 15)
            .iter()
            .zip(seg(&a, 10, 15))
            .map(|(x, y)| x - y)
            .collect();
        let err_out: Vec<f32> = seg(&r.samples, 10, 15)
            .iter()
            .zip(seg(&a, 10, 15))
            .map(|(x, y)| x - y)
            .collect();
        let snr_in = db(power(seg(&a, 10, 15)) / power(&err_in));
        let snr_out = db(power(seg(&a, 10, 15)) / power(&err_out));
        assert!(
            snr_out > snr_in + 6.0,
            "in {snr_in:.1} dB, out {snr_out:.1} dB"
        );
        // And the voice itself was not cancelled: the output still holds most of its power.
        assert!(power(seg(&r.samples, 10, 15)) > 0.7 * power(seg(&a, 10, 15)));
    }

    #[test]
    fn a_silent_reference_changes_nothing() {
        let a = talker(1, 4, &[(0, 4)]);
        let silent = vec![0f32; a.len()];
        let r = reduce(&a, &silent, RATE, &Settings::default());
        assert_eq!(r.samples, a);
        assert_eq!(r.removed_db, 0.0);
    }

    #[test]
    fn a_small_timing_offset_between_the_recordings_is_followed() {
        let a = talker(1, 20, &[(5, 10)]);
        let b = talker(2, 20, &[(0, 5), (10, 20)]);
        // The other mic's recording starts 40 samples late, as if the files were not quite in step.
        let mut b_late = vec![0f32; 40];
        b_late.extend_from_slice(&b[..b.len() - 40]);
        let m1 = mic(&a, &b);
        let r = reduce(&m1, &b_late, RATE, &Settings::default());
        let ratio = power(seg(&m1, 15, 20)) / power(seg(&r.samples, 15, 20));
        assert!(db(ratio) > 15.0, "only {:.1} dB", db(ratio));
    }

    #[test]
    fn both_mics_are_cleaned_against_each_other() {
        let a = talker(1, 20, &[(5, 10), (10, 15)]);
        let b = talker(2, 20, &[(0, 5), (10, 20)]);
        let m1 = mic(&a, &b);
        let m2 = mic(&b, &a);
        let (c1, c2) = reduce_pair(&m1, &m2, RATE, &Settings::default());
        assert_eq!(c1.samples.len(), m1.len());
        assert_eq!(c2.samples.len(), m2.len());
        // 5 to 10 s only A talks: mic 2 is nothing but A's bleed.
        assert!(db(power(seg(&m2, 5, 10)) / power(seg(&c2.samples, 5, 10))) > 15.0);
        // Overall the change is small because each mic is mostly its own voice, but never negative.
        assert!(c1.removed_db > 0.0 && c2.removed_db > 0.0);
    }

    #[test]
    fn lengths_and_settings_are_handled() {
        let r = reduce(&[0.1; 10], &[0.1; 5], RATE, &Settings::default());
        assert_eq!(r.samples.len(), 5);
        assert!(reduce(&[], &[], RATE, &Settings::default())
            .samples
            .is_empty());
        assert_eq!(taps_for(48_000, 12.0), 576);
        assert_eq!(taps_for(8_000, 0.0), 16);
    }
}
