// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Video through AVFoundation's asset reader (macOS).
//!
//! A first version written without a Mac to build it on: it is behind the
//! `apple` feature until it has been compiled and tried. The reader decodes
//! the first video track to 32-bit color. A seek starts a new reader a little
//! before the time and decodes forward, so the system takes care of the key
//! frame before it.

use std::path::Path;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_av_foundation::{
    AVAssetReader, AVAssetReaderTrackOutput, AVAssetTrack, AVMediaTypeAudio, AVMediaTypeVideo,
    AVURLAsset,
};
use objc2_core_media::{CMTime, CMTimeRange};
use objc2_core_video::{
    CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
    CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags,
    CVPixelBufferUnlockBaseAddress,
};
use objc2_foundation::{ns_string, NSDictionary, NSNumber, NSString, NSURL};

use super::{Backend, RawFrame, VideoInfo};

/// `kCVPixelFormatType_32BGRA`: blue, green, red, alpha bytes.
const BGRA: u32 = 0x4247_5241;
/// Most pictures decoded and thrown away to land on one time after a seek.
const MAX_DECODE: usize = 240;
/// Timescale for the times we hand the reader.
const SCALE: i32 = 6000;

/// A reader that is part way through the file, and the output it feeds.
struct Running {
    reader: Retained<AVAssetReader>,
    output: Retained<AVAssetReaderTrackOutput>,
}

impl Drop for Running {
    fn drop(&mut self) {
        unsafe { self.reader.cancelReading() };
    }
}

pub struct Decoder {
    asset: Retained<AVURLAsset>,
    track: Retained<AVAssetTrack>,
    duration: f64,
    frame_len: f64,
    turns: u32,
    cur: Option<Running>,
}

fn reason(what: &str, e: Option<Retained<objc2_foundation::NSError>>) -> String {
    match e {
        Some(e) => format!("Video ({what}): {}", e.localizedDescription()),
        None => format!("Video ({what}): the system could not read this file."),
    }
}

impl Decoder {
    /// Starts a reader at `from` seconds and runs to the end of the video.
    unsafe fn start(&mut self, from: f64) -> Result<(), String> {
        self.cur = None;
        let reader = AVAssetReader::assetReaderWithAsset_error(&self.asset)
            .map_err(|e| reason("read", Some(e)))?;
        let num = NSNumber::new_u32(BGRA);
        // SAFETY: an NSNumber is an object like any other; the dictionary only stores it.
        let value: &AnyObject = &*(&*num as *const NSNumber as *const AnyObject);
        let settings = NSDictionary::<NSString, AnyObject>::from_slices(
            &[ns_string!("PixelFormatType")],
            &[value],
        );
        let output = AVAssetReaderTrackOutput::assetReaderTrackOutputWithTrack_outputSettings(
            &self.track,
            Some(&settings),
        );
        output.setAlwaysCopiesSampleData(false);
        reader.addOutput(&output);
        let from = from.clamp(0.0, self.duration.max(0.0));
        let left = (self.duration - from).max(0.0) + 1.0;
        reader.setTimeRange(CMTimeRange {
            start: CMTime::with_seconds(from, SCALE),
            duration: CMTime::with_seconds(left, SCALE),
        });
        if !reader.startReading() {
            return Err(reason("start", reader.error()));
        }
        self.cur = Some(Running { reader, output });
        Ok(())
    }

    /// The next decoded picture, or None at the end.
    unsafe fn read(&mut self) -> Result<Option<RawFrame>, String> {
        let Some(cur) = self.cur.as_ref() else {
            return Ok(None);
        };
        loop {
            let Some(sample) = cur.output.copyNextSampleBuffer() else {
                return Ok(None);
            };
            let Some(image) = sample.image_buffer() else {
                continue;
            };
            let time = sample.presentation_time_stamp().seconds();
            if CVPixelBufferLockBaseAddress(&image, CVPixelBufferLockFlags::ReadOnly) != 0 {
                return Err("Video (buffer): the picture could not be read.".into());
            }
            let width = CVPixelBufferGetWidth(&image);
            let height = CVPixelBufferGetHeight(&image);
            let stride = CVPixelBufferGetBytesPerRow(&image);
            let base = CVPixelBufferGetBaseAddress(&image) as *const u8;
            let row = width * 4;
            let mut bgra = vec![0u8; row * height];
            if !base.is_null() && stride >= row {
                for y in 0..height {
                    std::ptr::copy_nonoverlapping(
                        base.add(y * stride),
                        bgra.as_mut_ptr().add(y * row),
                        row,
                    );
                }
            }
            CVPixelBufferUnlockBaseAddress(&image, CVPixelBufferLockFlags::ReadOnly);
            if base.is_null() {
                return Err("Video (buffer): the picture has no data.".into());
            }
            return Ok(Some(RawFrame {
                width: width as u32,
                height: height as u32,
                stride: row,
                bgra,
                time,
            }));
        }
    }
}

impl Backend for Decoder {
    #[allow(deprecated)]
    fn open(path: &Path) -> Result<(Self, VideoInfo), String> {
        unsafe {
            let text = path
                .to_str()
                .ok_or("Video: that file name can't be read.")?;
            let url = NSURL::fileURLWithPath_isDirectory(&NSString::from_str(text), false);
            let asset = AVURLAsset::URLAssetWithURL_options(&url, None);
            let video = AVMediaTypeVideo.ok_or("Video: this system has no video support.")?;
            let track = asset
                .tracksWithMediaType(video)
                .firstObject()
                .ok_or("Video: this file has no picture.")?;
            let size = track.naturalSize();
            if size.width < 1.0 || size.height < 1.0 {
                return Err("Video: the picture has no size.".into());
            }
            let t = track.preferredTransform();
            // The rotation phones record: clockwise quarter turns to stand it upright.
            let turns = ((t.b.atan2(t.a) / std::f64::consts::FRAC_PI_2).round() as i32)
                .rem_euclid(4) as u32;
            let (w, h) = (size.width.round() as u32, size.height.round() as u32);
            let (w, h) = if turns % 2 == 1 { (h, w) } else { (w, h) };
            let fps = f64::from(track.nominalFrameRate());
            let duration = asset.duration().seconds();
            let duration = if duration.is_finite() && duration > 0.0 {
                duration
            } else {
                0.0
            };
            let has_audio = AVMediaTypeAudio
                .map(|a| asset.tracksWithMediaType(a).count() > 0)
                .unwrap_or(false);
            let info = VideoInfo {
                width: w,
                height: h,
                fps,
                duration,
                has_audio,
            };
            let frame_len = if fps > 0.0 { 1.0 / fps } else { 1.0 / 30.0 };
            Ok((
                Self {
                    asset,
                    track,
                    duration,
                    frame_len,
                    turns,
                    cur: None,
                },
                info,
            ))
        }
    }

    fn frame_at(&mut self, seconds: f64) -> Result<RawFrame, String> {
        unsafe {
            let target = seconds.max(0.0);
            // Start a little early so the picture showing at `target` is not skipped.
            self.start((target - 2.0 * self.frame_len).max(0.0))?;
            let slack = self.frame_len * 0.5;
            let mut last: Option<RawFrame> = None;
            for _ in 0..MAX_DECODE {
                match self.read()? {
                    None => break,
                    Some(f) => {
                        let reached = f.time + slack >= target;
                        last = Some(f);
                        if reached {
                            break;
                        }
                    }
                }
            }
            last.ok_or_else(|| "Video: there is no picture at that time.".to_string())
        }
    }

    fn next_frame(&mut self) -> Result<Option<RawFrame>, String> {
        unsafe {
            if self.cur.is_none() {
                self.start(0.0)?;
            }
            self.read()
        }
    }

    fn quarter_turns(&self) -> u32 {
        self.turns
    }
}
