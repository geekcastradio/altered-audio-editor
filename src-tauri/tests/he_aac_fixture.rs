// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! An HE-AAC file stores a half-rate core plus a band replication layer the
//! decoder does not apply. The file must still import at the right speed:
//! the core rate, the true length and a label that says the highs are missing.
//! The fixture is a 5 second clip a user sent with issue 321.

use altered_audio_editor_lib::codecs::{probe, AudioReader};
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/he-aac-48k.m4a")
}

#[test]
fn he_aac_imports_at_the_core_rate_with_the_true_length() {
    let info = probe(&fixture()).unwrap();
    assert_eq!(info.sample_rate, 24_000);
    assert!((info.duration_seconds() - 4.99).abs() < 0.05, "{info:?}");
    assert!(info.codec.contains("without high band"), "{}", info.codec);
}

#[test]
fn he_aac_reads_the_whole_length_at_that_rate() {
    let mut r = AudioReader::open(&fixture(), 2, None).unwrap();
    assert_eq!(r.source_rate(), 24_000);
    let mut buf = Vec::new();
    let mut frames = 0usize;
    while r.read_into(&mut buf).unwrap() {
        frames += buf.len() / 2;
        buf.clear();
    }
    let seconds = frames as f64 / 24_000.0;
    assert!((seconds - 4.99).abs() < 0.1, "{seconds}");
}
