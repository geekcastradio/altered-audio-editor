// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Waveform peaks, kept in memory.
//!
//! For each media file we keep the minimum and maximum sample of every
//! block of 256 frames, plus coarser levels of 4,096 and 65,536 frames
//! for zoomed-out views. Values are 16-bit. Every channel is kept, up
//! to 64.
//!
//! Nothing is written to disk: the peaks are built the first time a file
//! is drawn and held while it is in use, so there are no cache files to
//! pile up or go stale. Each entry carries the source file's size and
//! modified time and is built again when either changes, so a file that
//! was saved over is never drawn from its old peaks. A file that changes
//! while it is being read is not remembered, and neither is an empty
//! result. Entries are dropped oldest-used first past a memory budget.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use serde::Serialize;
use thiserror::Error;

use crate::codecs::{AudioReader, ProbeError};

/// Frames per block at each level, finest first.
pub const LEVELS: [u32; 3] = [256, 4096, 65536];
/// The most memory the held peaks may use, in bytes.
const BUDGET: usize = 256 * 1024 * 1024;

/// The levels for a file of `length` frames at `rate`: the standard three,
/// with a finer one in front for files short enough that it stays small
/// (32 frames up to 10 minutes, 64 up to an hour; about 22 MB at most).
pub fn levels_for(length: u64, rate: u32) -> Vec<u32> {
    let minutes = length as f64 / f64::from(rate.max(1)) / 60.0;
    let mut v = Vec::with_capacity(4);
    if length > 0 && minutes <= 10.0 {
        v.push(32);
    } else if length > 0 && minutes <= 60.0 {
        v.push(64);
    }
    v.extend_from_slice(&LEVELS);
    v
}

#[derive(Debug, Error)]
pub enum PeakError {
    #[error("{0}")]
    Decode(#[from] ProbeError),
    #[error("Peaks: {0}")]
    Io(#[from] std::io::Error),
    #[error("That level of the peaks does not exist.")]
    Corrupt,
    #[error("Cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelInfo {
    pub block: u32,
    pub count: u64,
}

/// What the interface needs to draw a file's waveform.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeakSummary {
    pub sample_rate: u32,
    pub channels: u16,
    pub length: u64,
    pub levels: Vec<LevelInfo>,
}

/// The built peaks of one file.
struct Entry {
    stamp: (u64, u64),
    summary: PeakSummary,
    levels: Vec<Vec<i16>>,
}

impl Entry {
    fn bytes(&self) -> usize {
        self.levels.iter().map(|l| l.len() * 2).sum()
    }
}

struct Slot {
    entry: Arc<Entry>,
    used: u64,
}

#[derive(Default)]
struct Store {
    map: HashMap<PathBuf, Slot>,
    tick: u64,
}

fn store() -> &'static Mutex<Store> {
    static STORE: OnceLock<Mutex<Store>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(Store::default()))
}

fn key_of(source: &Path) -> PathBuf {
    fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf())
}

/// The held entry for a file, if it was built from the file as it is now.
fn lookup(key: &Path, stamp: (u64, u64)) -> Option<Arc<Entry>> {
    let mut guard = store().lock().ok()?;
    let s = &mut *guard;
    s.tick += 1;
    let tick = s.tick;
    let slot = s.map.get_mut(key)?;
    if slot.entry.stamp == stamp {
        slot.used = tick;
        return Some(slot.entry.clone());
    }
    // Built from an older version of the file.
    s.map.remove(key);
    None
}

/// Holds an entry, dropping the least recently used ones past the budget.
fn keep(key: PathBuf, entry: Arc<Entry>) {
    // An empty result is never held: the file may still be being written.
    if entry.summary.length == 0 || entry.levels.iter().all(|l| l.is_empty()) {
        return;
    }
    let Ok(mut guard) = store().lock() else {
        return;
    };
    let s = &mut *guard;
    s.tick += 1;
    let tick = s.tick;
    s.map.insert(key.clone(), Slot { entry, used: tick });
    let mut total: usize = s.map.values().map(|x| x.entry.bytes()).sum();
    while total > BUDGET {
        let oldest = s
            .map
            .iter()
            .filter(|(k, _)| **k != key)
            .min_by_key(|(_, x)| x.used)
            .map(|(k, _)| k.clone());
        let Some(k) = oldest else { break };
        if let Some(gone) = s.map.remove(&k) {
            total -= gone.entry.bytes();
        }
    }
}

/// Lets go of a file's held peaks.
pub fn release(source: &Path) {
    if let Ok(mut s) = store().lock() {
        s.map.remove(&key_of(source));
    }
}

fn source_stamp(source: &Path) -> Result<(u64, u64), PeakError> {
    let meta = fs::metadata(source).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => PeakError::Decode(ProbeError::NotFound),
        _ => PeakError::Io(e),
    })?;
    let modified = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    Ok((meta.len(), modified))
}

/// The peaks for a file as it is now: held ones if they match, otherwise built.
/// `progress` gets values from 0 to 1 and returns false to cancel.
fn entry_for(
    source: &Path,
    progress: &mut impl FnMut(f64) -> bool,
) -> Result<Arc<Entry>, PeakError> {
    let key = key_of(source);
    let mut last = None;
    // A file saved over while it is read is read again, a few times at most.
    for _ in 0..3 {
        let stamp = source_stamp(source)?;
        if let Some(e) = lookup(&key, stamp) {
            return Ok(e);
        }
        let entry = Arc::new(build(source, stamp, &mut *progress)?);
        if source_stamp(source).is_ok_and(|now| now == stamp) {
            keep(key.clone(), entry.clone());
            return Ok(entry);
        }
        last = Some(entry);
    }
    Ok(last.expect("the loop ran"))
}

/// The summary of a file's peaks, building them first if needed.
/// `progress` gets values from 0 to 1 and returns false to cancel.
pub fn ensure(
    source: &Path,
    mut progress: impl FnMut(f64) -> bool,
) -> Result<PeakSummary, PeakError> {
    Ok(entry_for(source, &mut progress)?.summary.clone())
}

fn build(
    source: &Path,
    stamp: (u64, u64),
    progress: &mut impl FnMut(f64) -> bool,
) -> Result<Entry, PeakError> {
    let info = crate::codecs::probe(source)?;
    let channels = usize::from(info.channels).clamp(1, 64);
    let mut reader = AudioReader::open(source, channels, None)?;
    let blocks = levels_for(info.length, info.sample_rate);
    let block0 = blocks[0] as usize;

    // Level 0, built while streaming.
    let mut level0: Vec<i16> =
        Vec::with_capacity((info.length as usize / block0 + 1) * channels * 2);
    let mut acc_min = vec![f32::MAX; channels];
    let mut acc_max = vec![f32::MIN; channels];
    let mut in_block = 0usize;
    let mut frames_done: u64 = 0;
    let mut buf = Vec::with_capacity(16384);
    let mut last_report = 0u64;

    let flush = |level0: &mut Vec<i16>, acc_min: &mut [f32], acc_max: &mut [f32]| {
        for c in 0..acc_min.len() {
            level0.push(to_i16(acc_min[c]));
            level0.push(to_i16(acc_max[c]));
            acc_min[c] = f32::MAX;
            acc_max[c] = f32::MIN;
        }
    };

    loop {
        buf.clear();
        if !reader.read_into(&mut buf)? {
            break;
        }
        for frame in buf.chunks_exact(channels) {
            for (c, v) in frame.iter().enumerate() {
                acc_min[c] = acc_min[c].min(*v);
                acc_max[c] = acc_max[c].max(*v);
            }
            in_block += 1;
            if in_block == block0 {
                flush(&mut level0, &mut acc_min, &mut acc_max);
                in_block = 0;
            }
        }
        frames_done += (buf.len() / channels) as u64;
        if info.length > 0 && frames_done - last_report > 262_144 {
            last_report = frames_done;
            if !progress((frames_done as f64 / info.length as f64).min(0.99)) {
                return Err(PeakError::Cancelled);
            }
        }
    }
    if in_block > 0 {
        flush(&mut level0, &mut acc_min, &mut acc_max);
    }

    let mut levels: Vec<Vec<i16>> = vec![level0];
    for w in blocks.windows(2) {
        let factor = (w[1] / w[0]) as usize;
        let prev = levels.last().expect("level 0 exists");
        levels.push(merge(prev, channels, factor));
    }

    let summary = PeakSummary {
        sample_rate: info.sample_rate,
        channels: channels as u16,
        // The probe's length accounts for encoder padding; decoding may not.
        length: if info.length > 0 {
            info.length
        } else {
            frames_done
        },
        levels: blocks
            .iter()
            .zip(&levels)
            .map(|(b, d)| LevelInfo {
                block: *b,
                count: (d.len() / (channels * 2)) as u64,
            })
            .collect(),
    };
    progress(1.0);
    Ok(Entry {
        stamp,
        summary,
        levels,
    })
}

pub(crate) fn to_i16(v: f32) -> i16 {
    if v == f32::MAX || v == f32::MIN {
        return 0;
    }
    (v.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// Combines `factor` blocks into one.
fn merge(prev: &[i16], channels: usize, factor: usize) -> Vec<i16> {
    let stride = channels * 2;
    let mut out = Vec::with_capacity((prev.len() / stride / factor + 1) * stride);
    for group in prev.chunks(stride * factor) {
        for c in 0..channels {
            let mut lo = i16::MAX;
            let mut hi = i16::MIN;
            for b in group.chunks_exact(stride) {
                lo = lo.min(b[c * 2]);
                hi = hi.max(b[c * 2 + 1]);
            }
            out.push(lo);
            out.push(hi);
        }
    }
    out
}

/// Raw bytes of one level: per block, per channel, min then max, i16 LE.
pub fn read_level(source: &Path, level: usize) -> Result<Vec<u8>, PeakError> {
    let entry = entry_for(source, &mut |_| true)?;
    let data = entry.levels.get(level).ok_or(PeakError::Corrupt)?;
    Ok(data.iter().flat_map(|v| v.to_le_bytes()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn media(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("media")
            .join(name)
    }

    #[test]
    fn finer_levels_only_for_shorter_files() {
        assert_eq!(levels_for(48_000 * 60, 48_000), vec![32, 256, 4096, 65536]);
        assert_eq!(
            levels_for(48_000 * 60 * 30, 48_000),
            vec![64, 256, 4096, 65536]
        );
        assert_eq!(
            levels_for(48_000 * 60 * 120, 48_000),
            vec![256, 4096, 65536]
        );
    }

    #[test]
    fn builds_reads_and_reuses_the_peaks() {
        let src = media("stereo_48k_16.wav");
        release(&src);
        let mut calls = 0;
        let s = ensure(&src, |_| {
            calls += 1;
            true
        })
        .unwrap();
        assert_eq!((s.sample_rate, s.channels, s.length), (48_000, 2, 24_000));
        // A short file starts with the fine 32-frame level.
        assert_eq!(
            s.levels[0],
            LevelInfo {
                block: 32,
                count: 750
            }
        );
        assert_eq!(
            s.levels[1],
            LevelInfo {
                block: 256,
                count: 94
            }
        );
        assert_eq!(s.levels[2].count, 6);
        assert_eq!(s.levels[3].count, 1);

        let bytes = read_level(&src, 1).unwrap();
        assert_eq!(bytes.len(), 94 * 2 * 4);
        let vals: Vec<i16> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| i16::from_le_bytes([b[0], b[1]]))
            .collect();
        // A sine tone swings both ways in every block.
        assert!(vals[0] < -1000 && vals[1] > 1000, "{:?}", &vals[..4]);

        // Second call uses what is held: no progress reports.
        let mut again = 0;
        let s2 = ensure(&src, |_| {
            again += 1;
            true
        })
        .unwrap();
        assert_eq!(s2, s);
        assert_eq!(again, 0);
        assert!(calls >= 1);
    }

    #[test]
    fn coarse_levels_cover_fine_levels() {
        let src = media("tone.mp3");
        ensure(&src, |_| true).unwrap();
        let fine = read_level(&src, 0).unwrap();
        let coarse = read_level(&src, 2).unwrap();
        let v = |b: &[u8], i: usize| i16::from_le_bytes([b[i * 2], b[i * 2 + 1]]);
        let fine_min = (0..fine.len() / 2)
            .step_by(2)
            .map(|i| v(&fine, i))
            .min()
            .unwrap();
        assert_eq!(v(&coarse, 0), fine_min);
    }

    fn write_wav(path: &Path, frames: usize, amp: i16) {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec).unwrap();
        for i in 0..frames {
            w.write_sample(if i % 2 == 0 { amp } else { -amp }).unwrap();
        }
        w.finalize().unwrap();
    }

    #[test]
    fn a_file_saved_over_is_drawn_from_its_new_contents() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("take.wav");
        write_wav(&src, 4_800, 1_000);
        let first = ensure(&src, |_| true).unwrap();
        assert_eq!(first.length, 4_800);
        write_wav(&src, 9_600, 8_000);
        // The summary and the levels both come from the new file.
        let second = ensure(&src, |_| true).unwrap();
        assert_eq!(second.length, 9_600);
        let level = read_level(&src, 1).unwrap();
        assert_eq!(level.len() as u64, second.levels[1].count * 4);
        let max = i16::from_le_bytes([level[2], level[3]]);
        assert!(max > 4_000, "{max}");
    }

    #[test]
    fn nothing_is_written_beside_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("take.wav");
        write_wav(&src, 4_800, 1_000);
        ensure(&src, |_| true).unwrap();
        read_level(&src, 0).unwrap();
        // Only the audio file is in the folder: no cache files anywhere.
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn an_empty_result_is_not_held() {
        let key = PathBuf::from("never-held-empty.wav");
        let entry = Entry {
            stamp: (1, 1),
            summary: PeakSummary {
                sample_rate: 48_000,
                channels: 1,
                length: 0,
                levels: vec![],
            },
            levels: vec![vec![]],
        };
        keep(key.clone(), Arc::new(entry));
        assert!(lookup(&key, (1, 1)).is_none());
    }

    #[test]
    fn release_lets_go_and_the_next_call_builds_again() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.wav");
        write_wav(&src, 4_800, 1_000);
        ensure(&src, |_| true).unwrap();
        assert!(lookup(&key_of(&src), source_stamp(&src).unwrap()).is_some());
        release(&src);
        assert!(lookup(&key_of(&src), source_stamp(&src).unwrap()).is_none());
        assert_eq!(ensure(&src, |_| true).unwrap().length, 4_800);
    }

    #[test]
    fn multichannel_peaks_read_back() {
        // A 12-channel file: each channel at its own level, so a mix-up shows.
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("twelve.wav");
        let spec = hound::WavSpec {
            channels: 12,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&src, spec).unwrap();
        for i in 0..4_800 {
            for c in 0..12 {
                let sign = if i % 2 == 0 { 1 } else { -1 };
                w.write_sample((sign * 1_000 * (c + 1)) as i16).unwrap();
            }
        }
        w.finalize().unwrap();
        let s = ensure(&src, |_| true).unwrap();
        assert_eq!(s.channels, 12);
        let level = read_level(&src, 0).unwrap();
        let max =
            |c: usize| i16::from_le_bytes([level[(c * 2 + 1) * 2], level[(c * 2 + 1) * 2 + 1]]);
        assert!(max(11) > max(0) * 10);
    }

    #[test]
    fn merge_takes_extremes() {
        // Two blocks, mono: (-5, 3) and (-2, 9) merge to (-5, 9).
        assert_eq!(merge(&[-5, 3, -2, 9], 1, 2), vec![-5, 9]);
    }
}
