// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Playback speed: plays the mix faster or slower without changing its
//! pitch, for listening and reviewing. Listening only: exports, saved
//! files and recording never see it.
//!
//! The mix is rendered in timeline time as always. At a speed other than
//! 1x it then goes through a pitch-preserving stretcher on the render
//! thread, and the stretched audio goes to the device. The device
//! thread is unchanged.

use std::sync::atomic::{AtomicU32, Ordering};

use rtrb::Producer;

use crate::dsp::stretch::Stretcher;

/// Slowest and fastest speed.
pub const MIN_SPEED: f32 = 0.5;
pub const MAX_SPEED: f32 = 2.0;

static SPEED: AtomicU32 = AtomicU32::new(0x3f80_0000);

/// The speed asked for, within range; values near 1 are exactly 1.
pub fn clamp(v: f32) -> f32 {
    if !v.is_finite() || (v - 1.0).abs() < 0.005 {
        return 1.0;
    }
    v.clamp(MIN_SPEED, MAX_SPEED)
}

/// Sets the speed for the next time playback starts (the caller
/// restarts playback that is running).
pub fn set_speed(v: f32) {
    SPEED.store(clamp(v).to_bits(), Ordering::Relaxed);
}

pub fn speed() -> f32 {
    f32::from_bits(SPEED.load(Ordering::Relaxed))
}

/// Timeline frames per output frame at `speed`.
pub fn timeline_frames(output_frames: u64, speed: f32) -> u64 {
    (output_frames as f64 * f64::from(speed)).round() as u64
}

/// Stretches rendered blocks and holds the result until the device's
/// queues have room.
pub struct SpeedStage {
    stretcher: Stretcher,
    /// Stretched audio waiting for room, stereo interleaved.
    pending: Vec<f32>,
    /// Silence for the metronome's queue, which stays muted at this speed.
    zeros: Vec<f32>,
    speed: f32,
}

impl SpeedStage {
    pub fn new(speed: f32, rate: u32) -> Self {
        Self {
            stretcher: Stretcher::new(1.0 / f64::from(speed), rate),
            pending: Vec::new(),
            zeros: Vec::new(),
            speed,
        }
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Frames stretched and waiting.
    pub fn buffered(&self) -> usize {
        self.pending.len() / 2
    }

    /// Takes a rendered block (stereo interleaved, timeline time).
    pub fn feed(&mut self, block: &[f32]) {
        self.stretcher.push(block);
        self.stretcher.pull(&mut self.pending);
    }

    /// Moves what fits into the mix queue, and the same number of silent
    /// frames into the metronome's. Returns the frames moved.
    pub fn flush(&mut self, mix: &mut Producer<f32>, click: &mut Producer<f32>) -> usize {
        let room = mix.slots().min(click.slots()) / 2;
        let m = self.buffered().min(room);
        if m == 0 {
            return 0;
        }
        let (_, left) = mix.push_partial_slice(&self.pending[..m * 2]);
        debug_assert!(left.is_empty(), "room was checked first");
        self.zeros.clear();
        self.zeros.resize(m * 2, 0.0);
        let (_, left) = click.push_partial_slice(&self.zeros);
        debug_assert!(left.is_empty(), "room was checked first");
        self.pending.drain(..m * 2);
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtrb::RingBuffer;

    const RATE: u32 = 48_000;

    fn tone(frames: usize) -> Vec<f32> {
        (0..frames)
            .flat_map(|i| {
                let v = (std::f32::consts::TAU * 440.0 * i as f32 / RATE as f32).sin() * 0.5;
                [v, v]
            })
            .collect()
    }

    #[test]
    fn speeds_are_kept_in_range_and_near_one_is_one() {
        assert_eq!(clamp(1.003), 1.0);
        assert_eq!(clamp(0.1), MIN_SPEED);
        assert_eq!(clamp(9.0), MAX_SPEED);
        assert_eq!(clamp(f32::NAN), 1.0);
        assert_eq!(clamp(1.5), 1.5);
        assert_eq!(timeline_frames(48_000, 1.5), 72_000);
    }

    #[test]
    fn half_speed_makes_twice_the_audio_and_a_silent_click() {
        let mut st = SpeedStage::new(0.5, RATE);
        let (mut p, mut c) = RingBuffer::<f32>::new(1 << 20);
        let (mut kp, mut kc) = RingBuffer::<f32>::new(1 << 20);
        let input = tone(RATE as usize);
        for block in input.chunks(2048 * 2) {
            st.feed(block);
            st.flush(&mut p, &mut kp);
        }
        let out = c.slots();
        // One second in, about two out, less what the last frames still hold.
        assert!((70_000..100_000).contains(&(out / 2)), "got {}", out / 2);
        assert_eq!(kc.slots(), out);
        for _ in 0..kc.slots() {
            assert_eq!(kc.pop().unwrap(), 0.0);
        }
        let peak = (0..out).map(|_| c.pop().unwrap().abs()).fold(0.0, f32::max);
        assert!(peak > 0.3 && peak < 0.8, "peak {peak}");
    }

    #[test]
    fn double_speed_makes_half_the_audio_and_waits_for_room() {
        let mut st = SpeedStage::new(2.0, RATE);
        let (mut p, c) = RingBuffer::<f32>::new(1 << 20);
        let (mut kp, _kc) = RingBuffer::<f32>::new(1 << 20);
        for block in tone(RATE as usize).chunks(2048 * 2) {
            st.feed(block);
            st.flush(&mut p, &mut kp);
        }
        let out = c.slots() / 2;
        assert!((18_000..30_000).contains(&out), "got {out}");

        // A small queue takes only what fits; the rest waits.
        let mut st = SpeedStage::new(0.5, RATE);
        let (mut p, c) = RingBuffer::<f32>::new(1000);
        let (mut kp, _kc) = RingBuffer::<f32>::new(1000);
        for block in tone(RATE as usize).chunks(2048 * 2) {
            st.feed(block);
            st.flush(&mut p, &mut kp);
        }
        assert_eq!(c.slots(), 1000);
        assert!(st.buffered() > 0);
    }
}
