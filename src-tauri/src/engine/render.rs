// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Renders each track's clips into a stereo block at the output rate.
//! Runs on a worker thread ahead of the audio device, never on it: it
//! opens files, seeks and allocates.

use std::collections::HashMap;

use super::plan::{ClipPlan, Plan};
use crate::codecs::AudioReader;
use crate::session::model::Id;

/// An open clip being streamed.
struct Active {
    reader: Option<AudioReader>,
    /// Decoded samples not yet used (stereo interleaved).
    pending: Vec<f32>,
    used: usize,
    /// Looped clips: frames left before the region starts over.
    lap_left: u64,
}

impl Active {
    fn open(clip: &ClipPlan, offset_frames: u64, out_rate: u32) -> Self {
        // A looped clip starts wherever its phase and the offset land in the region.
        let (into, lap_left) = if clip.loop_len > 0 {
            let p = (clip.loop_phase + offset_frames) % clip.loop_len;
            (p, clip.loop_len - p)
        } else {
            (offset_frames, 0)
        };
        let reader = AudioReader::open(&clip.path, 2, Some(out_rate))
            .ok()
            .and_then(|mut r| {
                let secs = clip.source_start + into as f64 / f64::from(out_rate);
                if secs > 0.0 {
                    r.seek(secs).ok()?;
                }
                Some(r)
            });
        Self {
            reader,
            pending: Vec::new(),
            used: 0,
            lap_left,
        }
    }

    /// Looped clips: back to the start of the region.
    fn rewind(&mut self, clip: &ClipPlan) {
        self.pending.clear();
        self.used = 0;
        self.lap_left = clip.loop_len;
        if let Some(r) = self.reader.as_mut() {
            if r.seek(clip.source_start).is_err() {
                self.reader = None;
            }
        }
    }

    /// Adds `frames` frames into `out` (stereo), shaped by the clip's
    /// gain and fades. `rel` is the clip-relative frame of the first one.
    /// Silence if the file ends early or could not be opened.
    fn mix_into(&mut self, out: &mut [f32], frames: usize, clip: &ClipPlan, rel: u64) {
        let mut done = 0;
        while done < frames {
            if self.used >= self.pending.len() {
                self.pending.clear();
                self.used = 0;
                let more = match self.reader.as_mut() {
                    Some(r) => r.read_into(&mut self.pending).unwrap_or(false),
                    None => false,
                };
                if !more || self.pending.is_empty() {
                    // File ended or failed: the rest stays silent.
                    self.reader = None;
                    return;
                }
            }
            let avail = (self.pending.len() - self.used) / 2;
            let mut n = avail.min(frames - done);
            if clip.loop_len > 0 {
                n = n.min(self.lap_left as usize);
            }
            let src = &self.pending[self.used..self.used + n * 2];
            let dst = &mut out[done * 2..(done + n) * 2];
            if clip.is_flat() {
                let g = clip.gain;
                if g != 0.0 {
                    for (o, s) in dst.iter_mut().zip(src) {
                        *o += *s * g;
                    }
                }
            } else {
                let at = rel + done as u64;
                for (i, (o, s)) in dst
                    .as_chunks_mut::<2>()
                    .0
                    .iter_mut()
                    .zip(src.as_chunks::<2>().0)
                    .enumerate()
                {
                    let [l, r] = clip.gains_at(at + i as u64);
                    o[0] += s[0] * l;
                    o[1] += s[1] * r;
                }
            }
            self.used += n * 2;
            done += n;
            if clip.loop_len > 0 {
                self.lap_left -= n as u64;
                if self.lap_left == 0 {
                    self.rewind(clip);
                }
            }
        }
    }
}

pub struct Renderer {
    plan: Plan,
    /// Next output frame to render.
    pos: u64,
    active: Vec<HashMap<Id, Active>>,
    /// On reaching frame `.0`, continue from frame `.1`: back to the
    /// start for Loop Playback, past the selection for Skip Selection.
    jump: Option<(u64, u64)>,
}

impl Renderer {
    pub fn new(plan: Plan, start_frame: u64) -> Self {
        let n = plan.tracks.len();
        Self {
            plan,
            pos: start_frame,
            active: (0..n).map(|_| HashMap::new()).collect(),
            jump: None,
        }
    }

    /// Jumps from output frame `at` to `to` every time playback reaches
    /// `at`. Only applies while playback is before `at`.
    pub fn with_jump(mut self, at: u64, to: u64) -> Self {
        if at > self.pos && at != to {
            self.jump = Some((at, to));
        }
        self
    }

    pub fn track_count(&self) -> usize {
        self.plan.tracks.len()
    }

    pub fn position(&self) -> u64 {
        self.pos
    }

    /// Renders the next `frames` frames. `out[t]` receives track `t` as
    /// stereo interleaved samples and is resized to `frames * 2`.
    /// Stops short at a jump point, so it may render fewer frames.
    pub fn render(&mut self, frames: usize, out: &mut [Vec<f32>]) {
        let frames = match self.jump {
            Some((at, _)) if self.pos < at => frames.min((at - self.pos) as usize),
            _ => frames,
        };
        let start = self.pos;
        let end = start + frames as u64;
        let rate = self.plan.out_rate;
        for (t, track) in self.plan.tracks.iter().enumerate() {
            let buf = &mut out[t];
            buf.clear();
            buf.resize(frames * 2, 0.0);
            let active = &mut self.active[t];
            // Forget clips that are behind us.
            active.retain(|id, _| {
                track
                    .clips
                    .iter()
                    .any(|c| c.id == *id && c.start + c.len > start)
            });
            for clip in &track.clips {
                let c_end = clip.start + clip.len;
                if c_end <= start || clip.start >= end {
                    continue;
                }
                let from = clip.start.max(start);
                let to = c_end.min(end);
                let a = active
                    .entry(clip.id)
                    .or_insert_with(|| Active::open(clip, from - clip.start, rate));
                let offset = (from - start) as usize;
                let n = (to - from) as usize;
                a.mix_into(&mut buf[offset * 2..], n, clip, from - clip.start);
            }
        }
        self.pos = end;
        if let Some((at, to)) = self.jump {
            if end == at {
                self.pos = to;
                // Clips reopen at the new spot.
                for a in &mut self.active {
                    a.clear();
                }
                if to > at {
                    // Skips happen once.
                    self.jump = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::plan::TrackPlan;
    use crate::session::model::new_id;
    use std::path::PathBuf;

    fn click_file(dir: &std::path::Path) -> PathBuf {
        let path = dir.join("click.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        for i in 0..48_000 {
            w.write_sample(if i == 1_000 { i16::MAX } else { 0 })
                .unwrap();
        }
        w.finalize().unwrap();
        path
    }

    fn first_click(buf: &[f32]) -> Option<usize> {
        buf.as_chunks::<2>().0.iter().position(|f| f[0] > 0.5)
    }

    #[test]
    fn plays_clips_at_their_positions_across_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let path = click_file(dir.path());
        // Clip starts at frame 5,000 and plays from source frame 500, so
        // the click (source 1,000) lands at timeline frame 5,500.
        let plan = Plan {
            out_rate: 48_000,
            session_rate: 48_000,
            tracks: vec![TrackPlan {
                id: new_id(),
                clips: vec![ClipPlan::plain(
                    new_id(),
                    path,
                    5_000,
                    10_000,
                    500.0 / 48_000.0,
                )],
            }],
        };
        let mut r = Renderer::new(plan.clone(), 0);
        let mut out = vec![Vec::new()];
        let mut all = Vec::new();
        for _ in 0..12 {
            r.render(1_024, &mut out);
            all.extend_from_slice(&out[0]);
        }
        assert_eq!(first_click(&all), Some(5_500));

        // Starting playback in the middle of the clip seeks correctly.
        let mut r = Renderer::new(plan, 5_200);
        r.render(1_024, &mut out);
        assert_eq!(first_click(&out[0]), Some(300));
    }

    fn clicks_in(r: &mut Renderer, total: usize) -> Vec<usize> {
        let mut out = vec![Vec::new()];
        let mut all = Vec::new();
        while all.len() < total * 2 {
            r.render(1_000, &mut out);
            all.extend_from_slice(&out[0]);
        }
        all.as_chunks::<2>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, f)| f[0] > 0.5)
            .map(|(i, _)| i)
            .collect()
    }

    fn click_plan(path: PathBuf) -> Plan {
        // Click at timeline frame 5,500 (clip at 5,000, source from 500).
        Plan {
            out_rate: 48_000,
            session_rate: 48_000,
            tracks: vec![TrackPlan {
                id: new_id(),
                clips: vec![ClipPlan::plain(
                    new_id(),
                    path,
                    5_000,
                    10_000,
                    500.0 / 48_000.0,
                )],
            }],
        }
    }

    #[test]
    fn loop_jump_repeats_the_range() {
        let dir = tempfile::tempdir().unwrap();
        let plan = click_plan(click_file(dir.path()));
        // Loop 5,000..6,000 from 4,000: click at 1,500 rendered frames, then every 1,000.
        let mut r = Renderer::new(plan, 4_000).with_jump(6_000, 5_000);
        assert_eq!(clicks_in(&mut r, 4_600), vec![1_500, 2_500, 3_500, 4_500]);
    }

    #[test]
    fn skip_jump_plays_around_the_range_once() {
        let dir = tempfile::tempdir().unwrap();
        let plan = click_plan(click_file(dir.path()));
        // Skipping 5,400..5,600 from 5,000 leaves the click out.
        let mut r = Renderer::new(plan.clone(), 5_000).with_jump(5_400, 5_600);
        assert!(clicks_in(&mut r, 3_000).is_empty());
        // Skipping 5,000..5,200 keeps it, 200 frames sooner.
        let mut r = Renderer::new(plan, 4_000).with_jump(5_000, 5_200);
        assert_eq!(clicks_in(&mut r, 3_000), vec![1_300]);
    }

    #[test]
    fn a_looped_clip_repeats_its_region() {
        let dir = tempfile::tempdir().unwrap();
        let path = click_file(dir.path());
        // Region is source 500..2_500 (2,000 frames, click at 500 into it).
        // The clip starts 300 frames into the region and runs 7,000 frames.
        let mut clip = ClipPlan::plain(new_id(), path, 0, 7_000, 500.0 / 48_000.0);
        clip.loop_len = 2_000;
        clip.loop_phase = 300;
        let plan = Plan {
            out_rate: 48_000,
            session_rate: 48_000,
            tracks: vec![TrackPlan {
                id: new_id(),
                clips: vec![clip],
            }],
        };
        let clicks = |buf: &[f32]| -> Vec<usize> {
            buf.as_chunks::<2>()
                .0
                .iter()
                .enumerate()
                .filter(|(_, f)| f[0] > 0.5)
                .map(|(i, _)| i)
                .collect()
        };
        let mut r = Renderer::new(plan.clone(), 0);
        let mut out = vec![Vec::new()];
        let mut all = Vec::new();
        for _ in 0..7 {
            r.render(1_000, &mut out);
            all.extend_from_slice(&out[0]);
        }
        assert_eq!(clicks(&all), vec![200, 2_200, 4_200, 6_200], "once per lap");

        // Starting mid-clip lands on the right lap position.
        let mut r = Renderer::new(plan, 3_000);
        r.render(2_000, &mut out);
        assert_eq!(clicks(&out[0]), vec![1_200]);
    }

    #[test]
    fn missing_files_are_silent() {
        let plan = Plan {
            out_rate: 48_000,
            session_rate: 48_000,
            tracks: vec![TrackPlan {
                id: new_id(),
                clips: vec![ClipPlan::plain(
                    new_id(),
                    "definitely/missing.wav".into(),
                    0,
                    4_800,
                    0.0,
                )],
            }],
        };
        let mut r = Renderer::new(plan, 0);
        let mut out = vec![Vec::new()];
        r.render(512, &mut out);
        assert!(out[0].iter().all(|v| *v == 0.0));
        assert_eq!(r.position(), 512);
    }

    #[test]
    fn applies_clip_gain_and_fades() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dc.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        for _ in 0..10_000 {
            w.write_sample(16_384i16).unwrap();
        }
        w.finalize().unwrap();
        let mut clip = ClipPlan::plain(new_id(), path, 0, 10_000, 0.0);
        clip.gain = 0.5;
        clip.fade_in = 1_000;
        clip.fade_out = 2_000;
        let plan = Plan {
            out_rate: 48_000,
            session_rate: 48_000,
            tracks: vec![TrackPlan {
                id: new_id(),
                clips: vec![clip],
            }],
        };
        let mut r = Renderer::new(plan, 0);
        let mut out = vec![Vec::new()];
        r.render(10_000, &mut out);
        let at = |f: usize| out[0][f * 2];
        assert!(at(0).abs() < 1e-6, "fade in starts silent");
        assert!((at(500) - 0.125).abs() < 0.01, "halfway up the fade in");
        assert!(
            (at(5_000) - 0.25).abs() < 0.01,
            "full level times clip gain"
        );
        assert!(
            (at(9_000) - 0.125).abs() < 0.01,
            "halfway down the fade out"
        );
    }
}
