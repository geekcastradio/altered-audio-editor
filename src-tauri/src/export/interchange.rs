// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! What the interchange writers (EDL, OTIO) need from a session, as plain values. The command
//! that calls them builds these from the session model, so the writers stay easy to test.

/// One clip on one track, positions in session samples.
#[derive(Debug, Clone, PartialEq)]
pub struct InClip {
    /// Zero-based index of the audio track the clip sits on.
    pub track: usize,
    pub name: String,
    /// Path of the media file, as the session stores it.
    pub path: String,
    /// Where the clip starts on the timeline.
    pub start: u64,
    /// Where in the media file the clip starts.
    pub source_start: u64,
    /// How long the clip plays on the timeline.
    pub length: u64,
    /// Length of the whole media file, when known.
    pub media_length: Option<u64>,
    /// 1.0 for an unstretched clip. Anything else is reported, not carried.
    pub stretch: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InMarker {
    pub name: String,
    pub start: u64,
    /// Zero for a point marker.
    pub length: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InSession {
    pub title: String,
    pub sample_rate: u32,
    pub track_names: Vec<String>,
    pub clips: Vec<InClip>,
    pub markers: Vec<InMarker>,
}
