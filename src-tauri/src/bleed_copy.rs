// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Bleed reduction on a copy of a clip's audio: reads the 32-bit float WAV copy, takes the
//! other mic's sound out of every channel using the reference, and writes it back in place.
//! The original file is never touched: the caller makes the copy first and points the clip at
//! it, so undo brings the original back.

use std::path::Path;

use crate::dsp::bleed::{reduce, Settings};

/// Cleans the WAV at `path` against `reference` (mono, at the same rate, starting where the
/// clip starts). Returns how many dB quieter the clip became overall.
pub fn clean_wav(path: &Path, reference: &[f32], s: &Settings) -> Result<f32, String> {
    let mut r = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = r.spec();
    if spec.sample_format != hound::SampleFormat::Float || spec.bits_per_sample != 32 {
        return Err("The copy of the clip is not 32-bit float audio.".into());
    }
    let channels = usize::from(spec.channels).max(1);
    let audio: Vec<f32> = r
        .samples::<f32>()
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    drop(r);
    let frames = audio.len() / channels;
    let mut out = audio.clone();
    let (mut before, mut after) = (0f64, 0f64);
    for c in 0..channels {
        let one: Vec<f32> = (0..frames).map(|f| audio[f * channels + c]).collect();
        let cleaned = reduce(&one, reference, spec.sample_rate, s);
        for (f, v) in cleaned.samples.iter().enumerate() {
            out[f * channels + c] = *v;
        }
        before += one.iter().map(|v| f64::from(*v).powi(2)).sum::<f64>();
        after += (0..frames)
            .map(|f| f64::from(out[f * channels + c]).powi(2))
            .sum::<f64>();
    }
    let mut w = hound::WavWriter::create(path, spec).map_err(|e| e.to_string())?;
    for v in out {
        w.write_sample(v).map_err(|e| e.to_string())?;
    }
    w.finalize().map_err(|e| e.to_string())?;
    Ok(if before > 0.0 && after > 0.0 {
        (10.0 * (before / after).log10()) as f32
    } else {
        0.0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, channels: u16, rate: u32, samples: &[f32]) {
        let spec = hound::WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut w = hound::WavWriter::create(path, spec).unwrap();
        for v in samples {
            w.write_sample(*v).unwrap();
        }
        w.finalize().unwrap();
    }

    fn noise(n: usize, seed: u64) -> Vec<f32> {
        let mut s = seed;
        (0..n)
            .map(|_| {
                s = s
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((s >> 33) as f32 / (1u64 << 31) as f32) * 2.0 - 1.0
            })
            .collect()
    }

    #[test]
    fn a_stereo_copy_loses_the_other_mics_sound_in_both_channels() {
        let rate = 16_000;
        let other = noise(rate as usize * 6, 7);
        // The clip is nothing but the other speaker, 0.25 as loud and 20 samples late.
        let bleed: Vec<f32> = (0..other.len())
            .map(|n| if n >= 20 { 0.25 * other[n - 20] } else { 0.0 })
            .collect();
        let mut stereo = Vec::new();
        for v in &bleed {
            stereo.push(*v);
            stereo.push(*v);
        }
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("clip.wav");
        write(&p, 2, rate, &stereo);
        let removed = clean_wav(&p, &other, &Settings::default()).unwrap();
        assert!(removed > 15.0, "only {removed:.1} dB");
        let r = hound::WavReader::open(&p).unwrap();
        assert_eq!(r.spec().channels, 2);
        assert_eq!(r.len() as usize, stereo.len());
    }

    #[test]
    fn a_copy_that_is_not_float_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("int.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 8_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&p, spec).unwrap();
        w.write_sample(0i16).unwrap();
        w.finalize().unwrap();
        assert!(clean_wav(&p, &[0.0], &Settings::default()).is_err());
    }
}
