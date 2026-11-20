// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! AAC writing with a pure-Rust encoder, for builds with the `pure-aac`
//! feature. It has the same four calls as the system writer in
//! `altered-os-aac` (`create`, `create_adts`, `write`, `finish`), so the
//! export code does not know which one it has.
//!
//! The encoder is `filmcraft-aac` (MIT OR Apache-2.0, no `unsafe`, written
//! from the standard without reading other encoders' source). The M4A
//! container is written by the `mp4` crate the app already uses to put a mix
//! back into a video.
//!
//! Differences from the Windows system writer:
//! - It works on every system, so AAC export is no longer Windows only.
//! - Any of the AAC sample rates and 1 to 8 channels are accepted.
//! - The M4A has no edit list, so players show about 21 ms of encoder
//!   delay (1024 samples) at the start. The sound is not changed.

use std::fs::File;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

use filmcraft_aac::{Encoder, EncoderConfig, FRAME_LEN};

/// True where this build can write AAC.
pub const AVAILABLE: bool = true;

/// Never shown: AAC is available. Kept so both writers offer the same names.
pub const UNAVAILABLE: &str = "";

/// Access units per chunk in the M4A sample table.
const PER_CHUNK: u32 = 32;

/// The M4A box layout is small and fixed, so it is written here rather
/// than by a general MP4 library: `ftyp`, `mdat` (the access units, in
/// order), then `moov` with one audio track. The `moov` goes last, so the
/// audio is streamed to disk as it is encoded.
struct M4a {
    file: BufWriter<File>,
    /// Where the `mdat` box starts, to write its size at the end.
    mdat_at: u64,
    /// First byte of audio data (after the `mdat` header).
    data_at: u64,
    sizes: Vec<u32>,
    bytes: u64,
    rate: u32,
    channels: u16,
    kbps: u16,
}

enum Sink {
    M4a(Box<M4a>),
    Adts(BufWriter<File>),
}

pub struct AacWriter {
    enc: Box<Encoder>,
    sink: Sink,
    channels: usize,
    /// Samples waiting for a whole block, one list per channel.
    planar: Vec<Vec<f32>>,
    /// Frames received, for the length of the track.
    frames_in: u64,
}

fn io(e: std::io::Error) -> String {
    e.to_string()
}

fn make_box(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut b = Vec::with_capacity(8 + body.len());
    b.extend_from_slice(&((8 + body.len()) as u32).to_be_bytes());
    b.extend_from_slice(kind);
    b.extend_from_slice(body);
    b
}

/// A box with a version and flags word (both zero) before the body.
fn full_box(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut b = vec![0, 0, 0, 0];
    b.extend_from_slice(body);
    make_box(kind, &b)
}

fn be32(v: u32) -> [u8; 4] {
    v.to_be_bytes()
}

/// An MPEG-4 descriptor: tag, length (one byte; every one here is under 128), body.
fn descriptor(tag: u8, body: &[u8]) -> Vec<u8> {
    let mut d = vec![tag, body.len() as u8];
    d.extend_from_slice(body);
    d
}

impl M4a {
    fn esds(&self, asc: &[u8]) -> Vec<u8> {
        let bps = u32::from(self.kbps) * 1000;
        let mut dec = vec![0x40, 0x15, 0, 0, 0]; // AAC, audio stream, buffer size
        dec.extend_from_slice(&be32(bps)); // max bit rate
        dec.extend_from_slice(&be32(bps)); // average bit rate
        dec.extend(descriptor(0x05, asc)); // the decoder's AudioSpecificConfig
        let mut es = vec![0, 1, 0]; // ES id 1, no flags
        es.extend(descriptor(0x04, &dec));
        es.extend(descriptor(0x06, &[0x02])); // SL config: the MP4 predefined one
        full_box(b"esds", &descriptor(0x03, &es))
    }

    fn moov(&self, asc: &[u8], input_frames: u64) -> Vec<u8> {
        let rate = self.rate;
        let n = self.sizes.len() as u32;
        let media_len = u64::from(n) * FRAME_LEN as u64;
        // The track plays the frames that were written, after the encoder's start delay.
        let play = input_frames.min(media_len.saturating_sub(FRAME_LEN as u64));
        let movie_ms = (play * 1000 / u64::from(rate)) as u32;

        let mut mvhd = Vec::new();
        mvhd.extend_from_slice(&[0; 8]); // created, modified
        mvhd.extend_from_slice(&be32(1000));
        mvhd.extend_from_slice(&be32(movie_ms));
        mvhd.extend_from_slice(&be32(0x0001_0000)); // rate 1.0
        mvhd.extend_from_slice(&[0x01, 0x00, 0, 0]); // volume 1.0, reserved
        mvhd.extend_from_slice(&[0; 8]);
        for m in [0x0001_0000u32, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x4000_0000] {
            mvhd.extend_from_slice(&be32(m));
        }
        mvhd.extend_from_slice(&[0; 24]); // pre-defined
        mvhd.extend_from_slice(&be32(2)); // next track id

        let mut tkhd = Vec::new();
        tkhd.extend_from_slice(&[0; 8]);
        tkhd.extend_from_slice(&be32(1)); // track id
        tkhd.extend_from_slice(&[0; 4]);
        tkhd.extend_from_slice(&be32(movie_ms));
        tkhd.extend_from_slice(&[0; 8]);
        tkhd.extend_from_slice(&[0, 0, 0, 0, 0x01, 0x00, 0, 0]); // layer, group, volume 1.0
        for m in [0x0001_0000u32, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x4000_0000] {
            tkhd.extend_from_slice(&be32(m));
        }
        tkhd.extend_from_slice(&[0; 8]); // width, height
                                         // Flags 3: the track is enabled and in the movie.
        let tkhd = make_box(b"tkhd", &[vec![0, 0, 0, 3], tkhd].concat());

        // Skip the encoder's start delay so the file plays exactly what went in.
        let mut elst = be32(1).to_vec();
        elst.extend_from_slice(&be32(movie_ms));
        elst.extend_from_slice(&be32(FRAME_LEN as u32));
        elst.extend_from_slice(&be32(0x0001_0000));
        let edts = make_box(b"edts", &full_box(b"elst", &elst));

        let mut mdhd = Vec::new();
        mdhd.extend_from_slice(&[0; 8]);
        mdhd.extend_from_slice(&be32(rate));
        mdhd.extend_from_slice(&be32(media_len as u32));
        mdhd.extend_from_slice(&[0x55, 0xC4, 0, 0]); // language "und"
        let mdhd = full_box(b"mdhd", &mdhd);

        let mut hdlr = vec![0; 4];
        hdlr.extend_from_slice(b"soun");
        hdlr.extend_from_slice(&[0; 12]);
        hdlr.extend_from_slice(b"SoundHandler\0");
        let hdlr = full_box(b"hdlr", &hdlr);

        let smhd = full_box(b"smhd", &[0; 4]);
        let url = full_box(b"url ", &[]);
        let mut url = url;
        url[11] = 1; // flags: the data is in this file
        let mut dref = be32(1).to_vec();
        dref.extend_from_slice(&url);
        let dinf = make_box(b"dinf", &full_box(b"dref", &dref));

        let mut mp4a = vec![0; 6];
        mp4a.extend_from_slice(&[0, 1]); // data reference 1
        mp4a.extend_from_slice(&[0; 8]);
        mp4a.extend_from_slice(&self.channels.to_be_bytes());
        mp4a.extend_from_slice(&[0, 16, 0, 0, 0, 0]); // 16-bit, pre-defined, reserved
        mp4a.extend_from_slice(&(rate.min(65_535) as u16).to_be_bytes());
        mp4a.extend_from_slice(&[0, 0]);
        mp4a.extend(self.esds(asc));
        let mut stsd = be32(1).to_vec();
        stsd.extend(make_box(b"mp4a", &mp4a));
        let stsd = full_box(b"stsd", &stsd);

        let mut stts = be32(1).to_vec();
        stts.extend_from_slice(&be32(n));
        stts.extend_from_slice(&be32(FRAME_LEN as u32));
        let stts = full_box(b"stts", &stts);

        let full_chunks = n / PER_CHUNK;
        let rest = n % PER_CHUNK;
        let mut runs: Vec<(u32, u32)> = Vec::new();
        if full_chunks > 0 {
            runs.push((1, PER_CHUNK));
        }
        if rest > 0 {
            runs.push((full_chunks + 1, rest));
        }
        let mut stsc = be32(runs.len() as u32).to_vec();
        for (first, per) in &runs {
            stsc.extend_from_slice(&be32(*first));
            stsc.extend_from_slice(&be32(*per));
            stsc.extend_from_slice(&be32(1));
        }
        let stsc = full_box(b"stsc", &stsc);

        let mut stsz = be32(0).to_vec();
        stsz.extend_from_slice(&be32(n));
        for z in &self.sizes {
            stsz.extend_from_slice(&be32(*z));
        }
        let stsz = full_box(b"stsz", &stsz);

        let mut offsets = Vec::new();
        let mut at = self.data_at;
        for chunk in self.sizes.chunks(PER_CHUNK as usize) {
            offsets.push(at);
            at += chunk.iter().map(|z| u64::from(*z)).sum::<u64>();
        }
        // 32-bit offsets while the file is under 4 GB, 64-bit beyond.
        let stco = if at < u64::from(u32::MAX) {
            let mut b = be32(offsets.len() as u32).to_vec();
            for o in &offsets {
                b.extend_from_slice(&be32(*o as u32));
            }
            full_box(b"stco", &b)
        } else {
            let mut b = be32(offsets.len() as u32).to_vec();
            for o in &offsets {
                b.extend_from_slice(&o.to_be_bytes());
            }
            full_box(b"co64", &b)
        };

        let stbl = make_box(b"stbl", &[stsd, stts, stsc, stsz, stco].concat());
        let minf = make_box(b"minf", &[smhd, dinf, stbl].concat());
        let mdia = make_box(b"mdia", &[mdhd, hdlr, minf].concat());
        let trak = make_box(b"trak", &[tkhd, edts, mdia].concat());
        make_box(b"moov", &[full_box(b"mvhd", &mvhd), trak].concat())
    }
}

impl AacWriter {
    fn new(enc: Encoder, sink: Sink, channels: u16) -> Self {
        Self {
            enc: Box::new(enc),
            sink,
            channels: usize::from(channels),
            planar: vec![Vec::new(); usize::from(channels)],
            frames_in: 0,
        }
    }

    /// An M4A (AAC in an MP4 file).
    pub fn create(path: &Path, rate: u32, channels: u16, kbps: u16) -> Result<Self, String> {
        if channels == 0 || channels > 2 {
            return Err(format!(
                "M4A export writes mono or stereo; this mix has {channels} channels."
            ));
        }
        let cfg = EncoderConfig::cbr(rate, usize::from(channels), u32::from(kbps) * 1000);
        let enc = Encoder::new(cfg).map_err(|e| format!("AAC encoder: {e}"))?;
        let mut file = BufWriter::new(File::create(path).map_err(io)?);
        let ftyp = [b"M4A ".as_slice(), &be32(0), b"M4A ", b"mp42", b"isom"].concat();
        file.write_all(&make_box(b"ftyp", &ftyp)).map_err(io)?;
        let mdat_at = file.stream_position().map_err(io)?;
        file.write_all(&make_box(b"mdat", &[])).map_err(io)?;
        let data_at = mdat_at + 8;
        Ok(Self::new(
            enc,
            Sink::M4a(Box::new(M4a {
                file,
                mdat_at,
                data_at,
                sizes: Vec::new(),
                bytes: 0,
                rate,
                channels,
                kbps,
            })),
            channels,
        ))
    }

    /// A raw AAC (ADTS) file, `.aac`, instead of an M4A.
    pub fn create_adts(path: &Path, rate: u32, channels: u16, kbps: u16) -> Result<Self, String> {
        let mut cfg = EncoderConfig::cbr(rate, usize::from(channels), u32::from(kbps) * 1000);
        cfg.adts = true;
        let enc = Encoder::new(cfg).map_err(|e| format!("AAC encoder: {e}"))?;
        let file = BufWriter::new(File::create(path).map_err(io)?);
        Ok(Self::new(enc, Sink::Adts(file), channels))
    }

    fn put(&mut self, units: Vec<Vec<u8>>) -> Result<(), String> {
        for au in units {
            match &mut self.sink {
                Sink::M4a(m) => {
                    m.file.write_all(&au).map_err(io)?;
                    m.sizes.push(au.len() as u32);
                    m.bytes += au.len() as u64;
                    if m.bytes > u64::from(u32::MAX) - 1_000_000 {
                        return Err("This mix is too long for one M4A file.".into());
                    }
                }
                Sink::Adts(f) => f.write_all(&au).map_err(io)?,
            }
        }
        Ok(())
    }

    /// Interleaved 16-bit samples.
    pub fn write(&mut self, samples: &[i16]) -> Result<(), String> {
        let ch = self.channels;
        for frame in samples.chunks_exact(ch) {
            for (c, s) in frame.iter().enumerate() {
                self.planar[c].push(f32::from(*s) / 32768.0);
            }
            self.frames_in += 1;
        }
        // Whole blocks go to the encoder; the rest waits for the next call.
        let whole = self.planar[0].len() / FRAME_LEN * FRAME_LEN;
        if whole == 0 {
            return Ok(());
        }
        let blocks: Vec<Vec<f32>> = self
            .planar
            .iter_mut()
            .map(|p| p.drain(..whole).collect())
            .collect();
        let refs: Vec<&[f32]> = blocks.iter().map(Vec::as_slice).collect();
        let units = self.enc.encode(&refs);
        self.put(units)
    }

    pub fn finish(mut self) -> Result<(), String> {
        let rest: Vec<Vec<f32>> = std::mem::take(&mut self.planar);
        if rest.first().is_some_and(|p| !p.is_empty()) {
            let refs: Vec<&[f32]> = rest.iter().map(Vec::as_slice).collect();
            let units = self.enc.encode(&refs);
            self.put(units)?;
        }
        let units = self.enc.flush();
        self.put(units)?;
        match self.sink {
            Sink::M4a(mut m) => {
                let asc = self.enc.audio_specific_config();
                let moov = m.moov(&asc, self.frames_in);
                m.file.write_all(&moov).map_err(io)?;
                // Now the size of the audio box is known.
                let mdat_len = u32::try_from(8 + m.bytes)
                    .map_err(|_| "This mix is too long for one M4A file.")?;
                m.file.seek(SeekFrom::Start(m.mdat_at)).map_err(io)?;
                m.file.write_all(&mdat_len.to_be_bytes()).map_err(io)?;
                m.file.flush().map_err(io)
            }
            Sink::Adts(mut f) => f.flush().map_err(io),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn tone(frames: usize, ch: usize) -> Vec<i16> {
        (0..frames * ch)
            .map(|i| {
                let t = (i / ch) as f32 / 48_000.0;
                ((t * 440.0 * std::f32::consts::TAU).sin() * 12_000.0) as i16
            })
            .collect()
    }

    #[test]
    fn writes_an_m4a_that_reads_back_with_the_right_shape() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.m4a");
        let mut w = AacWriter::create(&p, 48_000, 2, 128).unwrap();
        // Odd chunk sizes, to exercise the leftover handling.
        let pcm = tone(48_000, 2);
        for c in pcm.chunks(2 * 1500 + 2) {
            w.write(c).unwrap();
        }
        w.finish().unwrap();
        let f = File::open(&p).unwrap();
        let n = f.metadata().unwrap().len();
        let r = mp4::Mp4Reader::read_header(std::io::BufReader::new(f), n).unwrap();
        let t = r.tracks().values().next().unwrap();
        assert_eq!(t.media_type().unwrap(), mp4::MediaType::AAC);
        assert_eq!(t.timescale(), 48_000);
        // One second plus the encoder's look-ahead: about 47 to 49 blocks.
        assert!(
            (46..=50).contains(&t.sample_count()),
            "{}",
            t.sample_count()
        );
    }

    #[test]
    fn writes_an_adts_file_with_a_valid_first_header() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("t.aac");
        let mut w = AacWriter::create_adts(&p, 44_100, 1, 96).unwrap();
        w.write(&tone(20_000, 1)).unwrap();
        w.finish().unwrap();
        let b = std::fs::read(&p).unwrap();
        assert_eq!(b[0], 0xFF);
        assert_eq!(b[1] & 0xF0, 0xF0);
        let frames = filmcraft_aac::split_adts(&b).unwrap();
        assert!(frames.len() > 10);
    }

    #[test]
    fn refuses_rates_and_layouts_it_cannot_write() {
        let dir = tempfile::tempdir().unwrap();
        assert!(AacWriter::create(&dir.path().join("a.m4a"), 12_345, 2, 128).is_err());
        assert!(AacWriter::create(&dir.path().join("b.m4a"), 48_000, 6, 128).is_err());
    }

    /// An independent decoder (ffmpeg) must accept both files and find about a second of sound.
    #[test]
    fn ffmpeg_decodes_both_files() {
        if Command::new("ffmpeg").arg("-version").output().is_err() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        for (name, adts) in [("o.m4a", false), ("o.aac", true)] {
            let p = dir.path().join(name);
            let mut w = if adts {
                AacWriter::create_adts(&p, 48_000, 2, 128)
            } else {
                AacWriter::create(&p, 48_000, 2, 128)
            }
            .unwrap();
            w.write(&tone(48_000, 2)).unwrap();
            w.finish().unwrap();
            let out = Command::new("ffmpeg")
                .args(["-v", "error", "-i"])
                .arg(&p)
                .args(["-f", "null", "-"])
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{name}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            assert!(
                out.stderr.is_empty(),
                "{name}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            // One second in, one second out: the start delay is trimmed by the edit list.
            if let Ok(probe) = Command::new("ffprobe")
                .args([
                    "-v",
                    "error",
                    "-show_entries",
                    "format=duration",
                    "-of",
                    "csv=p=0",
                ])
                .arg(&p)
                .output()
            {
                let secs: f64 = String::from_utf8_lossy(&probe.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0.0);
                assert!((0.98..1.05).contains(&secs), "{name}: {secs} s");
            }
        }
    }
}
