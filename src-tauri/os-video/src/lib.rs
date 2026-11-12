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
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

/// True where this build can show video.
pub const AVAILABLE: bool = cfg!(windows) || cfg!(all(target_os = "macos", feature = "apple"));

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
    fn open(path: &Path) -> Result<(Self, VideoInfo), String>;
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
use windows_impl::Decoder;

#[cfg(all(target_os = "macos", feature = "apple"))]
mod mac_impl;
#[cfg(all(target_os = "macos", feature = "apple"))]
use mac_impl::Decoder;

#[cfg(not(any(windows, all(target_os = "macos", feature = "apple"))))]
struct Decoder;

#[cfg(not(any(windows, all(target_os = "macos", feature = "apple"))))]
impl Backend for Decoder {
    fn open(_path: &Path) -> Result<(Self, VideoInfo), String> {
        Err(UNAVAILABLE.into())
    }
    fn frame_at(&mut self, _seconds: f64) -> Result<RawFrame, String> {
        Err(UNAVAILABLE.into())
    }
    fn next_frame(&mut self) -> Result<Option<RawFrame>, String> {
        Err(UNAVAILABLE.into())
    }
}

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
}

impl VideoReader {
    /// Opens `path` and reports what it holds.
    pub fn open(path: &Path) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel::<Cmd>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<VideoInfo, String>>();
        let owned = path.to_path_buf();
        let thread = thread::Builder::new()
            .name("video".into())
            .spawn(move || {
                let mut dec = match Decoder::open(&owned) {
                    Ok((d, info)) => {
                        let _ = ready_tx.send(Ok(info));
                        d
                    }
                    Err(e) => {
                        let _ = ready_tx.send(Err(e));
                        return;
                    }
                };
                while let Ok(cmd) = rx.recv() {
                    match cmd {
                        Cmd::At {
                            seconds,
                            max_width,
                            reply,
                        } => {
                            let q = dec.quarter_turns();
                            let r = dec
                                .frame_at(seconds)
                                .map(|f| scale(&rotate(f, q), max_width));
                            let _ = reply.send(r);
                        }
                        Cmd::Next { max_width, reply } => {
                            let q = dec.quarter_turns();
                            let r = dec
                                .next_frame()
                                .map(|f| f.map(|f| scale(&rotate(f, q), max_width)));
                            let _ = reply.send(r);
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
        })
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
    fn opening_says_why_when_unavailable() {
        let e = VideoReader::open(Path::new("nothing.mp4")).err().unwrap();
        assert_eq!(e, UNAVAILABLE);
    }
}
