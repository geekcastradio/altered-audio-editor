// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Room tone: the steady sound of a quiet room, found in a recording and
//! used to fill gaps. Cutting a breath or a flub out of narration leaves
//! either a hole of digital silence or a step in the noise floor, and
//! both stand out (audiobook checks reject silence outright).
//!
//! The method is granular and original: the quietest stretch of the
//! recording is cut into short overlapping grains taken at random places
//! in it, each shaped by a sine window. Two overlapping sine-shaped
//! grains add to constant power for uncorrelated noise, so the fill has
//! the level and tone of the source and never loops audibly.

/// Length of a grain, seconds.
const GRAIN_SECONDS: f64 = 0.2;
/// Windows of 50 ms; a run this long is one second of tone.
pub const WINDOW_SECONDS: f64 = 0.05;
pub const SOURCE_WINDOWS: usize = 20;
/// Power below this (about -120 dBFS) is digital silence, not room tone.
pub const SILENT_POWER: f64 = 1e-12;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in 0..n.
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

/// Where the quietest run of `need` consecutive windows starts, as a
/// window index. `windows` holds each window's power, or None where it
/// cannot be used (digital silence, or inside the part being filled).
pub fn quietest_run(windows: &[Option<f64>], need: usize) -> Option<usize> {
    if need == 0 || windows.len() < need {
        return None;
    }
    let mut best: Option<(usize, f64)> = None;
    'runs: for start in 0..=windows.len() - need {
        let mut sum = 0.0;
        for w in &windows[start..start + need] {
            match w {
                Some(p) => sum += p,
                None => continue 'runs,
            }
        }
        let mean = sum / need as f64;
        if best.is_none_or(|(_, b)| mean < b) {
            best = Some((start, mean));
        }
    }
    best.map(|(s, _)| s)
}

/// `frames` frames of room tone made from `source` (stereo, interleaved).
/// None when the source is shorter than two grains.
pub fn synthesize(source: &[f32], frames: usize, rate: u32, seed: u64) -> Option<Vec<f32>> {
    let src = source.as_chunks::<2>().0;
    let grain = ((f64::from(rate) * GRAIN_SECONDS) as usize).max(8) & !1;
    if src.len() < grain * 2 {
        return None;
    }
    let hop = grain / 2;
    let window: Vec<f32> = (0..grain)
        .map(|i| (std::f64::consts::PI * (i as f64 + 0.5) / grain as f64).sin() as f32)
        .collect();
    let mut out = vec![0f32; frames * 2];
    let mut rng = Rng(seed | 1);
    let room = src.len() - grain;
    let mut last = rng.below(room + 1);
    // The first grain starts half a grain early so the fill begins at full level.
    let mut pos = -(hop as i64);
    while pos < frames as i64 {
        let mut off = rng.below(room + 1);
        // Keep neighbouring grains from coming from the same place.
        for _ in 0..4 {
            if off.abs_diff(last) > grain / 4 {
                break;
            }
            off = rng.below(room + 1);
        }
        last = off;
        for i in 0..grain {
            let at = pos + i as i64;
            if at < 0 || at >= frames as i64 {
                continue;
            }
            let s = src[off + i];
            let w = window[i];
            let o = at as usize * 2;
            out[o] += s[0] * w;
            out[o + 1] += s[1] * w;
        }
        pos += hop as i64;
    }
    Some(out)
}

/// `fill` laid over `orig` (both `n`-channel interleaved, same length)
/// with a blend of `edge` frames at each end, so the change is not a click.
pub fn blend(orig: &[f32], fill: &[f32], n: usize, edge: usize) -> Vec<f32> {
    let n = n.max(1);
    let frames = orig.len().min(fill.len()) / n;
    let edge = edge.min(frames / 2);
    let mut out = Vec::with_capacity(frames * n);
    for i in 0..frames {
        let k = if edge == 0 {
            1.0
        } else {
            (i.min(frames - 1 - i) as f32 / edge as f32).min(1.0)
        };
        for c in 0..n {
            let o = orig[i * n + c];
            out.push(o + (fill[i * n + c] - o) * k);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noise(frames: usize, amp: f32, seed: u64) -> Vec<f32> {
        let mut r = Rng(seed | 1);
        (0..frames * 2)
            .map(|_| ((r.next() >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0) * amp)
            .collect()
    }

    fn rms(x: &[f32]) -> f64 {
        (x.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
    }

    #[test]
    fn the_fill_matches_the_level_of_the_source() {
        let src = noise(48_000, 0.01, 7);
        let fill = synthesize(&src, 48_000 * 3, 48_000, 99).unwrap();
        assert_eq!(fill.len(), 48_000 * 3 * 2);
        let ratio = rms(&fill) / rms(&src);
        assert!((ratio - 1.0).abs() < 0.08, "{ratio}");
        // Steady from the first frame to the last: no fade in or out.
        let head = rms(&fill[..4800]);
        let tail = rms(&fill[fill.len() - 4800..]);
        assert!((head / rms(&src) - 1.0).abs() < 0.2, "{head}");
        assert!((tail / rms(&src) - 1.0).abs() < 0.2, "{tail}");
    }

    #[test]
    fn it_does_not_repeat_itself() {
        let src = noise(48_000, 0.01, 3);
        let fill = synthesize(&src, 48_000 * 2, 48_000, 5).unwrap();
        let a = &fill[..9600];
        let b = &fill[9600..19200];
        let dot: f64 = a
            .iter()
            .zip(b)
            .map(|(x, y)| f64::from(*x) * f64::from(*y))
            .sum();
        let norm = rms(a) * rms(b) * a.len() as f64;
        assert!((dot / norm).abs() < 0.2, "{}", dot / norm);
    }

    #[test]
    fn too_short_a_source_is_refused() {
        assert!(synthesize(&noise(1000, 0.01, 1), 100, 48_000, 1).is_none());
    }

    #[test]
    fn it_is_repeatable_for_a_seed_and_differs_between_seeds() {
        let src = noise(48_000, 0.01, 2);
        let a = synthesize(&src, 10_000, 48_000, 11).unwrap();
        let b = synthesize(&src, 10_000, 48_000, 11).unwrap();
        let c = synthesize(&src, 10_000, 48_000, 12).unwrap();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn the_quietest_run_skips_unusable_windows() {
        let mut w: Vec<Option<f64>> = vec![Some(1.0); 30];
        for p in &mut w[10..14] {
            *p = Some(0.001);
        }
        w[12] = None; // a hole breaks that run
        for p in &mut w[20..26] {
            *p = Some(0.01);
        }
        assert_eq!(quietest_run(&w, 4), Some(20));
        assert_eq!(quietest_run(&w, 40), None);
        assert_eq!(quietest_run(&[], 1), None);
    }

    #[test]
    fn the_blend_keeps_the_ends_and_swaps_the_middle() {
        let orig = vec![1.0f32; 200];
        let fill = vec![0.0f32; 200];
        let out = blend(&orig, &fill, 2, 10);
        assert_eq!(out.len(), 200);
        assert!((out[0] - 1.0).abs() < 1e-6);
        assert!((out[199] - 1.0).abs() < 1e-6);
        assert!(out[100].abs() < 1e-6);
        // Mono works the same way.
        let mono = blend(&[1.0; 100], &[0.0; 100], 1, 10);
        assert_eq!(mono.len(), 100);
        assert!(mono[50].abs() < 1e-6);
    }
}
