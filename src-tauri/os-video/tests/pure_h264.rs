// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The built-in H.264 decoder, through the public reader. The clip is six
//! seconds of ffmpeg's test pattern, 25 frames a second, a key picture every
//! second and B-frames, so seeking and reordering are both exercised.
#![cfg(feature = "pure-video")]

use altered_os_video::{set_prefer_built_in, VideoReader};
use std::path::PathBuf;

fn clip() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/h264-testsrc.mp4")
}

#[test]
fn reports_what_the_file_holds() {
    set_prefer_built_in(true);
    let r = VideoReader::open(&clip()).unwrap();
    let i = r.info();
    assert_eq!((i.width, i.height), (320, 240));
    assert!((i.fps - 25.0).abs() < 0.5, "{}", i.fps);
    assert!((i.duration - 6.0).abs() < 0.1, "{}", i.duration);
    assert_eq!(i.codec, "H.264");
}

#[test]
fn frames_come_out_in_order_and_cover_the_clip() {
    set_prefer_built_in(true);
    let r = VideoReader::open(&clip()).unwrap();
    let mut last = -1.0f64;
    let mut n = 0;
    while let Some(f) = r.next_frame(320).unwrap() {
        assert!(f.time > last, "{} after {}", f.time, last);
        last = f.time;
        n += 1;
    }
    assert!((148..=151).contains(&n), "{n}");
}

#[test]
fn a_seek_lands_on_the_right_picture() {
    set_prefer_built_in(true);
    let r = VideoReader::open(&clip()).unwrap();
    // Stepping through gives the reference picture for each time.
    let mut all = Vec::new();
    while let Some(f) = r.next_frame(320).unwrap() {
        all.push(f);
    }
    for t in [0.0, 1.0, 2.5, 4.2, 5.9] {
        let want = all.iter().rfind(|f| f.time <= t + 1e-6).unwrap();
        let got = r.frame_at(t, 320).unwrap();
        assert!(
            (got.time - want.time).abs() < 1e-6,
            "t={t}: {} vs {}",
            got.time,
            want.time
        );
        assert_eq!(got.rgba, want.rgba, "t={t}");
    }
}

#[test]
fn a_test_pattern_has_color() {
    set_prefer_built_in(true);
    let r = VideoReader::open(&clip()).unwrap();
    let f = r.frame_at(0.0, 320).unwrap();
    let distinct: std::collections::HashSet<[u8; 3]> =
        f.rgba.chunks(4).map(|p| [p[0], p[1], p[2]]).collect();
    assert!(distinct.len() > 8, "{}", distinct.len());
}
