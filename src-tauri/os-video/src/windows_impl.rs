// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Video through Media Foundation's source reader.
//!
//! The reader opens the file, picks the first video stream and hands back
//! pictures as 32-bit color (it runs the decoder's own color conversion).
//! A seek goes to the key frame before the time and decodes forward, so a
//! file with far-apart key frames takes longer to land on a picture.

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::core::{Interface, GUID, PCWSTR};
use windows::Win32::Media::MediaFoundation::{
    IMF2DBuffer, IMFAttributes, IMFSample, IMFSourceReader, MFCreateAttributes, MFCreateMediaType,
    MFCreateSourceReaderFromURL, MFMediaType_Video, MFShutdown, MFStartup, MFVideoFormat_RGB32,
    MFSTARTUP_LITE, MF_MT_DEFAULT_STRIDE, MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE, MF_MT_MAJOR_TYPE,
    MF_MT_SUBTYPE, MF_MT_VIDEO_ROTATION, MF_PD_DURATION, MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING,
    MF_VERSION,
};
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED};

use super::{Backend, RawFrame, VideoInfo};

/// The source reader's names for "the first video stream", "the first audio
/// stream", "every stream" and "the file itself".
const FIRST_VIDEO: u32 = 0xFFFF_FFFC;
const FIRST_AUDIO: u32 = 0xFFFF_FFFD;
const ALL_STREAMS: u32 = 0xFFFF_FFFE;
const MEDIA_SOURCE: u32 = 0xFFFF_FFFF;
/// Stream flags from a read: an error, and the end of the stream.
const FLAG_ERROR: u32 = 0x1;
const FLAG_END: u32 = 0x2;

/// Most pictures decoded and thrown away to land on one time after a seek.
const MAX_DECODE: usize = 1500;

fn err(what: &str, e: windows::core::Error) -> String {
    format!("Video ({what}): {e}")
}

/// Media Foundation and COM started for this thread; stopped when dropped.
struct Mf {
    com: bool,
}

impl Mf {
    fn start() -> Result<Self, String> {
        unsafe {
            // Fine if COM is already set up on this thread in either mode.
            let com = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
            if let Err(e) = MFStartup(MF_VERSION, MFSTARTUP_LITE) {
                if com {
                    CoUninitialize();
                }
                return Err(err("start", e));
            }
            Ok(Self { com })
        }
    }
}

impl Drop for Mf {
    fn drop(&mut self) {
        unsafe {
            let _ = MFShutdown();
            if self.com {
                CoUninitialize();
            }
        }
    }
}

pub struct Decoder {
    reader: IMFSourceReader,
    width: u32,
    height: u32,
    /// Bytes per row as the media type reports it; negative means bottom row first.
    stride: i32,
    /// One frame in 100-nanosecond units.
    frame_len: i64,
    /// Clockwise quarter turns that stand the picture upright.
    turns: u32,
    // Last, so it is dropped after the reader.
    _mf: Mf,
}

impl Decoder {
    /// The next decoded picture and its time (100 ns units), or None at the end.
    unsafe fn read(&self) -> Result<Option<(IMFSample, i64)>, String> {
        // A read can come back with no picture (a gap marker); ask again.
        for _ in 0..64 {
            let mut flags = 0u32;
            let mut at = 0i64;
            let mut sample: Option<IMFSample> = None;
            self.reader
                .ReadSample(
                    FIRST_VIDEO,
                    0,
                    None,
                    Some(&mut flags),
                    Some(&mut at),
                    Some(&mut sample),
                )
                .map_err(|e| err("read", e))?;
            if flags & FLAG_ERROR != 0 {
                return Err("Video: the decoder reported an error.".into());
            }
            if let Some(s) = sample {
                return Ok(Some((s, at)));
            }
            if flags & FLAG_END != 0 {
                return Ok(None);
            }
        }
        Ok(None)
    }

    /// Copies a decoded sample into rows of our own, top row first.
    unsafe fn raw(&self, sample: &IMFSample, at: i64) -> Result<RawFrame, String> {
        let w = self.width as usize;
        let h = self.height as usize;
        let row = w * 4;
        let buf = sample
            .ConvertToContiguousBuffer()
            .map_err(|e| err("buffer", e))?;
        let mut bgra = vec![0u8; row * h];
        if let Ok(b2) = buf.cast::<IMF2DBuffer>() {
            // The 2D lock gives the top row and a signed pitch, whichever way the rows run.
            let mut scan0: *mut u8 = std::ptr::null_mut();
            let mut pitch = 0i32;
            b2.Lock2D(&mut scan0, &mut pitch)
                .map_err(|e| err("buffer", e))?;
            for y in 0..h {
                let src = scan0.offset(y as isize * pitch as isize);
                std::ptr::copy_nonoverlapping(src, bgra.as_mut_ptr().add(y * row), row);
            }
            b2.Unlock2D().map_err(|e| err("buffer", e))?;
        } else {
            let mut ptr: *mut u8 = std::ptr::null_mut();
            buf.Lock(&mut ptr, None, None)
                .map_err(|e| err("buffer", e))?;
            let pitch = self.stride.unsigned_abs() as usize;
            for y in 0..h {
                // A negative stride means the bottom row comes first in memory.
                let from = if self.stride < 0 { h - 1 - y } else { y };
                std::ptr::copy_nonoverlapping(
                    ptr.add(from * pitch),
                    bgra.as_mut_ptr().add(y * row),
                    row,
                );
            }
            buf.Unlock().map_err(|e| err("buffer", e))?;
        }
        Ok(RawFrame {
            width: self.width,
            height: self.height,
            stride: row,
            bgra,
            time: at as f64 / 10_000_000.0,
        })
    }
}

impl Backend for Decoder {
    fn open(path: &Path) -> Result<(Self, VideoInfo), String> {
        let mf = Mf::start()?;
        unsafe {
            let wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            // Let the reader convert the picture to plain color for us.
            let mut attrs: Option<IMFAttributes> = None;
            MFCreateAttributes(&mut attrs, 1).map_err(|e| err("settings", e))?;
            let attrs = attrs.ok_or("Video: could not make settings.")?;
            attrs
                .SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1)
                .map_err(|e| err("settings", e))?;
            let reader = MFCreateSourceReaderFromURL(PCWSTR(wide.as_ptr()), &attrs)
                .map_err(|e| err("open", e))?;
            reader
                .SetStreamSelection(ALL_STREAMS, false)
                .map_err(|e| err("streams", e))?;
            reader
                .SetStreamSelection(FIRST_VIDEO, true)
                .map_err(|e| err("streams", e))?;
            let want = MFCreateMediaType().map_err(|e| err("format", e))?;
            let a: &IMFAttributes = &want;
            a.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
                .map_err(|e| err("format", e))?;
            a.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)
                .map_err(|e| err("format", e))?;
            reader
                .SetCurrentMediaType(FIRST_VIDEO, None, &want)
                .map_err(|e| err("format", e))?;
            let got = reader
                .GetCurrentMediaType(FIRST_VIDEO)
                .map_err(|e| err("format", e))?;
            let g: &IMFAttributes = &got;
            let size = g.GetUINT64(&MF_MT_FRAME_SIZE).map_err(|e| err("size", e))?;
            let (width, height) = ((size >> 32) as u32, size as u32);
            if width == 0 || height == 0 {
                return Err("Video: the picture has no size.".into());
            }
            let rate = g.GetUINT64(&MF_MT_FRAME_RATE).unwrap_or(0);
            let fps = if (rate as u32) > 0 {
                f64::from((rate >> 32) as u32) / f64::from(rate as u32)
            } else {
                0.0
            };
            let stride = g
                .GetUINT32(&MF_MT_DEFAULT_STRIDE)
                .map(|v| v as i32)
                .unwrap_or(width as i32 * 4);
            let hns = reader
                .GetPresentationAttribute(MEDIA_SOURCE, &MF_PD_DURATION)
                .ok()
                .and_then(|v: PROPVARIANT| u64::try_from(&v).ok())
                .unwrap_or(0);
            // The rotation the file asks for (phone clips). Media Foundation names it in
            // degrees counterclockwise; flip this if a vertical clip comes out upside down.
            let degrees = reader
                .GetNativeMediaType(FIRST_VIDEO, 0)
                .ok()
                .and_then(|n| {
                    let a: &IMFAttributes = &n;
                    a.GetUINT32(&MF_MT_VIDEO_ROTATION).ok()
                })
                .unwrap_or(0);
            let turns = ((360 - degrees % 360) / 90) % 4;
            let (shown_w, shown_h) = if turns % 2 == 1 {
                (height, width)
            } else {
                (width, height)
            };
            let has_audio = reader.GetNativeMediaType(FIRST_AUDIO, 0).is_ok();
            let info = VideoInfo {
                width: shown_w,
                height: shown_h,
                fps,
                duration: hns as f64 / 10_000_000.0,
                has_audio,
            };
            let frame_len = if fps > 0.0 {
                (10_000_000.0 / fps) as i64
            } else {
                333_333
            };
            Ok((
                Self {
                    reader,
                    width,
                    height,
                    stride,
                    frame_len,
                    turns,
                    _mf: mf,
                },
                info,
            ))
        }
    }

    fn frame_at(&mut self, seconds: f64) -> Result<RawFrame, String> {
        unsafe {
            let target = (seconds.max(0.0) * 10_000_000.0) as i64;
            self.reader
                .SetCurrentPosition(&GUID::zeroed(), &PROPVARIANT::from(target))
                .map_err(|e| err("seek", e))?;
            let slack = self.frame_len / 2;
            let mut last: Option<(IMFSample, i64)> = None;
            for _ in 0..MAX_DECODE {
                match self.read()? {
                    None => break,
                    Some((s, at)) => {
                        let reached = at + slack >= target;
                        last = Some((s, at));
                        if reached {
                            break;
                        }
                    }
                }
            }
            match last {
                Some((s, at)) => self.raw(&s, at),
                None => Err("Video: there is no picture at that time.".into()),
            }
        }
    }

    fn next_frame(&mut self) -> Result<Option<RawFrame>, String> {
        unsafe {
            match self.read()? {
                Some((s, at)) => self.raw(&s, at).map(Some),
                None => Ok(None),
            }
        }
    }

    fn quarter_turns(&self) -> u32 {
        self.turns
    }
}
