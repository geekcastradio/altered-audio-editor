// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Puts a finished mix into a copy of a video file: the picture is copied as it is (no
//! re-encoding), the sound is the AAC track of an `.m4a` the app exported, and any sound the video
//! had is dropped.
//!
//! Works for H.264 in `.mp4`, `.m4v` and `.mov` files. Other picture formats are refused with a
//! clear message, so nothing is half written. The sound starts where the picture starts: export the
//! mix from the time the video's first picture plays.
//!
//! A video with B frames starts its first picture late and hides the gap in an edit list this
//! writer does not copy. Every composition offset is shifted by the first picture's lateness, so
//! picture and sound start together.

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use mp4::{
    AacConfig, AvcConfig, MediaConfig, MediaType, Mp4Config, Mp4Reader, Mp4Sample, Mp4Writer,
    TrackConfig, TrackType,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MuxError {
    #[error("Could not read or write a file: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Mp4(#[from] mp4::Error),
    #[error("This video has no picture track the app can copy.")]
    NoVideo,
    #[error("The picture in this file is not H.264, which is the only kind that can be copied without re-encoding.")]
    UnsupportedVideo,
    #[error("The exported sound file has no AAC track.")]
    NoAudio,
}

/// What was written, for the notice shown after export.
#[derive(Debug, Clone, PartialEq)]
pub struct MuxReport {
    pub video_samples: u32,
    pub audio_samples: u32,
    /// Sound tracks the source video had that are not in the copy.
    pub dropped_audio_tracks: usize,
    /// Seconds the first picture sat late in the source, now removed.
    pub shifted_seconds: f64,
}

/// How many pictures at the start are looked at to find the first one shown.
const LOOKAHEAD: u32 = 64;

/// Writes `out` from the picture of `video` and the AAC sound of `audio_m4a`.
pub fn mux(video: &Path, audio_m4a: &Path, out: &Path) -> Result<MuxReport, MuxError> {
    let vf = File::open(video)?;
    let vsize = vf.metadata()?.len();
    let mut vr = Mp4Reader::read_header(BufReader::new(vf), vsize)?;
    let af = File::open(audio_m4a)?;
    let asize = af.metadata()?.len();
    let mut ar = Mp4Reader::read_header(BufReader::new(af), asize)?;

    let (vid_id, avc, vts, vcount) = {
        let t = vr
            .tracks()
            .values()
            .filter(|t| t.track_type().ok() == Some(TrackType::Video))
            .min_by_key(|t| t.track_id())
            .ok_or(MuxError::NoVideo)?;
        if t.media_type().ok() != Some(MediaType::H264) {
            return Err(MuxError::UnsupportedVideo);
        }
        (
            t.track_id(),
            AvcConfig {
                width: t.width(),
                height: t.height(),
                seq_param_set: t.sequence_parameter_set()?.to_vec(),
                pic_param_set: t.picture_parameter_set()?.to_vec(),
            },
            t.timescale(),
            t.sample_count(),
        )
    };
    let dropped_audio_tracks = vr
        .tracks()
        .values()
        .filter(|t| t.track_type().ok() == Some(TrackType::Audio))
        .count();

    let (aud_id, aac, ats, acount) = {
        let t = ar
            .tracks()
            .values()
            .filter(|t| t.media_type().ok() == Some(MediaType::AAC))
            .min_by_key(|t| t.track_id())
            .ok_or(MuxError::NoAudio)?;
        (
            t.track_id(),
            AacConfig {
                bitrate: t.bitrate(),
                profile: t.audio_profile()?,
                freq_index: t.sample_freq_index()?,
                chan_conf: t.channel_config()?,
            },
            t.timescale(),
            t.sample_count(),
        )
    };

    // The first picture shown, found among the first few in decode order.
    let mut shift: i64 = i64::MAX;
    for s in 1..=vcount.min(LOOKAHEAD) {
        if let Some(smp) = vr.read_sample(vid_id, s)? {
            shift = shift.min(i64::from(smp.rendering_offset) + smp.start_time as i64);
        }
    }
    if shift == i64::MAX || shift < 0 {
        shift = 0;
    }

    let file = BufWriter::new(File::create(out)?);
    let result = write(
        &mut vr, &mut ar, file, vid_id, avc, vts, vcount, aud_id, aac, ats, acount, shift,
    );
    if result.is_err() {
        let _ = std::fs::remove_file(out);
    }
    let (video_samples, audio_samples) = result?;
    Ok(MuxReport {
        video_samples,
        audio_samples,
        dropped_audio_tracks,
        shifted_seconds: shift as f64 / f64::from(vts.max(1)),
    })
}

#[allow(clippy::too_many_arguments)]
fn write(
    vr: &mut Mp4Reader<BufReader<File>>,
    ar: &mut Mp4Reader<BufReader<File>>,
    file: BufWriter<File>,
    vid_id: u32,
    avc: AvcConfig,
    vts: u32,
    vcount: u32,
    aud_id: u32,
    aac: AacConfig,
    ats: u32,
    acount: u32,
    shift: i64,
) -> Result<(u32, u32), MuxError> {
    let brand = |s: &str| str::parse(s).expect("four characters");
    let mut wr = Mp4Writer::write_start(
        file,
        &Mp4Config {
            major_brand: brand("isom"),
            minor_version: 512,
            compatible_brands: vec![brand("isom"), brand("iso2"), brand("avc1"), brand("mp41")],
            timescale: 1000,
        },
    )?;
    wr.add_track(&TrackConfig {
        track_type: TrackType::Video,
        timescale: vts,
        language: "und".into(),
        media_conf: MediaConfig::AvcConfig(avc),
    })?;
    wr.add_track(&TrackConfig {
        track_type: TrackType::Audio,
        timescale: ats,
        language: "und".into(),
        media_conf: MediaConfig::AacConfig(aac),
    })?;

    // Picture and sound are written in step, so the file plays before it has all arrived.
    let mut next_audio = 1u32;
    let mut audio_written = 0u32;
    let mut write_audio_until =
        |wr: &mut Mp4Writer<BufWriter<File>>, seconds: f64| -> Result<(), MuxError> {
            while next_audio <= acount {
                let Some(s) = ar.read_sample(aud_id, next_audio)? else {
                    next_audio += 1;
                    continue;
                };
                if s.start_time as f64 / f64::from(ats.max(1)) > seconds {
                    break;
                }
                wr.write_sample(
                    2,
                    &Mp4Sample {
                        start_time: s.start_time,
                        duration: s.duration,
                        rendering_offset: 0,
                        is_sync: true,
                        bytes: s.bytes,
                    },
                )?;
                audio_written += 1;
                next_audio += 1;
            }
            Ok(())
        };
    let mut video_written = 0u32;
    for id in 1..=vcount {
        let Some(mut s) = vr.read_sample(vid_id, id)? else {
            continue;
        };
        write_audio_until(&mut wr, s.start_time as f64 / f64::from(vts.max(1)))?;
        s.rendering_offset -= shift as i32;
        wr.write_sample(1, &s)?;
        video_written += 1;
    }
    write_audio_until(&mut wr, f64::MAX)?;
    wr.write_end()?;
    Ok((video_written, audio_written))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn ffmpeg(args: &[&str]) -> bool {
        Command::new("ffmpeg")
            .args(["-y", "-v", "error"])
            .args(args)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Makes a test video with B frames and its own sound, and an AAC file. False when ffmpeg is absent.
    fn make(dir: &Path, codec: &str, bframes: bool) -> bool {
        let v = dir.join("v.mp4");
        let a = dir.join("a.m4a");
        let b = if bframes { "3" } else { "0" };
        ffmpeg(&[
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=160x120:rate=25:duration=2",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=330:duration=2",
            "-c:v",
            codec,
            "-bf",
            b,
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-shortest",
            v.to_str().unwrap(),
        ]) && ffmpeg(&[
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=880:duration=2.1",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            a.to_str().unwrap(),
        ])
    }

    fn probe(path: &Path, what: &str) -> String {
        let o = Command::new("ffprobe")
            .args(["-v", "error", "-show_entries", what, "-of", "csv=p=0"])
            .arg(path)
            .output()
            .unwrap();
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    #[test]
    fn copies_the_picture_and_swaps_the_sound() {
        let d = tempfile::tempdir().unwrap();
        if !make(d.path(), "libx264", true) {
            return;
        }
        let out = d.path().join("out.mp4");
        let r = mux(&d.path().join("v.mp4"), &d.path().join("a.m4a"), &out).unwrap();
        assert_eq!(r.video_samples, 50);
        assert_eq!(r.dropped_audio_tracks, 1);
        assert!(r.audio_samples > 90);
        assert_eq!(probe(&out, "stream=codec_name"), "h264\naac");
        let start = probe(&out, "stream=start_time");
        for t in start.lines() {
            assert!(t.parse::<f64>().unwrap() < 0.1, "start {t}");
        }
        // The picture is the same bytes as the source.
        let hash = |p: &Path| {
            let o = Command::new("ffmpeg")
                .args(["-v", "error", "-i"])
                .arg(p)
                .args(["-map", "0:v", "-c", "copy", "-f", "md5", "-"])
                .output()
                .unwrap();
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        };
        assert_eq!(hash(&d.path().join("v.mp4")), hash(&out));
    }

    #[test]
    fn a_file_without_b_frames_works_too() {
        let d = tempfile::tempdir().unwrap();
        if !make(d.path(), "libx264", false) {
            return;
        }
        let out = d.path().join("out.mp4");
        let r = mux(&d.path().join("v.mp4"), &d.path().join("a.m4a"), &out).unwrap();
        assert_eq!(r.shifted_seconds, 0.0);
        assert_eq!(probe(&out, "stream=codec_name"), "h264\naac");
    }

    #[test]
    fn other_picture_formats_are_refused_and_nothing_is_left_behind() {
        let d = tempfile::tempdir().unwrap();
        if !make(d.path(), "libx265", false) {
            return;
        }
        let out = d.path().join("out.mp4");
        let e = mux(&d.path().join("v.mp4"), &d.path().join("a.m4a"), &out).unwrap_err();
        assert!(matches!(e, MuxError::UnsupportedVideo), "{e}");
        assert!(!out.exists());
    }

    #[test]
    fn a_file_with_no_sound_is_refused() {
        let d = tempfile::tempdir().unwrap();
        if !make(d.path(), "libx264", false) {
            return;
        }
        let silent = d.path().join("silent.mp4");
        let v = d.path().join("v.mp4");
        if !ffmpeg(&[
            "-i",
            v.to_str().unwrap(),
            "-an",
            "-c:v",
            "copy",
            silent.to_str().unwrap(),
        ]) {
            return;
        }
        let e = mux(&v, &silent, &d.path().join("o.mp4"));
        assert!(matches!(e, Err(MuxError::NoAudio)));
    }
}
