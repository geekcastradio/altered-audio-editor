// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Timecode math for interchange files (EDL, OTIO). Pure functions, no session types.

/// Frame rates the interchange exports offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameRate {
    F23_976,
    F24,
    F25,
    F29_97Ndf,
    F29_97Df,
    F30,
}

impl FrameRate {
    pub const ALL: [FrameRate; 6] = [
        FrameRate::F23_976,
        FrameRate::F24,
        FrameRate::F25,
        FrameRate::F29_97Ndf,
        FrameRate::F29_97Df,
        FrameRate::F30,
    ];

    /// The real rate as a fraction of frames per second.
    pub fn fraction(self) -> (u64, u64) {
        match self {
            FrameRate::F23_976 => (24_000, 1001),
            FrameRate::F24 => (24, 1),
            FrameRate::F25 => (25, 1),
            FrameRate::F29_97Ndf | FrameRate::F29_97Df => (30_000, 1001),
            FrameRate::F30 => (30, 1),
        }
    }

    /// Frames counted per timecode second (the label rate).
    pub fn nominal(self) -> u64 {
        match self {
            FrameRate::F23_976 | FrameRate::F24 => 24,
            FrameRate::F25 => 25,
            FrameRate::F29_97Ndf | FrameRate::F29_97Df | FrameRate::F30 => 30,
        }
    }

    pub fn is_drop(self) -> bool {
        self == FrameRate::F29_97Df
    }

    pub fn label(self) -> &'static str {
        match self {
            FrameRate::F23_976 => "23.976",
            FrameRate::F24 => "24",
            FrameRate::F25 => "25",
            FrameRate::F29_97Ndf => "29.97",
            FrameRate::F29_97Df => "29.97 drop frame",
            FrameRate::F30 => "30",
        }
    }
}

/// Audio samples to the nearest whole frame.
pub fn samples_to_frames(samples: u64, sample_rate: u32, rate: FrameRate) -> u64 {
    let (num, den) = rate.fraction();
    let rate = u128::from(sample_rate.max(1));
    let n = u128::from(samples) * u128::from(num);
    let d = rate * u128::from(den);
    ((n * 2 + d) / (d * 2)) as u64
}

/// Whole frames back to audio samples (nearest).
pub fn frames_to_samples(frames: u64, sample_rate: u32, rate: FrameRate) -> u64 {
    let (num, den) = rate.fraction();
    let n = u128::from(frames) * u128::from(den) * u128::from(sample_rate);
    let d = u128::from(num);
    ((n * 2 + d) / (d * 2)) as u64
}

/// A frame count as `HH:MM:SS:FF`, with `;` before the frames for drop frame.
pub fn frames_to_timecode(frames: u64, rate: FrameRate) -> String {
    let fps = rate.nominal();
    let mut n = frames;
    if rate.is_drop() {
        // Drop frame skips frame numbers 0 and 1 at the start of every minute except each tenth.
        let per_10min = 17_982; // 10 * 60 * 30 - 9 * 2
        let per_min = 1_798; // 60 * 30 - 2
        let tens = n / per_10min;
        let rest = n % per_10min;
        n += 18 * tens;
        if rest >= 2 {
            n += 2 * ((rest - 2) / per_min);
        }
    }
    let ff = n % fps;
    let ss = (n / fps) % 60;
    let mm = (n / (fps * 60)) % 60;
    let hh = n / (fps * 3600);
    let sep = if rate.is_drop() { ';' } else { ':' };
    format!("{hh:02}:{mm:02}:{ss:02}{sep}{ff:02}")
}

/// Parses `HH:MM:SS:FF` or `HH:MM:SS;FF` back to a frame count. `None` when it is not a timecode.
pub fn timecode_to_frames(text: &str, rate: FrameRate) -> Option<u64> {
    let parts: Vec<&str> = text.split([':', ';']).collect();
    if parts.len() != 4 {
        return None;
    }
    let mut v = [0u64; 4];
    for (slot, p) in v.iter_mut().zip(&parts) {
        *slot = p.parse().ok()?;
    }
    let [hh, mm, ss, ff] = v;
    let fps = rate.nominal();
    if ff >= fps || ss >= 60 || mm >= 60 {
        return None;
    }
    let total_minutes = hh * 60 + mm;
    let mut frames = (total_minutes * 60 + ss) * fps + ff;
    if rate.is_drop() {
        if mm % 10 != 0 && ss == 0 && ff < 2 {
            return None; // that label does not exist in drop frame
        }
        frames -= 2 * (total_minutes - total_minutes / 10);
    }
    Some(frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_seconds_at_24() {
        assert_eq!(frames_to_timecode(0, FrameRate::F24), "00:00:00:00");
        assert_eq!(frames_to_timecode(24, FrameRate::F24), "00:00:01:00");
        assert_eq!(frames_to_timecode(24 * 3600, FrameRate::F24), "01:00:00:00");
        assert_eq!(frames_to_timecode(25, FrameRate::F24), "00:00:01:01");
    }

    #[test]
    fn samples_round_to_the_nearest_frame() {
        // 1 second at 48 kHz is 24 frames at 24 fps and 25 frames at 25 fps.
        assert_eq!(samples_to_frames(48_000, 48_000, FrameRate::F24), 24);
        assert_eq!(samples_to_frames(48_000, 48_000, FrameRate::F25), 25);
        // Half a frame rounds up, just under half rounds down (frame = 2000 samples at 24 fps).
        assert_eq!(samples_to_frames(1_000, 48_000, FrameRate::F24), 1);
        assert_eq!(samples_to_frames(999, 48_000, FrameRate::F24), 0);
    }

    #[test]
    fn ntsc_rates_are_slightly_slower_than_their_label() {
        // One hour of real time is 107,892 frames at 29.97, not 108,000.
        let hour = 48_000u64 * 3600;
        assert_eq!(
            samples_to_frames(hour, 48_000, FrameRate::F29_97Ndf),
            107_892
        );
        assert_eq!(samples_to_frames(hour, 48_000, FrameRate::F23_976), 86_314);
    }

    #[test]
    fn drop_frame_skips_the_right_labels() {
        let df = FrameRate::F29_97Df;
        assert_eq!(frames_to_timecode(1_799, df), "00:00:59;29");
        assert_eq!(frames_to_timecode(1_800, df), "00:01:00;02");
        assert_eq!(frames_to_timecode(17_981, df), "00:09:59;29");
        // The tenth minute keeps its first two labels.
        assert_eq!(frames_to_timecode(17_982, df), "00:10:00;00");
        // One real hour at 29.97 reads one hour of timecode, which is the point of drop frame.
        assert_eq!(frames_to_timecode(107_892, df), "01:00:00;00");
    }

    #[test]
    fn every_rate_round_trips_across_a_long_stretch() {
        for rate in FrameRate::ALL {
            for f in (0..200_000u64)
                .step_by(7)
                .chain([17_981, 17_982, 17_983, 1_799, 1_800])
            {
                let tc = frames_to_timecode(f, rate);
                assert_eq!(timecode_to_frames(&tc, rate), Some(f), "{rate:?} {f} {tc}");
            }
        }
    }

    #[test]
    fn frames_and_samples_round_trip_at_a_frame_boundary() {
        for rate in FrameRate::ALL {
            for f in [0u64, 1, 24, 1000, 107_892] {
                let s = frames_to_samples(f, 48_000, rate);
                assert_eq!(samples_to_frames(s, 48_000, rate), f, "{rate:?} {f}");
            }
        }
    }

    #[test]
    fn bad_timecodes_are_refused() {
        assert_eq!(timecode_to_frames("1:2:3", FrameRate::F24), None);
        assert_eq!(timecode_to_frames("00:00:00:24", FrameRate::F24), None);
        assert_eq!(timecode_to_frames("00:01:00;00", FrameRate::F29_97Df), None);
        assert_eq!(
            timecode_to_frames("00:10:00;00", FrameRate::F29_97Df),
            Some(17_982)
        );
    }
}
