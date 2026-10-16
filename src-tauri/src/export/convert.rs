// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Converting one audio file to another format: any file the app opens,
//! to WAV, FLAC or MP3. Used by Batch Convert and by Save As in the
//! Waveform editor.
//!
//! The new file is written next to its destination under a temporary
//! name and moved into place only when it is complete, so a failed or
//! stopped conversion never leaves a half-written file, and converting a
//! file onto itself is safe.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::encode::{EncodeError, OutputFormat, Writer};
use crate::codecs::{probe, AudioReader};

#[derive(Debug, Clone, PartialEq)]
pub enum Converted {
    Done { frames: u64 },
    Cancelled,
}

/// The temporary name a conversion writes to before moving into place.
fn partial_path(dest: &Path) -> PathBuf {
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    dest.with_file_name(format!(".{name}.partial"))
}

/// Converts `src` to `dest`. `progress` gets (done, total) source frames.
/// Refuses to replace an existing `dest` unless `overwrite`.
pub fn convert_file(
    src: &Path,
    dest: &Path,
    fmt: &OutputFormat,
    overwrite: bool,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64, u64),
) -> Result<Converted, String> {
    if dest.exists() && !overwrite {
        return Err("A file with that name is already there.".into());
    }
    let info = probe(src).map_err(|e| e.to_string())?;
    let bits = info.bit_depth.unwrap_or(24).min(32) as u16;
    let resolved = fmt
        .resolve(info.sample_rate, info.channels, bits)
        .map_err(|e| e.to_string())?;
    let mut reader =
        AudioReader::open(src, usize::from(info.channels), None).map_err(|e| e.to_string())?;
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = partial_path(dest);
    let result = (|| -> Result<Converted, String> {
        let enc = |e: EncodeError| e.to_string();
        let mut w =
            Writer::create(&tmp, &resolved, info.sample_rate, info.channels).map_err(enc)?;
        let mut buf = Vec::new();
        let mut done = 0u64;
        let ch = usize::from(info.channels.max(1));
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Ok(Converted::Cancelled);
            }
            buf.clear();
            if !reader.read_into(&mut buf).map_err(|e| e.to_string())? {
                break;
            }
            w.write(&buf).map_err(enc)?;
            done += (buf.len() / ch) as u64;
            progress(done.min(info.length), info.length);
        }
        let frames = w.finish().map_err(enc)?;
        Ok(Converted::Done { frames })
    })();
    match result {
        Ok(Converted::Done { frames }) => {
            if overwrite && dest.exists() {
                // Windows won't rename over an existing file.
                std::fs::remove_file(dest).map_err(|e| e.to_string())?;
            }
            std::fs::rename(&tmp, dest).map_err(|e| e.to_string())?;
            Ok(Converted::Done { frames })
        }
        other => {
            let _ = std::fs::remove_file(&tmp);
            other
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::encode::FormatKind;

    fn tone(path: &Path, frames: u32) {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec).unwrap();
        for i in 0..frames {
            let v = ((i as f32 * 0.03).sin() * 12_000.0) as i16;
            w.write_sample(v).unwrap();
            w.write_sample(v).unwrap();
        }
        w.finalize().unwrap();
    }

    #[test]
    fn converts_and_protects_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("in.wav");
        tone(&src, 48_000);
        let dest = dir.path().join("out").join("in.flac");
        let fmt = OutputFormat {
            kind: FormatKind::Flac,
            ..Default::default()
        };
        let mut last = (0, 0);
        let r = convert_file(&src, &dest, &fmt, false, &AtomicBool::new(false), |d, t| {
            last = (d, t)
        })
        .unwrap();
        assert_eq!(r, Converted::Done { frames: 48_000 });
        assert_eq!(last, (48_000, 48_000));
        let info = probe(&dest).unwrap();
        assert_eq!(
            (info.length, info.channels, info.bit_depth),
            (48_000, 2, Some(16))
        );
        assert!(
            convert_file(&src, &dest, &fmt, false, &AtomicBool::new(false), |_, _| {}).is_err()
        );
        assert!(!partial_path(&dest).exists());
    }

    #[test]
    fn converts_a_file_onto_itself_and_stops_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("same.wav");
        tone(&src, 24_000);
        let fmt = OutputFormat {
            kind: FormatKind::Wav,
            bit_depth: Some(24),
            channels: Some(1),
            ..Default::default()
        };
        convert_file(&src, &src, &fmt, true, &AtomicBool::new(false), |_, _| {}).unwrap();
        let r = hound::WavReader::open(&src).unwrap();
        assert_eq!(
            (r.spec().channels, r.spec().bits_per_sample, r.len()),
            (1, 24, 24_000)
        );

        let dest = dir.path().join("never.mp3");
        let r = convert_file(
            &src,
            &dest,
            &OutputFormat {
                kind: FormatKind::Mp3,
                ..Default::default()
            },
            false,
            &AtomicBool::new(true),
            |_, _| {},
        )
        .unwrap();
        assert_eq!(r, Converted::Cancelled);
        assert!(!dest.exists() && !partial_path(&dest).exists());
    }
}
