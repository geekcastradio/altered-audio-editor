// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Sync Tracks: lines up two recordings of the same conversation (a double-ender, or a
//! call recording against a local mic) and measures how far their clocks drift apart.
//!
//! The first track is the timebase. This module only measures: it reports where the other
//! track starts relative to it and how fast the gap grows, and the caller decides what to do.
//! The two recordings must share audio. Two mics that each hear only their own voice have
//! nothing in common to match.
//!
//! Method: a coarse pass correlates the loudness envelopes to find the offset to a few
//! milliseconds, then short windows spread across the recording are correlated sample by
//! sample around that estimate. A straight line through the window results gives the offset
//! at time zero and the drift.

use super::align::align;

/// Needs at least this much audio on each track.
pub const MIN_SECONDS: usize = 120;
/// The largest offset the search covers.
pub const MAX_OFFSET_SECONDS: usize = 600;
/// Envelope resolution for the coarse pass.
const ENVELOPE_HZ: usize = 200;
/// Longest stretch of each track the coarse pass looks at.
const COARSE_SECONDS: usize = 1800;
const WINDOW_SECONDS: usize = 8;
const MAX_WINDOWS: usize = 40;
/// A window needs at least this much match to count.
const MIN_WINDOW_SCORE: f32 = 0.2;
/// The envelope match has to beat this to trust the coarse offset.
const MIN_COARSE_SCORE: f32 = 0.08;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    /// Samples the other track trails the first at its start (negative: it starts early).
    pub offset: i64,
    /// How much the gap grows, in parts per million of the first track's time. Positive means
    /// the other track's clock runs slower.
    pub drift_ppm: f64,
    /// 0 to 1. Low means the answer is a guess; show it and let the user decide.
    pub confidence: f32,
    /// The other track is upside down against the first.
    pub inverted: bool,
    /// How many windows agreed on the line.
    pub windows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncError {
    /// One of the tracks has less than two minutes of audio.
    TooShort,
    /// No shared audio was found within ten minutes either way.
    NoMatch,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::TooShort => write!(f, "Each track needs at least two minutes of audio to sync."),
            SyncError::NoMatch => write!(f, "These tracks do not seem to share any audio within ten minutes of each other. Sync needs both tracks to have heard the same conversation."),
        }
    }
}

/// Loudness of each block of `factor` samples, with the average taken out.
fn envelope(x: &[f32], factor: usize) -> Vec<f32> {
    let mut out: Vec<f32> = x
        .chunks(factor)
        .map(|c| c.iter().map(|v| v.abs()).sum::<f32>() / c.len() as f32)
        .collect();
    if out.is_empty() {
        return out;
    }
    let mean = out.iter().sum::<f32>() / out.len() as f32;
    for v in &mut out {
        *v -= mean;
    }
    out
}

/// Measures `other` against `reference` (both mono, same sample rate).
pub fn sync(reference: &[f32], other: &[f32], rate: u32) -> Result<SyncResult, SyncError> {
    let rate = rate as usize;
    if reference.len() < MIN_SECONDS * rate || other.len() < MIN_SECONDS * rate {
        return Err(SyncError::TooShort);
    }
    let factor = (rate / ENVELOPE_HZ).max(1);
    let span = reference.len().min(other.len()).min(COARSE_SECONDS * rate);
    let ea = envelope(&reference[..span], factor);
    let eb = envelope(&other[..span], factor);
    let coarse = align(&ea, &eb, MAX_OFFSET_SECONDS * rate / factor).ok_or(SyncError::NoMatch)?;
    if coarse.score < MIN_COARSE_SCORE {
        return Err(SyncError::NoMatch);
    }
    let est = coarse.lag * factor as i64;

    // Fine pass: windows across the first track, each matched against `other` near the estimate.
    let w = WINDOW_SECONDS * rate;
    let pad = factor * 3 + 64;
    let last = reference.len().saturating_sub(w);
    let step = (30 * rate).max(last / MAX_WINDOWS).max(1);
    let mut points: Vec<(f64, f64, f32, bool)> = Vec::new();
    let mut tried = 0usize;
    let mut p = 0usize;
    while p <= last {
        tried += 1;
        let start = p as i64 + est - pad as i64;
        if start >= 0 && start as usize + w <= other.len() {
            let r = &reference[p..p + w];
            let o = &other[start as usize..start as usize + w];
            if let Some(a) = align(r, o, 2 * pad) {
                if a.score >= MIN_WINDOW_SCORE {
                    let lag = est - pad as i64 + a.lag;
                    points.push(((p + w / 2) as f64, lag as f64, a.score, a.inverted));
                }
            }
        }
        p += step;
    }
    if points.len() < 3 {
        return Err(SyncError::NoMatch);
    }

    // A straight line through the window results; drop the ones far off it and fit again.
    let mut keep = points.clone();
    let mut fit = line(&keep);
    for _ in 0..2 {
        let limit = (rate as f64 * 0.002).max(2.0);
        let next: Vec<_> = keep
            .iter()
            .copied()
            .filter(|q| (q.1 - (fit.0 + fit.1 * q.0)).abs() <= limit)
            .collect();
        if next.len() < 3 || next.len() == keep.len() {
            break;
        }
        keep = next;
        fit = line(&keep);
    }
    if keep.len() < 3 {
        return Err(SyncError::NoMatch);
    }
    let mean_score = keep.iter().map(|q| q.2).sum::<f32>() / keep.len() as f32;
    let flipped = keep.iter().filter(|q| q.3).count() * 2 > keep.len();
    Ok(SyncResult {
        offset: fit.0.round() as i64,
        drift_ppm: fit.1 * 1e6,
        confidence: (mean_score * (keep.len() as f32 / tried as f32)).clamp(0.0, 1.0),
        inverted: flipped,
        windows: keep.len(),
    })
}

/// Least-squares line through (position, lag): returns (lag at position 0, slope).
fn line(points: &[(f64, f64, f32, bool)]) -> (f64, f64) {
    let n = points.len() as f64;
    let mx = points.iter().map(|q| q.0).sum::<f64>() / n;
    let my = points.iter().map(|q| q.1).sum::<f64>() / n;
    let sxx: f64 = points.iter().map(|q| (q.0 - mx).powi(2)).sum();
    let sxy: f64 = points.iter().map(|q| (q.0 - mx) * (q.1 - my)).sum();
    let slope = if sxx > 0.0 { sxy / sxx } else { 0.0 };
    (my - slope * mx, slope)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 16_000;

    /// Speech-like: noise and a tone gated by phrases of random length with pauses between.
    fn talk(seconds: usize, mut seed: u32) -> Vec<f32> {
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as f32 / u32::MAX as f32
        };
        let total = seconds * RATE as usize;
        let mut out = vec![0f32; total];
        let mut i = 0;
        let mut speaking = true;
        while i < total {
            let len = ((0.2 + next() * 1.3) * RATE as f32) as usize;
            if speaking {
                let pitch = 90.0 + next() * 140.0;
                for (k, v) in out
                    .iter_mut()
                    .enumerate()
                    .take((i + len).min(total))
                    .skip(i)
                {
                    let n = next() - 0.5;
                    *v = n * 0.3
                        + (k as f32 * pitch * std::f32::consts::TAU / RATE as f32).sin() * 0.25;
                }
            }
            speaking = !speaking;
            i += len;
        }
        out
    }

    /// `x` seen through a clock that starts `offset` samples late and falls behind by `drift` per sample.
    fn through(x: &[f32], offset: f64, drift: f64, gain: f32) -> Vec<f32> {
        (0..x.len())
            .map(|t| {
                let src = t as f64 - offset - drift * t as f64;
                if src < 0.0 || src >= (x.len() - 1) as f64 {
                    return 0.0;
                }
                let i = src.floor() as usize;
                let f = (src - i as f64) as f32;
                gain * (x[i] * (1.0 - f) + x[i + 1] * f)
            })
            .collect()
    }

    #[test]
    fn finds_offset_and_drift() {
        let a = talk(240, 7);
        let b = through(&a, 1.37 * RATE as f64, 100e-6, 0.5);
        let r = sync(&a, &b, RATE).unwrap();
        assert!(
            (r.offset - (1.37 * RATE as f64) as i64).abs() <= 4,
            "offset {}",
            r.offset
        );
        assert!((r.drift_ppm - 100.0).abs() < 8.0, "drift {}", r.drift_ppm);
        assert!(!r.inverted);
        assert!(r.confidence > 0.3, "confidence {}", r.confidence);
    }

    #[test]
    fn finds_a_track_that_starts_early() {
        let a = talk(240, 9);
        let b = through(&a, -2.5 * RATE as f64, 0.0, 1.0);
        let r = sync(&a, &b, RATE).unwrap();
        assert!(
            (r.offset + (2.5 * RATE as f64) as i64).abs() <= 4,
            "offset {}",
            r.offset
        );
        assert!(r.drift_ppm.abs() < 8.0, "drift {}", r.drift_ppm);
    }

    #[test]
    fn notices_a_flipped_track() {
        let a = talk(200, 3);
        let b = through(&a, 0.8 * RATE as f64, 0.0, -0.7);
        assert!(sync(&a, &b, RATE).unwrap().inverted);
    }

    #[test]
    fn unrelated_audio_has_no_match() {
        assert_eq!(
            sync(&talk(200, 1), &talk(200, 2), RATE),
            Err(SyncError::NoMatch)
        );
    }

    #[test]
    fn short_tracks_are_refused() {
        assert_eq!(
            sync(&talk(60, 1), &talk(60, 1), RATE),
            Err(SyncError::TooShort)
        );
    }
}
