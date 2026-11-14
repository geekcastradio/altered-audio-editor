// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Starter session templates: a few ready-made track layouts for the shows people make most. They
//! are written into the templates folder once, as ordinary templates, so they can be renamed,
//! changed or deleted like any other. A marker file records that they were written, so a template
//! the user deletes stays deleted.

use std::path::Path;

use crate::commands::template::{save_in, TemplateEntry};
use crate::session::model::{ChannelLayout, MixRole, Session, Track};

/// Written into the templates folder once the starters are there.
const MARKER: &str = ".starters-seeded";

struct Starter {
    name: &'static str,
    description: &'static str,
    session: Session,
}

fn track(name: &str, channels: ChannelLayout, role: MixRole, color: &str) -> Track {
    let mut t = Track::new(name, channels);
    t.mix_role = role;
    t.color = Some(color.to_string());
    t
}

fn starters() -> Vec<Starter> {
    let mut two_host = Session::new("Two-Host Show", 48_000);
    two_host.bit_depth = 24;
    two_host.loudness_target = Some("podcast-stereo".to_string());
    two_host.tracks = vec![
        track("Host", ChannelLayout::Mono, MixRole::Foreground, "#4f8fd9"),
        track("Guest", ChannelLayout::Mono, MixRole::Foreground, "#d9884f"),
        track("Music", ChannelLayout::Stereo, MixRole::Ducked, "#8f6fd0"),
        track(
            "Stingers and Effects",
            ChannelLayout::Stereo,
            MixRole::Normal,
            "#58b36a",
        ),
    ];

    let mut solo = Session::new("Solo Voiceover", 48_000);
    solo.bit_depth = 24;
    solo.tracks = vec![
        track("Voice", ChannelLayout::Mono, MixRole::Foreground, "#4f8fd9"),
        track(
            "Room Tone",
            ChannelLayout::Mono,
            MixRole::Background,
            "#7a8a99",
        ),
        track("Music", ChannelLayout::Stereo, MixRole::Ducked, "#8f6fd0"),
    ];

    let mut book = Session::new("Audiobook", 44_100);
    book.bit_depth = 24;
    book.tracks = vec![
        track(
            "Narration",
            ChannelLayout::Mono,
            MixRole::Foreground,
            "#4f8fd9",
        ),
        track(
            "Room Tone",
            ChannelLayout::Mono,
            MixRole::Background,
            "#7a8a99",
        ),
    ];

    vec![
        Starter {
            name: "Two-Host Show",
            description: "Host and guest on their own tracks, music that ducks under them, and a track for stingers. 48 kHz, 24-bit. No effects: add your Voice Chain, then Save as Session Template to make it yours.",
            session: two_host,
        },
        Starter {
            name: "Solo Voiceover",
            description: "One voice, a room tone track for patching gaps, and music that ducks. 48 kHz, 24-bit.",
            session: solo,
        },
        Starter {
            name: "Audiobook",
            description: "Narration and a room tone track, at 44.1 kHz, the rate audiobook retailers ask for. 24-bit while you work.",
            session: book,
        },
    ]
}

/// Writes the starter templates into `dir` the first time it is called for that folder. Returns
/// the ones written, none on later calls. Failures leave the folder as it was and are not an error:
/// the starters are a convenience.
pub fn seed_in(dir: &Path) -> Vec<TemplateEntry> {
    if dir.join(MARKER).exists() {
        return Vec::new();
    }
    if std::fs::create_dir_all(dir).is_err() {
        return Vec::new();
    }
    let mut written = Vec::new();
    for s in starters() {
        if let Ok(e) = save_in(dir, s.name, s.description, &s.session) {
            written.push(e);
        }
    }
    if !written.is_empty() {
        let _ = std::fs::write(dir.join(MARKER), b"1");
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::template::{list_in, session_from};

    #[test]
    fn written_once_and_deleting_one_keeps_it_deleted() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(seed_in(dir.path()).len(), 3);
        let list = list_in(dir.path());
        assert_eq!(list.len(), 3);
        assert!(seed_in(dir.path()).is_empty());

        std::fs::remove_file(&list[0].path).unwrap();
        assert!(seed_in(dir.path()).is_empty());
        assert_eq!(list_in(dir.path()).len(), 2);
    }

    #[test]
    fn two_host_template_starts_a_session_with_its_tracks_and_rate() {
        let dir = tempfile::tempdir().unwrap();
        seed_in(dir.path());
        let entry = list_in(dir.path())
            .into_iter()
            .find(|e| e.name == "Two-Host Show")
            .unwrap();
        assert_eq!(entry.tracks, 4);
        assert_eq!(entry.sample_rate, 48_000);
        let s = session_from(&entry.path, "Episode 1").unwrap();
        assert_eq!(s.name, "Episode 1");
        assert_eq!(s.tracks[2].mix_role, MixRole::Ducked);
        assert_eq!(s.tracks[0].channels, ChannelLayout::Mono);
    }

    #[test]
    fn audiobook_runs_at_44100() {
        let dir = tempfile::tempdir().unwrap();
        seed_in(dir.path());
        let e = list_in(dir.path())
            .into_iter()
            .find(|e| e.name == "Audiobook")
            .unwrap();
        assert_eq!(e.sample_rate, 44_100);
    }
}
