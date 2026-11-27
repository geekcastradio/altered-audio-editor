// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Video pictures through the decoders built into the operating system.
//!
//! This is its own small crate because talking to the system decoders needs
//! `unsafe` calls, and the app crate forbids `unsafe` everywhere. Keeping it
//! here means that rule still holds for the app itself.
//!
//! One frame at a time: a [`VideoReader`] decodes forward from where it was
//! asked to be and hands back one picture, already shrunk to the size the
//! preview needs. Nothing is decoded ahead and no picture is kept, so the
//! memory in use stays at a frame or two however long the video is.
//!
//! Windows: Media Foundation. Other systems: [`VideoReader::open`] returns
//! an error that says so.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

/// True where this build can show video.
pub const AVAILABLE: bool = cfg!(windows)
    || cfg!(all(target_os = "macos", feature = "apple"))
    || cfg!(feature = "pure-video");

/// Why video is not available here, for the error message.
pub const UNAVAILABLE: &str =
    "Video pictures are not available on this system yet. The sound of a video still opens.";

/// What a video file holds.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    /// Frames per second (0 when the file does not say).
    pub fps: f64,
    /// Length in seconds (0 when unknown).
    pub duration: f64,
    /// True when the file also has a sound track.
    pub has_audio: bool,
    /// The picture codec by its common name ("H.264", "HEVC"), or empty when unknown.
    pub codec: String,
}

/// Whether to ask the system for a hardware decoder first. Off by default:
/// software decoding is what has been tried the most. Applies to videos
/// opened after it is set.
static PREFER_HARDWARE: AtomicBool = AtomicBool::new(false);

/// Sets whether videos opened from now on ask for hardware decoding first.
/// If that fails the video opens with software decoding instead.
pub fn set_prefer_hardware(on: bool) {
    PREFER_HARDWARE.store(on, Ordering::Relaxed);
}

/// Whether hardware decoding is asked for first.
pub fn prefer_hardware() -> bool {
    PREFER_HARDWARE.load(Ordering::Relaxed)
}

/// How long pictures took to get, for the Video panel's details line.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DecodeStats {
    /// Pictures delivered.
    pub frames: u64,
    /// Pictures that took longer than one frame of the video to get.
    pub slow: u64,
    pub total_ms: f64,
    pub max_ms: f64,
}

impl DecodeStats {
    /// Counts one picture that took `ms`, against `budget_ms` (one frame).
    pub fn record(&mut self, ms: f64, budget_ms: f64) {
        self.frames += 1;
        self.total_ms += ms;
        if ms > self.max_ms {
            self.max_ms = ms;
        }
        if ms > budget_ms {
            self.slow += 1;
        }
    }

    /// Mean time per picture, 0 before the first.
    pub fn average_ms(&self) -> f64 {
        if self.frames == 0 {
            0.0
        } else {
            self.total_ms / self.frames as f64
        }
    }
}

/// The common name for a picture codec from the first four bytes of its
/// format code (a four-letter tag such as "H264"), or empty when unknown.
pub fn codec_name(tag: u32) -> String {
    let bytes = tag.to_le_bytes();
    let text: String = bytes.iter().map(|b| char::from(*b)).collect();
    let name = match text.to_ascii_uppercase().as_str() {
        "H264" | "AVC1" | "X264" => "H.264",
        "H265" | "HEVC" | "HVC1" | "HEV1" => "HEVC",
        "AV01" => "AV1",
        "VP90" => "VP9",
        "VP80" => "VP8",
        "MP4V" | "MP4S" => "MPEG-4",
        "WMV3" | "WVC1" => "Windows Media Video",
        "MJPG" => "Motion JPEG",
        _ => "",
    };
    if !name.is_empty() {
        return name.to_string();
    }
    if bytes.iter().all(|b| b.is_ascii_graphic()) {
        text
    } else {
        String::new()
    }
}

/// An error message for a failed system call. Two failures have a plain
/// answer the person can act on; everything else keeps the system's own text.
pub fn explain(what: &str, code: u32, text: &str) -> String {
    match code {
        // No decoder installed for this picture format.
        0xC00D_5212 => "Video: Windows has no decoder for this video's codec. For HEVC or AV1, \
            install the matching video extension from the Microsoft Store, then open it again."
            .to_string(),
        // The file type is not one the system can read.
        0xC00D_36C4 => "Video: Windows can't read this kind of file. MP4 and MOV files with \
            H.264 pictures are the safest."
            .to_string(),
        _ => format!("Video ({what}): {text}"),
    }
}

/// A picture as the system decoder delivered it: blue, green, red and a
/// spare byte per pixel, top row first.
pub struct RawFrame {
    pub width: u32,
    pub height: u32,
    /// Bytes from the start of one row to the next.
    pub stride: usize,
    pub bgra: Vec<u8>,
    /// Where in the video this picture belongs, in seconds.
    pub time: f64,
}

/// A picture ready to draw: red, green, blue, alpha per pixel.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub time: f64,
    pub rgba: Vec<u8>,
}

/// Turns a picture clockwise by `quarters` quarter turns (phone clips carry a
/// rotation the decoder does not apply). The result has no row padding.
pub fn rotate(raw: RawFrame, quarters: u32) -> RawFrame {
    let q = quarters % 4;
    if q == 0 {
        return raw;
    }
    let (w, h) = (raw.width as usize, raw.height as usize);
    let (nw, nh) = if q == 2 { (w, h) } else { (h, w) };
    let mut out = vec![0u8; nw * nh * 4];
    for y in 0..h {
        for x in 0..w {
            let src = y * raw.stride + x * 4;
            let (dx, dy) = match q {
                1 => (h - 1 - y, x),
                2 => (w - 1 - x, h - 1 - y),
                _ => (y, w - 1 - x),
            };
            let dst = (dy * nw + dx) * 4;
            if let (Some(s), Some(d)) = (raw.bgra.get(src..src + 4), out.get_mut(dst..dst + 4)) {
                d.copy_from_slice(s);
            }
        }
    }
    RawFrame {
        width: nw as u32,
        height: nh as u32,
        stride: nw * 4,
        bgra: out,
        time: raw.time,
    }
}

/// Shrinks a decoded picture to at most `max_width` pixels across (never
/// enlarging it), averaging the pixels that fall into each new one, and puts
/// the colors in drawing order.
pub fn scale(raw: &RawFrame, max_width: u32) -> Frame {
    let sw = raw.width.max(1) as usize;
    let sh = raw.height.max(1) as usize;
    let dw = (max_width.max(1) as usize).min(sw);
    let dh = ((sh * dw + sw / 2) / sw).max(1);
    let mut rgba = Vec::with_capacity(dw * dh * 4);
    for y in 0..dh {
        let y0 = y * sh / dh;
        let y1 = (((y + 1) * sh).div_ceil(dh)).clamp(y0 + 1, sh);
        for x in 0..dw {
            let x0 = x * sw / dw;
            let x1 = (((x + 1) * sw).div_ceil(dw)).clamp(x0 + 1, sw);
            let (mut b, mut g, mut r) = (0u32, 0u32, 0u32);
            for yy in y0..y1 {
                let row = yy * raw.stride;
                for xx in x0..x1 {
                    let p = row + xx * 4;
                    if let Some(px) = raw.bgra.get(p..p + 3) {
                        b += u32::from(px[0]);
                        g += u32::from(px[1]);
                        r += u32::from(px[2]);
                    }
                }
            }
            let n = ((y1 - y0) * (x1 - x0)) as u32;
            rgba.extend_from_slice(&[(r / n) as u8, (g / n) as u8, (b / n) as u8, 255]);
        }
    }
    Frame {
        width: dw as u32,
        height: dh as u32,
        time: raw.time,
        rgba,
    }
}

/// How a platform's decoder is driven. Lives on the thread that made it,
/// because system decoder objects are tied to the thread that opened them.
trait Backend: Sized {
    /// `hardware` asks the system for a hardware decoder first.
    fn open(path: &Path, hardware: bool) -> Result<(Self, VideoInfo), String>;
    /// The picture showing at `seconds`: seeks to the key frame before it and
    /// decodes forward to it.
    fn frame_at(&mut self, seconds: f64) -> Result<RawFrame, String>;
    /// The next picture after the one last returned; None at the end.
    fn next_frame(&mut self) -> Result<Option<RawFrame>, String>;
    /// Clockwise quarter turns to apply to every picture so it shows upright.
    /// The size in `VideoInfo` is already the turned size.
    fn quarter_turns(&self) -> u32 {
        0
    }
}

#[cfg(windows)]
mod windows_impl;
#[cfg(windows)]
use windows_impl::Decoder as SysDecoder;

#[cfg(all(target_os = "macos", feature = "apple"))]
mod mac_impl;
#[cfg(all(target_os = "macos", feature = "apple"))]
use mac_impl::Decoder as SysDecoder;

#[cfg(feature = "pure-video")]
mod pure_impl;

/// Whether videos opened from now on try the built-in decoder first. Off by
/// default. With it on, any file the built-in decoder cannot read opens with
/// the system decoder instead.
static PREFER_BUILT_IN: AtomicBool = AtomicBool::new(false);

/// Sets whether videos opened from now on try the built-in decoder first.
pub fn set_prefer_built_in(on: bool) {
    PREFER_BUILT_IN.store(on, Ordering::Relaxed);
}

/// True when this build has the built-in decoder.
pub const BUILT_IN: bool = cfg!(feature = "pure-video");

/// The decoder in use for one open video: the built-in one or the system's.
enum Decoder {
    #[cfg(any(windows, all(target_os = "macos", feature = "apple")))]
    System(SysDecoder),
    #[cfg(feature = "pure-video")]
    BuiltIn(Box<pure_impl::PureDecoder>),
    /// Never made: keeps the enum valid in a build with no decoder at all.
    #[allow(dead_code)]
    None(std::convert::Infallible),
}

impl Backend for Decoder {
    fn open(path: &Path, hardware: bool) -> Result<(Self, VideoInfo), String> {
        #[cfg(feature = "pure-video")]
        {
            if PREFER_BUILT_IN.load(Ordering::Relaxed) || !SYSTEM {
                match pure_impl::PureDecoder::open(path, hardware) {
                    Ok((d, info)) => return Ok((Decoder::BuiltIn(Box::new(d)), info)),
                    Err(e) if !SYSTEM => return Err(e),
                    Err(_) => {}
                }
            }
        }
        #[cfg(any(windows, all(target_os = "macos", feature = "apple")))]
        {
            SysDecoder::open(path, hardware).map(|(d, info)| (Decoder::System(d), info))
        }
        #[cfg(not(any(windows, all(target_os = "macos", feature = "apple"))))]
        {
            let _ = (path, hardware);
            Err(UNAVAILABLE.into())
        }
    }
    fn frame_at(&mut self, seconds: f64) -> Result<RawFrame, String> {
        let _ = seconds;
        match self {
            #[cfg(any(windows, all(target_os = "macos", feature = "apple")))]
            Decoder::System(d) => d.frame_at(seconds),
            #[cfg(feature = "pure-video")]
            Decoder::BuiltIn(d) => d.frame_at(seconds),
            Decoder::None(n) => match *n {},
        }
    }
    fn next_frame(&mut self) -> Result<Option<RawFrame>, String> {
        match self {
            #[cfg(any(windows, all(target_os = "macos", feature = "apple")))]
            Decoder::System(d) => d.next_frame(),
            #[cfg(feature = "pure-video")]
            Decoder::BuiltIn(d) => d.next_frame(),
            Decoder::None(n) => match *n {},
        }
    }
    fn quarter_turns(&self) -> u32 {
        match self {
            #[cfg(any(windows, all(target_os = "macos", feature = "apple")))]
            Decoder::System(d) => d.quarter_turns(),
            #[cfg(feature = "pure-video")]
            Decoder::BuiltIn(_) => 0,
            Decoder::None(n) => match *n {},
        }
    }
}

/// True where a system decoder exists in this build.
#[cfg(feature = "pure-video")]
const SYSTEM: bool = cfg!(windows) || cfg!(all(target_os = "macos", feature = "apple"));

enum Cmd {
    At {
        seconds: f64,
        max_width: u32,
        reply: mpsc::Sender<Result<Frame, String>>,
    },
    Next {
        max_width: u32,
        reply: mpsc::Sender<Result<Option<Frame>, String>>,
    },
}

/// An open video. Cheap to keep: one thread holding the system decoder.
pub struct VideoReader {
    tx: Option<mpsc::Sender<Cmd>>,
    thread: Option<JoinHandle<()>>,
    info: VideoInfo,
    path: PathBuf,
    stats: Arc<Mutex<DecodeStats>>,
    hardware: Arc<AtomicBool>,
}

/// Opens with a hardware decoder when asked and one works, else with software.
/// The flag says which one it ended up with.
fn open_best(path: &Path) -> Result<(Decoder, VideoInfo, bool), String> {
    if prefer_hardware() {
        if let Ok((d, info)) = Decoder::open(path, true) {
            return Ok((d, info, true));
        }
    }
    Decoder::open(path, false).map(|(d, info)| (d, info, false))
}

fn picture_at(dec: &mut Decoder, seconds: f64, max_width: u32) -> Result<Frame, String> {
    let q = dec.quarter_turns();
    dec.frame_at(seconds)
        .map(|f| scale(&rotate(f, q), max_width))
}

fn picture_next(dec: &mut Decoder, max_width: u32) -> Result<Option<Frame>, String> {
    let q = dec.quarter_turns();
    dec.next_frame()
        .map(|f| f.map(|f| scale(&rotate(f, q), max_width)))
}

impl VideoReader {
    /// Opens `path` and reports what it holds.
    pub fn open(path: &Path) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel::<Cmd>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<VideoInfo, String>>();
        let owned = path.to_path_buf();
        let stats = Arc::new(Mutex::new(DecodeStats::default()));
        let hardware = Arc::new(AtomicBool::new(false));
        let (stats_in, hardware_in) = (Arc::clone(&stats), Arc::clone(&hardware));
        let thread = thread::Builder::new()
            .name("video".into())
            .spawn(move || {
                let (mut dec, info, mut hw) = match open_best(&owned) {
                    Ok(opened) => opened,
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                hardware_in.store(hw, Ordering::Relaxed);
                // One frame of the video, the time a picture may take.
                let budget = if info.fps > 0.0 {
                    1000.0 / info.fps
                } else {
                    40.0
                };
                let _ = ready_tx.send(Ok(info));
                while let Ok(cmd) = rx.recv() {
                    let started = Instant::now();
                    let counted = match cmd {
                        Cmd::At {
                            seconds,
                            max_width,
                            reply,
                        } => {
                            let mut r = picture_at(&mut dec, seconds, max_width);
                            if r.is_err() && hw {
                                // The hardware decoder failed: go on with software.
                                if let Ok((d, _)) = Decoder::open(&owned, false) {
                                    dec = d;
                                    hw = false;
                                    hardware_in.store(false, Ordering::Relaxed);
                                    r = picture_at(&mut dec, seconds, max_width);
                                }
                            }
                            let counted = r.is_ok();
                            let _ = reply.send(r);
                            counted
                        }
                        Cmd::Next { max_width, reply } => {
                            let mut r = picture_next(&mut dec, max_width);
                            if r.is_err() && hw {
                                if let Ok((d, _)) = Decoder::open(&owned, false) {
                                    dec = d;
                                    hw = false;
                                    hardware_in.store(false, Ordering::Relaxed);
                                    r = picture_next(&mut dec, max_width);
                                }
                            }
                            let counted = matches!(r, Ok(Some(_)));
                            let _ = reply.send(r);
                            counted
                        }
                    };
                    if counted {
                        let ms = started.elapsed().as_secs_f64() * 1000.0;
                        if let Ok(mut s) = stats_in.lock() {
                            s.record(ms, budget);
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        let info = ready_rx
            .recv()
            .map_err(|_| "The video decoder stopped unexpectedly.".to_string())??;
        Ok(Self {
            tx: Some(tx),
            thread: Some(thread),
            info,
            path: path.to_path_buf(),
            stats,
            hardware,
        })
    }

    /// How long pictures have taken so far.
    pub fn stats(&self) -> DecodeStats {
        self.stats.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// True when this video is being decoded with a hardware decoder the
    /// system accepted (it can still be the software one if that failed).
    pub fn hardware(&self) -> bool {
        self.hardware.load(Ordering::Relaxed)
    }

    pub fn info(&self) -> &VideoInfo {
        &self.info
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The picture showing at `seconds`, at most `max_width` pixels across.
    pub fn frame_at(&self, seconds: f64, max_width: u32) -> Result<Frame, String> {
        let (reply, got) = mpsc::channel();
        self.send(Cmd::At {
            seconds,
            max_width,
            reply,
        })?;
        got.recv()
            .map_err(|_| "The video decoder stopped unexpectedly.".to_string())?
    }

    /// The picture after the last one given, or None at the end.
    pub fn next_frame(&self, max_width: u32) -> Result<Option<Frame>, String> {
        let (reply, got) = mpsc::channel();
        self.send(Cmd::Next { max_width, reply })?;
        got.recv()
            .map_err(|_| "The video decoder stopped unexpectedly.".to_string())?
    }

    fn send(&self, cmd: Cmd) -> Result<(), String> {
        self.tx
            .as_ref()
            .ok_or_else(|| "The video is closed.".to_string())?
            .send(cmd)
            .map_err(|_| "The video decoder stopped unexpectedly.".to_string())
    }
}

impl Drop for VideoReader {
    fn drop(&mut self) {
        // Closing the channel ends the thread's loop; it closes the decoder itself.
        self.tx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// What a video file holds, without keeping it open.
pub fn probe(path: &Path) -> Result<VideoInfo, String> {
    VideoReader::open(path).map(|r| r.info().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, bgr: [u8; 3]) -> RawFrame {
        let mut bgra = Vec::new();
        for _ in 0..w * h {
            bgra.extend_from_slice(&[bgr[0], bgr[1], bgr[2], 0]);
        }
        RawFrame {
            width: w,
            height: h,
            stride: w as usize * 4,
            bgra,
            time: 1.5,
        }
    }

    #[test]
    fn colors_come_out_in_drawing_order_with_full_alpha() {
        // Blue 10, green 20, red 30 in; red 30, green 20, blue 10, opaque out.
        let f = scale(&solid(4, 2, [10, 20, 30]), 4);
        assert_eq!((f.width, f.height), (4, 2));
        assert_eq!(&f.rgba[..4], &[30, 20, 10, 255]);
        assert_eq!(f.time, 1.5);
    }

    #[test]
    fn a_picture_is_shrunk_by_averaging_and_keeps_its_shape() {
        // Left half black, right half white, 8x4 down to 4x2: each new pixel is
        // pure black or pure white, none in between.
        let mut raw = solid(8, 4, [0, 0, 0]);
        for y in 0..4 {
            for x in 4..8 {
                let p = y * raw.stride + x * 4;
                raw.bgra[p..p + 3].copy_from_slice(&[255, 255, 255]);
            }
        }
        let f = scale(&raw, 4);
        assert_eq!((f.width, f.height), (4, 2));
        assert_eq!(f.rgba[0], 0);
        assert_eq!(f.rgba[4 * 3], 255);
        // A 2x2 block of two different values averages.
        let mut mix = solid(2, 2, [0, 0, 0]);
        mix.bgra[0..3].copy_from_slice(&[200, 200, 200]);
        let one = scale(&mix, 1);
        assert_eq!((one.width, one.height), (1, 1));
        assert_eq!(one.rgba[0], 50);
    }

    #[test]
    fn a_small_picture_is_never_enlarged() {
        let f = scale(&solid(16, 9, [1, 2, 3]), 640);
        assert_eq!((f.width, f.height), (16, 9));
    }

    #[test]
    fn rows_are_read_by_their_stride() {
        // Two 1-pixel rows with 4 spare bytes after each.
        let raw = RawFrame {
            width: 1,
            height: 2,
            stride: 8,
            bgra: vec![0, 0, 255, 0, 9, 9, 9, 9, 255, 0, 0, 0, 9, 9, 9, 9],
            time: 0.0,
        };
        let f = scale(&raw, 1);
        // Top pixel is red, bottom is blue.
        assert_eq!(&f.rgba[..3], &[255, 0, 0]);
        assert_eq!(&f.rgba[4..7], &[0, 0, 255]);
    }

    #[cfg(not(windows))]
    #[test]
    fn a_clockwise_turn_moves_the_top_left_to_the_top_right() {
        // 2 wide, 1 tall: [A][B] turned once clockwise is 1 wide, 2 tall: A over B.
        let raw = RawFrame {
            width: 2,
            height: 1,
            stride: 8,
            bgra: vec![1, 1, 1, 255, 2, 2, 2, 255],
            time: 0.5,
        };
        let r = rotate(raw, 1);
        assert_eq!((r.width, r.height, r.stride), (1, 2, 4));
        assert_eq!(r.bgra, vec![1, 1, 1, 255, 2, 2, 2, 255]);
        assert_eq!(r.time, 0.5);
    }

    #[test]
    fn four_turns_come_back_and_two_turns_flip_both_ways() {
        let raw = |q| {
            rotate(
                RawFrame {
                    width: 2,
                    height: 2,
                    stride: 8,
                    bgra: (0..4u8).flat_map(|i| [i, i, i, 255]).collect(),
                    time: 0.0,
                },
                q,
            )
            .bgra
        };
        assert_eq!(raw(4), raw(0));
        let flipped: Vec<u8> = (0..4u8).rev().flat_map(|i| [i, i, i, 255]).collect();
        assert_eq!(raw(2), flipped);
        // Three clockwise turns put the top-right pixel (1) at the top-left.
        assert_eq!(&raw(3)[..4], &[1, 1, 1, 255]);
    }

    #[test]
    fn codecs_get_their_common_names() {
        let tag = |s: &[u8; 4]| u32::from_le_bytes(*s);
        assert_eq!(codec_name(tag(b"H264")), "H.264");
        assert_eq!(codec_name(tag(b"avc1")), "H.264");
        assert_eq!(codec_name(tag(b"hvc1")), "HEVC");
        assert_eq!(codec_name(tag(b"AV01")), "AV1");
        assert_eq!(codec_name(tag(b"VP90")), "VP9");
        // An unknown but readable tag is shown as it is; an unreadable one is empty.
        assert_eq!(codec_name(tag(b"ABCD")), "ABCD");
        assert_eq!(codec_name(0x0000_0001), "");
    }

    #[test]
    fn two_failures_get_a_plain_answer_and_the_rest_keep_the_systems_text() {
        assert!(explain("open", 0xC00D_5212, "x").contains("no decoder"));
        assert!(explain("open", 0xC00D_5212, "x").contains("Microsoft Store"));
        assert!(explain("open", 0xC00D_36C4, "x").contains("can't read"));
        assert_eq!(explain("read", 5, "boom"), "Video (read): boom");
    }

    #[test]
    fn stats_count_slow_pictures_against_one_frame() {
        let mut s = DecodeStats::default();
        assert_eq!(s.average_ms(), 0.0);
        s.record(10.0, 33.0);
        s.record(50.0, 33.0);
        s.record(30.0, 33.0);
        assert_eq!(s.frames, 3);
        assert_eq!(s.slow, 1);
        assert_eq!(s.max_ms, 50.0);
        assert!((s.average_ms() - 30.0).abs() < 1e-9);
    }

    #[test]
    fn opening_says_why_when_unavailable() {
        let e = VideoReader::open(Path::new("nothing.mp4")).err().unwrap();
        if AVAILABLE {
            // A real decoder reports the file it could not open.
            assert!(!e.is_empty() && e != UNAVAILABLE, "{e}");
        } else {
            assert_eq!(e, UNAVAILABLE);
        }
    }
}
