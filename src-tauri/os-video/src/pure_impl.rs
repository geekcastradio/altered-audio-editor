// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Video pictures decoded inside the program, with no system codecs.
//!
//! Opt-in and experimental (feature `pure-video`, and a switch in the Video
//! panel). MP4 and MOV files with H.264 pictures are read with the same
//! container reader the audio side uses and decoded by a pure Rust decoder.
//! Anything else returns an error, and the caller falls back to the system
//! decoder.
//!
//! Known gaps: no rotation (a phone clip that carries a turn shows sideways),
//! and only 8-bit 4:2:0 pictures.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::fs::File;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use craft_video::h264::decode::{Decoder as H264, Picture};
use symphonia::core::codecs::video::well_known::extra_data::VIDEO_EXTRA_DATA_ID_AVC_DECODER_CONFIG;
use symphonia::core::codecs::video::well_known::CODEC_ID_H264;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::{TimeBase, Timestamp};

use crate::{Backend, RawFrame, VideoInfo};

pub struct PureDecoder {
    format: Box<dyn FormatReader>,
    path: std::path::PathBuf,
    track_id: u32,
    time_base: TimeBase,
    length_size: usize,
    config: Vec<u8>,
    dec: H264,
    /// Pictures decoded but not yet handed out, in display order, with their
    /// timestamps in track ticks.
    ready: VecDeque<(Picture, i64)>,
    /// Packets read ahead while looking for a key picture, to decode first.
    queue: VecDeque<(i64, Vec<u8>)>,
    /// Decode times of the packets fed so far that have not yet been given to
    /// a picture. The file reader hands out decode times, not display times,
    /// and pictures come out in display order, so the smallest unused time
    /// belongs to the next picture out.
    times: BinaryHeap<Reverse<i64>>,
    ended: bool,
}

fn err<E: std::fmt::Display>(what: &str) -> impl Fn(E) -> String + '_ {
    move |e| format!("Video ({what}): {e}")
}

fn open_format(path: &Path) -> Result<Box<dyn FormatReader>, String> {
    let file = File::open(path).map_err(err("open"))?;
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    symphonia::default::get_probe()
        .probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .map_err(err("read"))
}

impl PureDecoder {
    fn fresh_decoder(config: &[u8]) -> Result<(H264, usize), String> {
        let dec = H264::from_avcc(config).map_err(|e| format!("Video (setup): {e}"))?;
        // The length prefix size is the low two bits of byte 4, plus one.
        let length_size = config.get(4).map_or(4, |b| usize::from(b & 3) + 1);
        Ok((dec, length_size))
    }

    fn secs(&self, ts: i64) -> f64 {
        self.time_base
            .calc_time(Timestamp::new(ts))
            .map_or(0.0, |t| t.as_secs_f64())
    }

    fn ticks(&self, seconds: f64) -> i64 {
        let t = seconds.max(0.0);
        let time = symphonia::core::units::Time::try_from_secs_f64(t)
            .unwrap_or(symphonia::core::units::Time::ZERO);
        self.time_base.calc_timestamp(time).map_or(0, |ts| ts.get())
    }

    /// Reads and decodes one packet. Returns false at the end of the file.
    fn feed_next_packet(&mut self) -> Result<bool, String> {
        if let Some((ts, data)) = self.queue.pop_front() {
            self.decode_packet(&data, ts)?;
            return Ok(true);
        }
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(p)) => p,
                Ok(None) => {
                    self.finish();
                    return Ok(false);
                }
                Err(symphonia::core::errors::Error::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    self.finish();
                    return Ok(false);
                }
                Err(e) => return Err(format!("Video (read): {e}")),
            };
            if packet.track_id != self.track_id {
                continue;
            }
            let ts = packet.dts.get();
            self.decode_packet(&packet.data, ts)?;
            return Ok(true);
        }
    }

    fn decode_packet(&mut self, data: &[u8], ts: i64) -> Result<(), String> {
        self.times.push(Reverse(ts));
        let dec = &mut self.dec;
        let out = catch_unwind(AssertUnwindSafe(|| dec.decode(data, ts)))
            .map_err(|_| "Video: the built-in decoder stopped on this file".to_string())?;
        // A damaged picture is skipped; the next key picture recovers.
        if let Ok(pictures) = out {
            self.take_times(pictures);
        }
        Ok(())
    }

    fn take_times(&mut self, pictures: Vec<Picture>) {
        for p in pictures {
            let ts = self.times.pop().map_or(p.pts, |Reverse(t)| t);
            self.ready.push_back((p, ts));
        }
    }

    fn finish(&mut self) {
        if !self.ended {
            self.ended = true;
            let dec = &mut self.dec;
            if let Ok(rest) = catch_unwind(AssertUnwindSafe(|| dec.flush())) {
                self.take_times(rest);
            }
        }
    }

    /// Frames per second from the gaps between the first packets' decode
    /// times, for files that do not say. Leaves the reader at the start.
    fn estimate_fps(&mut self) -> f64 {
        let mut times = Vec::new();
        while times.len() < 24 {
            match self.format.next_packet() {
                Ok(Some(p)) if p.track_id == self.track_id => times.push(p.dts.get()),
                Ok(Some(_)) => {}
                _ => break,
            }
        }
        times.sort_unstable();
        let gap = times
            .windows(2)
            .map(|w| w[1] - w[0])
            .filter(|g| *g > 0)
            .min();
        let _ = self.format.seek(
            SeekMode::Accurate,
            SeekTo::Timestamp {
                ts: Timestamp::new(0),
                track_id: self.track_id,
            },
        );
        match gap.and_then(|g| self.time_base.calc_time(Timestamp::new(g))) {
            Some(t) if t.as_secs_f64() > 0.0 => 1.0 / t.as_secs_f64(),
            _ => 0.0,
        }
    }

    /// Whether a packet holds a key picture (an IDR slice, NAL type 5).
    fn is_key(data: &[u8], length_size: usize) -> bool {
        let mut i = 0;
        while i + length_size < data.len() {
            let len = data[i..i + length_size]
                .iter()
                .fold(0usize, |a, b| (a << 8) | usize::from(*b));
            let head = i + length_size;
            if data.get(head).is_some_and(|h| h & 0x1f == 5) {
                return true;
            }
            i = head.saturating_add(len);
        }
        false
    }

    /// Reads from a little before `seconds` and queues every packet from the
    /// last key picture at or before it, plus a little after, ready to decode.
    fn queue_from_key_before(&mut self, seconds: f64) -> Result<(), String> {
        let target = self.ticks(seconds);
        let slack = self.ticks(0.5);
        let mut window = 2.0f64;
        loop {
            let start = (seconds - window).max(0.0);
            self.format
                .seek(
                    SeekMode::Accurate,
                    SeekTo::Timestamp {
                        ts: Timestamp::new(self.ticks(start)),
                        track_id: self.track_id,
                    },
                )
                .map_err(err("seek"))?;
            let mut run: VecDeque<(i64, Vec<u8>)> = VecDeque::new();
            let mut has_key = false;
            while let Ok(Some(p)) = self.format.next_packet() {
                if p.track_id != self.track_id {
                    continue;
                }
                let ts = p.dts.get();
                if ts > target + slack {
                    // Keep this one too: reading goes on after it.
                    if has_key {
                        run.push_back((ts, p.data.to_vec()));
                    }
                    break;
                }
                if ts <= target && Self::is_key(&p.data, self.length_size) {
                    run.clear();
                    has_key = true;
                }
                if has_key {
                    run.push_back((ts, p.data.to_vec()));
                }
            }
            if has_key || start <= 0.0 {
                self.queue = run;
                return Ok(());
            }
            window *= 2.0;
        }
    }
}

/// Converts a decoded picture to blue, green, red, spare.
fn to_bgra(f: &Picture, time: f64) -> RawFrame {
    let (w, h) = (f.width as usize, f.height as usize);
    // Matrix 1 is BT.709, 5 and 6 are BT.601; when the file does not say,
    // pictures from 720 lines up are taken as 709.
    let hd = match f.color.matrix {
        1 => true,
        5 | 6 => false,
        _ => h >= 720,
    };
    let (kr, kb) = if hd {
        (0.2126f32, 0.0722f32)
    } else {
        (0.299, 0.114)
    };
    let kg = 1.0 - kr - kb;
    let (ys, cs) = if f.color.full_range {
        (1.0, 255.0 / 254.0)
    } else {
        (255.0 / 219.0, 255.0 / 224.0)
    };
    let off = if f.color.full_range { 0.0 } else { 16.0 };
    let mut bgra = vec![255u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let yy = f.y[y * f.y_stride + x] as f32;
            let ci = (y / 2) * f.uv_stride + x / 2;
            let u = f.u[ci] as f32;
            let v = f.v[ci] as f32;
            let yl = (yy - off) * ys;
            let cb = (u - 128.0) * cs;
            let cr = (v - 128.0) * cs;
            let r = yl + 2.0 * (1.0 - kr) * cr;
            let b = yl + 2.0 * (1.0 - kb) * cb;
            let g = (yl - kr * r - kb * b) / kg;
            let o = (y * w + x) * 4;
            bgra[o] = b.clamp(0.0, 255.0) as u8;
            bgra[o + 1] = g.clamp(0.0, 255.0) as u8;
            bgra[o + 2] = r.clamp(0.0, 255.0) as u8;
        }
    }
    RawFrame {
        width: f.width,
        height: f.height,
        stride: w * 4,
        bgra,
        time,
    }
}

impl Backend for PureDecoder {
    fn open(path: &Path, _hardware: bool) -> Result<(Self, VideoInfo), String> {
        let format = open_format(path)?;
        let tracks = format.tracks();
        let track = tracks
            .iter()
            .find(|t| t.codec_params.as_ref().and_then(|p| p.video()).is_some())
            .ok_or("Video: no picture track in this file")?;
        let vp = track
            .codec_params
            .as_ref()
            .and_then(|p| p.video())
            .ok_or("Video: no picture track in this file")?;
        if vp.codec != CODEC_ID_H264 {
            return Err("Video: the built-in decoder reads H.264 only so far".into());
        }
        let config = vp
            .extra_data
            .iter()
            .find(|e| e.id == VIDEO_EXTRA_DATA_ID_AVC_DECODER_CONFIG)
            .map(|e| e.data.to_vec())
            .ok_or("Video: this H.264 file has no decoder settings")?;
        let time_base = track.time_base.ok_or("Video: the file has no timing")?;
        let has_audio = tracks
            .iter()
            .any(|t| t.codec_params.as_ref().and_then(|p| p.audio()).is_some());
        let (width, height) = (
            u32::from(vp.width.unwrap_or(0)),
            u32::from(vp.height.unwrap_or(0)),
        );
        let duration = track
            .duration
            .and_then(|d| time_base.calc_time(Timestamp::new(d.get() as i64)))
            .map_or(0.0, |t| t.as_secs_f64());
        let fps = match (track.num_frames, duration) {
            (Some(n), d) if d > 0.0 => n as f64 / d,
            _ => 0.0,
        };
        let track_id = track.id;
        let (dec, length_size) = Self::fresh_decoder(&config)?;
        let _ = TrackType::Video;
        let me = Self {
            format,
            path: path.to_path_buf(),
            track_id,
            time_base,
            length_size,
            config,
            dec,
            ready: VecDeque::new(),
            queue: VecDeque::new(),
            times: BinaryHeap::new(),
            ended: false,
        };
        let mut me = me;
        let fps = if fps > 0.0 { fps } else { me.estimate_fps() };
        let info = VideoInfo {
            width,
            height,
            fps,
            duration,
            has_audio,
            codec: "H.264".into(),
        };
        Ok((me, info))
    }

    fn frame_at(&mut self, seconds: f64) -> Result<RawFrame, String> {
        if self.ended {
            // The file reader cannot seek again once it has read to the end.
            self.format = open_format(&self.path)?;
        }
        self.queue_from_key_before(seconds)?;
        let (dec, ls) = Self::fresh_decoder(&self.config)?;
        self.dec = dec;
        self.length_size = ls;
        self.ready.clear();
        self.times.clear();
        self.ended = false;
        let target = self.ticks(seconds);
        let mut last: Option<(Picture, i64)> = None;
        loop {
            while let Some((f, ts)) = self.ready.pop_front() {
                if ts <= target {
                    last = Some((f, ts));
                } else {
                    // First picture after the target: the one before it shows now.
                    self.ready.push_front((f, ts));
                    return match last {
                        Some((f, ts)) => Ok(to_bgra(&f, self.secs(ts))),
                        None => Err("Video: no picture at this time".into()),
                    };
                }
            }
            if !self.feed_next_packet()? {
                // End of file: whatever is left is the last picture.
                while let Some((f, ts)) = self.ready.pop_front() {
                    if ts <= target {
                        last = Some((f, ts));
                    }
                }
                return match last {
                    Some((f, ts)) => Ok(to_bgra(&f, self.secs(ts))),
                    None => Err("Video: no picture at this time".into()),
                };
            }
        }
    }

    fn next_frame(&mut self) -> Result<Option<RawFrame>, String> {
        loop {
            if let Some((f, ts)) = self.ready.pop_front() {
                return Ok(Some(to_bgra(&f, self.secs(ts))));
            }
            if self.ended || !self.feed_next_packet()? {
                return Ok(self
                    .ready
                    .pop_front()
                    .map(|(f, ts)| to_bgra(&f, self.secs(ts))));
            }
        }
    }
}
