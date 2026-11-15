// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Opens RF64, BW64 and Wave64 files for the decoder. The decoder reads plain WAV, so these files
//! are shown to it as a plain WAV: a short synthetic header, then the audio bytes straight from
//! the file. Nothing is copied or written to disk. A file past 4 GB gets a header whose sizes say
//! "streamed", which the decoder reads to the end of the data.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use symphonia::core::io::MediaSource;

use super::wavlong::{header, parse, Container, Info};

/// How much of the file's head is read to find the audio. Broadcast files carry large metadata
/// chunks before the audio, so this is generous.
const HEAD: usize = 256 * 1024;

/// A source for the decoder, and what is known about it without decoding.
pub struct Opened {
    pub source: Box<dyn MediaSource>,
    /// Set for RF64, BW64 and Wave64: the exact length in frames and the container's name.
    pub long: Option<(u64, &'static str)>,
}

/// Opens `path` as a decoder source. Plain files are returned as they are.
pub fn open_source(path: &Path) -> io::Result<Opened> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let mut head = vec![0u8; HEAD.min(len as usize)];
    file.read_exact(&mut head)?;
    file.seek(SeekFrom::Start(0))?;
    let is_long =
        head.starts_with(b"RF64") || head.starts_with(b"BW64") || head.starts_with(&GUID_RIFF);
    if !is_long {
        return Ok(Opened {
            source: Box::new(file),
            long: None,
        });
    }
    match parse(&head, len) {
        Ok(info) => {
            let name = if info.container == Container::W64 {
                "W64"
            } else {
                "RF64"
            };
            let frames = info.frames();
            Ok(Opened {
                source: Box::new(VirtualWav::new(file, &info)),
                long: Some((frames, name)),
            })
        }
        // Let the decoder report what it cannot read.
        Err(_) => Ok(Opened {
            source: Box::new(file),
            long: None,
        }),
    }
}

const GUID_RIFF: [u8; 16] = [
    0x72, 0x69, 0x66, 0x66, 0x2E, 0x91, 0xCF, 0x11, 0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00,
];

/// A long file seen as a plain WAV.
pub struct VirtualWav {
    head: Vec<u8>,
    file: File,
    data_offset: u64,
    data_bytes: u64,
    pos: u64,
    /// Where the file's own cursor is, so seeks are only made when needed.
    file_pos: Option<u64>,
}

impl VirtualWav {
    pub fn new(file: File, info: &Info) -> Self {
        let mut head = header(&info.layout, info.data_bytes, Container::Wav);
        // Past 4 GB the sizes cannot be written; the maximum value means "to the end of the file".
        if info.data_bytes >= u64::from(u32::MAX) - 4096 {
            head[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
            let n = head.len();
            head[n - 4..].copy_from_slice(&u32::MAX.to_le_bytes());
        }
        Self {
            head,
            file,
            data_offset: info.data_offset,
            data_bytes: info.data_bytes,
            pos: 0,
            file_pos: None,
        }
    }

    fn total(&self) -> u64 {
        self.head.len() as u64 + self.data_bytes
    }
}

impl Read for VirtualWav {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let hl = self.head.len() as u64;
        if self.pos < hl {
            let at = self.pos as usize;
            let n = buf.len().min(self.head.len() - at);
            buf[..n].copy_from_slice(&self.head[at..at + n]);
            self.pos += n as u64;
            return Ok(n);
        }
        let into = self.pos - hl;
        if into >= self.data_bytes {
            return Ok(0);
        }
        let want = (self.data_bytes - into).min(buf.len() as u64) as usize;
        let at = self.data_offset + into;
        if self.file_pos != Some(at) {
            self.file.seek(SeekFrom::Start(at))?;
        }
        let n = self.file.read(&mut buf[..want])?;
        self.pos += n as u64;
        self.file_pos = Some(at + n as u64);
        Ok(n)
    }
}

impl Seek for VirtualWav {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        let next = match to {
            SeekFrom::Start(n) => Some(n),
            SeekFrom::Current(d) => self.pos.checked_add_signed(d),
            SeekFrom::End(d) => self.total().checked_add_signed(d),
        };
        match next {
            Some(n) => {
                self.pos = n;
                Ok(n)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek before the start",
            )),
        }
    }
}

impl MediaSource for VirtualWav {
    fn is_seekable(&self) -> bool {
        true
    }
    fn byte_len(&self) -> Option<u64> {
        Some(self.total())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::wavlong::{header, pad_after, Container, Layout};
    use symphonia::core::codecs::audio::AudioDecoderOptions;
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::formats::{FormatOptions, TrackType};
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;

    fn layout() -> Layout {
        Layout {
            channels: 2,
            rate: 48_000,
            bits: 24,
            float: false,
            mask: 0,
        }
    }

    /// 24-bit stereo: sample k of channel c is (k*2+c) * 256, so every value is checkable.
    fn audio(frames: usize) -> Vec<u8> {
        let mut v = Vec::new();
        for k in 0..frames {
            for c in 0..2 {
                let s = (((k * 2 + c) % 20_000) as i32) * 256;
                v.extend_from_slice(&s.to_le_bytes()[..3]);
            }
        }
        v
    }

    fn write(dir: &std::path::Path, name: &str, c: Container, frames: usize) -> std::path::PathBuf {
        let a = audio(frames);
        let mut f = header(&layout(), a.len() as u64, c);
        f.extend_from_slice(&a);
        f.extend(std::iter::repeat_n(0u8, pad_after(a.len() as u64, c)));
        let p = dir.join(name);
        std::fs::write(&p, f).unwrap();
        p
    }

    fn decode_all(p: &std::path::Path) -> (u64, Vec<f32>) {
        let o = open_source(p).unwrap();
        let mss = MediaSourceStream::new(o.source, Default::default());
        let mut hint = Hint::new();
        hint.with_extension("wav");
        let mut fmt = symphonia::default::get_probe()
            .probe(
                &hint,
                mss,
                FormatOptions::default(),
                MetadataOptions::default(),
            )
            .unwrap();
        let track = fmt.default_track(TrackType::Audio).unwrap().clone();
        let params = track
            .codec_params
            .as_ref()
            .unwrap()
            .audio()
            .unwrap()
            .clone();
        let mut dec = symphonia::default::get_codecs()
            .make_audio_decoder(&params, &AudioDecoderOptions::default())
            .unwrap();
        let mut frames = 0u64;
        let mut samples = Vec::new();
        while let Ok(Some(pkt)) = fmt.next_packet() {
            if pkt.track_id != track.id {
                continue;
            }
            let buf = dec.decode(&pkt).unwrap();
            let mut tmp = vec![0f32; buf.samples_interleaved()];
            buf.copy_to_slice_interleaved(&mut tmp);
            frames += (tmp.len() / 2) as u64;
            samples.extend_from_slice(&tmp);
        }
        (frames, samples)
    }

    #[test]
    fn rf64_and_w64_decode_like_wav() {
        let dir = tempfile::tempdir().unwrap();
        let frames = 30_000;
        let want = audio(frames);
        for (name, c) in [
            ("a.wav", Container::Wav),
            ("a.rf64", Container::Rf64),
            ("a.w64", Container::W64),
        ] {
            let p = write(dir.path(), name, c, frames);
            let o = open_source(&p).unwrap();
            assert_eq!(o.long.is_some(), c != Container::Wav, "{name}");
            if let Some((n, label)) = o.long {
                assert_eq!(n, frames as u64);
                assert_eq!(label, if c == Container::W64 { "W64" } else { "RF64" });
            }
            drop(o);
            let (n, got) = decode_all(&p);
            assert_eq!(n, frames as u64, "{name}");
            // Compare the 24-bit values.
            for i in [0usize, 1, 2, 1000, frames * 2 - 1] {
                let w = i32::from_le_bytes([0, want[i * 3], want[i * 3 + 1], want[i * 3 + 2]]) >> 8;
                let g = (got[i] * 8_388_608.0).round() as i32;
                assert_eq!(g, w, "{name} sample {i}");
            }
        }
    }

    #[test]
    fn a_header_claiming_more_than_4_gb_still_plays_to_the_end() {
        // The data is short; the header (ds64) says 6 GB. The reader trusts the file's real size.
        let dir = tempfile::tempdir().unwrap();
        let a = audio(10_000);
        let mut f = header(&layout(), 6_000_000_000, Container::Rf64);
        f.extend_from_slice(&a);
        let p = dir.path().join("big.rf64");
        std::fs::write(&p, f).unwrap();
        let o = open_source(&p).unwrap();
        assert_eq!(o.long.unwrap().0, 10_000);
        drop(o);
        let (n, _) = decode_all(&p);
        assert_eq!(n, 10_000);
    }

    #[test]
    fn plain_files_are_not_wrapped() {
        let dir = tempfile::tempdir().unwrap();
        let p = write(dir.path(), "plain.wav", Container::Wav, 100);
        assert!(open_source(&p).unwrap().long.is_none());
    }

    #[test]
    fn seeking_a_long_file_lands_on_the_right_frame() {
        use symphonia::core::formats::{SeekMode, SeekTo};
        use symphonia::core::units::Time;
        let dir = tempfile::tempdir().unwrap();
        let frames = 96_000;
        let p = write(dir.path(), "s.rf64", Container::Rf64, frames);
        let o = open_source(&p).unwrap();
        let mss = MediaSourceStream::new(o.source, Default::default());
        let mut hint = Hint::new();
        hint.with_extension("wav");
        let mut fmt = symphonia::default::get_probe()
            .probe(
                &hint,
                mss,
                FormatOptions::default(),
                MetadataOptions::default(),
            )
            .unwrap();
        let track = fmt.default_track(TrackType::Audio).unwrap().clone();
        let params = track
            .codec_params
            .as_ref()
            .unwrap()
            .audio()
            .unwrap()
            .clone();
        let mut dec = symphonia::default::get_codecs()
            .make_audio_decoder(&params, &AudioDecoderOptions::default())
            .unwrap();
        let seeked = fmt
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: Time::try_from_secs_f64(1.0).unwrap(),
                    track_id: Some(track.id),
                },
            )
            .unwrap();
        dec.reset();
        let pkt = loop {
            let p = fmt.next_packet().unwrap().unwrap();
            if p.track_id == track.id {
                break p;
            }
        };
        let buf = dec.decode(&pkt).unwrap();
        let mut tmp = vec![0f32; buf.samples_interleaved()];
        buf.copy_to_slice_interleaved(&mut tmp);
        let start = seeked.actual_ts.get() as usize;
        let want = audio(frames);
        let w =
            i32::from_le_bytes([0, want[start * 6], want[start * 6 + 1], want[start * 6 + 2]]) >> 8;
        assert_eq!((tmp[0] * 8_388_608.0).round() as i32, w);
        assert!((48_000 - 4096..=48_000).contains(&start));
    }

    #[test]
    fn a_file_from_wav_out_plays_like_any_wav() {
        use crate::codecs::wavlong::WavOut;
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("out.wav");
        let mut w = WavOut::create(&p, 2, 48_000, 24, false).unwrap();
        for k in 0..5_000i32 {
            w.put_i24(k * 100).unwrap();
            w.put_i24(-k * 100).unwrap();
        }
        w.finish().unwrap();
        let (n, got) = decode_all(&p);
        assert_eq!(n, 5_000);
        assert_eq!((got[2 * 40] * 8_388_608.0).round() as i32, 4_000);
        assert_eq!((got[2 * 40 + 1] * 8_388_608.0).round() as i32, -4_000);
    }
}
