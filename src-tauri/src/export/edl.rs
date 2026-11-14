// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! CMX3600 edit decision lists, and a marker list in the form DaVinci Resolve reads.
//!
//! An EDL has no fades, volume, pan or effects and places clips on whole frames, so what it
//! carries is clip names, where each clip sits and which part of its file plays. The writer
//! returns warnings for everything it had to round or leave out, so the dialog can say so.

use crate::export::interchange::{InClip, InMarker, InSession};
use crate::export::timecode::{frames_to_timecode, samples_to_frames, FrameRate};

#[derive(Debug, Clone)]
pub struct EdlOptions {
    pub rate: FrameRate,
    /// Timecode of the session start, in frames (3,600 x fps for one hour).
    pub start_frames: u64,
}

impl EdlOptions {
    pub fn new(rate: FrameRate) -> Self {
        Self {
            rate,
            start_frames: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdlFile {
    /// Which track this file holds.
    pub track: usize,
    pub text: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Report {
    pub warnings: Vec<String>,
}

fn fcm(rate: FrameRate) -> &'static str {
    if rate.is_drop() {
        "DROP FRAME"
    } else {
        "NON-DROP FRAME"
    }
}

fn clean_title(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}

/// One EDL per track that has clips, in track order. Each uses source `AX` on audio channel A,
/// the form every editor reads; one file per track avoids the four-channel limit of the format.
pub fn edls(session: &InSession, opts: &EdlOptions) -> (Vec<EdlFile>, Report) {
    let mut report = Report::default();
    let mut tracks: Vec<usize> = session.clips.iter().map(|c| c.track).collect();
    tracks.sort_unstable();
    tracks.dedup();
    let rate = session.sample_rate;
    let mut files = Vec::new();
    for track in tracks {
        let mut clips: Vec<&InClip> = session.clips.iter().filter(|c| c.track == track).collect();
        clips.sort_by_key(|c| (c.start, c.source_start));
        let label = session
            .track_names
            .get(track)
            .cloned()
            .unwrap_or_else(|| format!("Track {}", track + 1));
        let mut text = format!(
            "TITLE: {} - {}\nFCM: {}\n\n",
            clean_title(&session.title),
            clean_title(&label),
            fcm(opts.rate)
        );
        let mut event = 0usize;
        let mut prev_end: u64 = 0;
        for clip in clips {
            if clip.length == 0 {
                continue;
            }
            if (clip.stretch - 1.0).abs() > 1e-9 {
                report.warnings.push(format!(
                    "{}: stretched clips are written at their timeline length without the stretch",
                    clip.name
                ));
            }
            let rec_in = samples_to_frames(clip.start, rate, opts.rate);
            let rec_out = samples_to_frames(clip.start + clip.length, rate, opts.rate);
            let src_in = samples_to_frames(clip.source_start, rate, opts.rate);
            let span = rec_out.saturating_sub(rec_in);
            if span == 0 {
                report
                    .warnings
                    .push(format!("{}: shorter than one frame, left out", clip.name));
                continue;
            }
            if rec_in < prev_end {
                report.warnings.push(format!(
                    "{}: overlaps the clip before it on {}, an EDL cannot hold overlaps on one track",
                    clip.name, label
                ));
            }
            prev_end = prev_end.max(rec_out);
            event += 1;
            let tc = |f: u64| frames_to_timecode(f + opts.start_frames, opts.rate);
            // Source timecode is the position in the file; record timecode is the session position.
            let sc = |f: u64| frames_to_timecode(f, opts.rate);
            text.push_str(&format!(
                "{event:03}  AX       A     C        {} {} {} {}\n",
                sc(src_in),
                sc(src_in + span),
                tc(rec_in),
                tc(rec_out)
            ));
            text.push_str(&format!("* FROM CLIP NAME: {}\n", clean_title(&clip.name)));
            if !clip.path.is_empty() {
                text.push_str(&format!("* SOURCE FILE: {}\n", clean_title(&clip.path)));
            }
            text.push('\n');
        }
        if event > 0 {
            files.push(EdlFile { track, text });
        }
    }
    if files.is_empty() {
        report.warnings.push("There are no clips to write.".into());
    }
    report.warnings.push(format!(
        "Positions are rounded to whole frames at {} fps.",
        opts.rate.label()
    ));
    (files, report)
}

/// Resolve marker colors by name. Anything else falls back to Blue.
pub const RESOLVE_COLORS: [&str; 8] = [
    "Blue", "Cyan", "Green", "Yellow", "Red", "Pink", "Purple", "Fuchsia",
];

/// Markers as an EDL that DaVinci Resolve imports as timeline markers: one event per marker with a
/// `|C:ResolveColor... |M:name |D:frames` comment. A range marker keeps its length.
/// The shape follows Resolve's own marker export; check an import in Resolve before relying on it.
pub fn marker_edl(session: &InSession, opts: &EdlOptions, color: &str) -> (String, Report) {
    let mut report = Report::default();
    let color = RESOLVE_COLORS
        .iter()
        .find(|c| c.eq_ignore_ascii_case(color))
        .copied()
        .unwrap_or("Blue");
    let mut markers: Vec<&InMarker> = session.markers.iter().collect();
    markers.sort_by_key(|m| m.start);
    let mut text = format!(
        "TITLE: {} markers\nFCM: {}\n\n",
        clean_title(&session.title),
        fcm(opts.rate)
    );
    for (n, m) in markers.iter().enumerate() {
        let a = samples_to_frames(m.start, session.sample_rate, opts.rate) + opts.start_frames;
        let dur = samples_to_frames(m.length, session.sample_rate, opts.rate).max(1);
        let tc_in = frames_to_timecode(a, opts.rate);
        let tc_out = frames_to_timecode(a + 1, opts.rate);
        text.push_str(&format!(
            "{:03}  001      V     C        {tc_in} {tc_out} {tc_in} {tc_out}\n",
            n + 1
        ));
        // The pipe is the field separator, so it cannot appear in a name.
        let name: String = clean_title(&m.name).replace('|', "/");
        text.push_str(&format!(" |C:ResolveColor{color} |M:{name} |D:{dur}\n\n"));
    }
    if session.markers.is_empty() {
        report
            .warnings
            .push("There are no markers to write.".into());
    }
    (text, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(track: usize, name: &str, start: u64, src: u64, len: u64) -> InClip {
        InClip {
            track,
            name: name.into(),
            path: format!("C:/audio/{name}.wav"),
            start,
            source_start: src,
            length: len,
            media_length: None,
            stretch: 1.0,
        }
    }

    fn session(clips: Vec<InClip>) -> InSession {
        InSession {
            title: "Episode 1".into(),
            sample_rate: 48_000,
            track_names: vec!["Host".into(), "Guest".into()],
            clips,
            markers: vec![],
        }
    }

    #[test]
    fn one_file_per_track_with_events_in_order() {
        let s = session(vec![
            clip(1, "guest1", 96_000, 0, 48_000),
            clip(0, "host2", 48_000, 24_000, 48_000),
            clip(0, "host1", 0, 0, 48_000),
        ]);
        let mut opts = EdlOptions::new(FrameRate::F24);
        opts.start_frames = 24 * 3600;
        let (files, rep) = edls(&s, &opts);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].track, 0);
        let host = &files[0].text;
        assert!(
            host.starts_with("TITLE: Episode 1 - Host\nFCM: NON-DROP FRAME\n"),
            "{host}"
        );
        let lines: Vec<&str> = host.lines().filter(|l| l.starts_with("00")).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0],
            "001  AX       A     C        00:00:00:00 00:00:01:00 01:00:00:00 01:00:01:00"
        );
        // The second clip starts half a second into its file and one second into the session.
        assert_eq!(
            lines[1],
            "002  AX       A     C        00:00:00:12 00:00:01:12 01:00:01:00 01:00:02:00"
        );
        assert!(host.contains("* FROM CLIP NAME: host1"));
        assert!(rep.warnings.iter().any(|w| w.contains("whole frames")));
    }

    #[test]
    fn drop_frame_sessions_say_so_and_use_semicolons() {
        let s = session(vec![clip(0, "a", 0, 0, 48_000 * 61)]);
        let (files, _) = edls(&s, &EdlOptions::new(FrameRate::F29_97Df));
        let t = &files[0].text;
        assert!(t.contains("FCM: DROP FRAME"));
        assert!(t.contains(';'), "{t}");
    }

    #[test]
    fn overlaps_stretch_and_tiny_clips_are_reported() {
        let mut stretched = clip(0, "slow", 200_000, 0, 48_000);
        stretched.stretch = 1.25;
        let s = session(vec![
            clip(0, "a", 0, 0, 96_000),
            clip(0, "b", 48_000, 0, 96_000),
            stretched,
            clip(0, "tiny", 400_000, 0, 100),
        ]);
        let (_, rep) = edls(&s, &EdlOptions::new(FrameRate::F25));
        let joined = rep.warnings.join("\n");
        assert!(joined.contains("b: overlaps"), "{joined}");
        assert!(joined.contains("slow: stretched"), "{joined}");
        assert!(joined.contains("tiny: shorter than one frame"), "{joined}");
    }

    #[test]
    fn an_empty_session_says_there_is_nothing_to_write() {
        let (files, rep) = edls(&session(vec![]), &EdlOptions::new(FrameRate::F24));
        assert!(files.is_empty());
        assert!(rep.warnings.iter().any(|w| w.contains("no clips")));
    }

    #[test]
    fn markers_become_resolve_comments() {
        let mut s = session(vec![]);
        s.markers = vec![
            InMarker {
                name: "Intro | start".into(),
                start: 48_000,
                length: 0,
            },
            InMarker {
                name: "Ad".into(),
                start: 0,
                length: 96_000,
            },
        ];
        let (text, rep) = marker_edl(&s, &EdlOptions::new(FrameRate::F24), "green");
        assert!(rep.warnings.is_empty());
        let first = text.lines().position(|l| l.starts_with("001")).unwrap();
        assert!(text
            .lines()
            .nth(first + 1)
            .unwrap()
            .contains("|C:ResolveColorGreen |M:Ad |D:48"));
        assert!(text.contains("|M:Intro / start |D:1"));
        let (_, rep) = marker_edl(&session(vec![]), &EdlOptions::new(FrameRate::F24), "x");
        assert_eq!(rep.warnings.len(), 1);
    }

    #[test]
    fn an_unknown_color_falls_back_to_blue() {
        let mut s = session(vec![]);
        s.markers = vec![InMarker {
            name: "x".into(),
            start: 0,
            length: 0,
        }];
        let (text, _) = marker_edl(&s, &EdlOptions::new(FrameRate::F24), "chartreuse");
        assert!(text.contains("ResolveColorBlue"));
    }
}
