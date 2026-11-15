// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Multichannel WAV for 3D Space exports: the extensible WAV header
//! with its channel mask, so other software knows which channel is
//! which speaker (left, right, center, low frequency, surrounds,
//! heights). 16 or 24-bit PCM, or 32-bit float. A file that grows past
//! 4 GB becomes RF64, which long multichannel exports reach quickly.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::codecs::wavlong::{close_header, open_header, Layout};

pub struct MultiWav {
    out: BufWriter<File>,
    layout: Layout,
    channels: u16,
    bits: u16,
    frames: u64,
    buf: Vec<u8>,
    /// Triangular dither for integer output.
    seed: u32,
}

impl MultiWav {
    pub fn create(
        path: &Path,
        rate: u32,
        channels: u16,
        mask: u32,
        bits: u16,
    ) -> std::io::Result<MultiWav> {
        let bits = if [16, 24, 32].contains(&bits) {
            bits
        } else {
            24
        };
        let layout = Layout {
            channels,
            rate,
            bits,
            float: bits == 32,
            mask,
        };
        let mut out = BufWriter::new(File::create(path)?);
        // The sizes are filled in at the end, when the length is known.
        out.write_all(&open_header(&layout))?;
        Ok(MultiWav {
            out,
            layout,
            channels,
            bits,
            frames: 0,
            buf: Vec::new(),
            seed: 0x9E37_79B9,
        })
    }

    fn noise(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        self.seed as f32 / u32::MAX as f32
    }

    /// Interleaved samples, `channels` to a frame.
    pub fn write(&mut self, samples: &[f32]) -> std::io::Result<()> {
        self.buf.clear();
        for &v in samples {
            match self.bits {
                32 => self.buf.extend_from_slice(&v.to_le_bytes()),
                24 => {
                    let d = (self.noise() - self.noise()) / 8_388_608.0;
                    let s = ((v + d).clamp(-1.0, 1.0) * 8_388_607.0).round() as i32;
                    self.buf.extend_from_slice(&s.to_le_bytes()[..3]);
                }
                _ => {
                    let d = (self.noise() - self.noise()) / 32_768.0;
                    let s = ((v + d).clamp(-1.0, 1.0) * 32_767.0).round() as i16;
                    self.buf.extend_from_slice(&s.to_le_bytes());
                }
            }
        }
        self.frames += (samples.len() / usize::from(self.channels)) as u64;
        self.out.write_all(&self.buf)
    }

    /// Writes the sizes into the header. Returns the frames written.
    pub fn finish(mut self) -> std::io::Result<u64> {
        let data = self.frames * u64::from(self.channels) * u64::from(self.bits / 8);
        if data % 2 == 1 {
            self.out.write_all(&[0])?;
        }
        self.out.flush()?;
        let mut f = self.out.into_inner().map_err(|e| e.into_error())?;
        close_header(&mut f, &self.layout, data)?;
        f.flush()?;
        Ok(self.frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_readable_multichannel_file_with_its_mask() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bed.wav");
        let mut w = MultiWav::create(&path, 48_000, 12, 0x2D63F, 32).unwrap();
        let frame: Vec<f32> = (0..12).map(|c| c as f32 / 20.0).collect();
        for _ in 0..100 {
            w.write(&frame).unwrap();
        }
        assert_eq!(w.finish().unwrap(), 100);
        let r = hound::WavReader::open(&path).unwrap();
        assert_eq!(r.spec().channels, 12);
        assert_eq!(r.len(), 1_200);
        let s: Vec<f32> = r.into_samples::<f32>().map(Result::unwrap).collect();
        assert_eq!(s[13], 0.05);
        let bytes = std::fs::read(&path).unwrap();
        // The mask sits 20 bytes into the fmt chunk's body.
        let fmt = bytes.windows(4).position(|w| w == b"fmt ").unwrap();
        let at = fmt + 8 + 20;
        assert_eq!(
            u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()),
            0x2D63F
        );
    }
}
