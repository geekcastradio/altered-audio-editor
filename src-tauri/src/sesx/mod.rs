// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Import of `.sesx` multitrack session files.
//!
//! Reads what a podcast edit needs to carry over: tracks (name, mono or
//! stereo, height, mute, solo, arm, volume, pan), clips (position,
//! source range, fades, lock) and the media files they use. Effects,
//! buses, sends and automation are not imported; the result says how
//! many effect racks were skipped so the user is told.
//!
//! This reads the user's own session files for interoperability. It
//! only parses the documented-by-example XML structure; nothing from the
//! other application is reused.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

use crate::codecs::probe;
use crate::session::model::{
    new_id, ChannelLayout, Clip, EffectChain, Fade, FadeShape, Id, MediaRef, Session, Track,
};

#[derive(Debug, Error)]
pub enum SesxError {
    #[error("Could not read the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("This is not a readable .sesx session: {0}")]
    Xml(String),
    #[error("This file has no multitrack session in it.")]
    NoSession,
}

/// What came across, for the notice shown after import.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub tracks: usize,
    pub clips: usize,
    /// Track, clip and master effects that were left out.
    pub skipped_effects: usize,
    /// Media files that could not be found. Their clips are still placed.
    pub missing_files: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct Imported {
    pub session: Session,
    pub report: ImportReport,
    /// Media ids not found on disk.
    pub missing: Vec<Id>,
}

struct FileEntry {
    absolute: Option<String>,
    relative: Option<String>,
}

fn attr<'a>(n: roxmltree::Node<'a, 'a>, name: &str) -> Option<&'a str> {
    n.attribute(name)
}

fn attr_u64(n: roxmltree::Node<'_, '_>, name: &str) -> Option<u64> {
    n.attribute(name)
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map(|v| v.max(0.0).round() as u64)
}

fn child<'a>(n: roxmltree::Node<'a, 'a>, tag: &str) -> Option<roxmltree::Node<'a, 'a>> {
    n.children().find(|c| c.has_tag_name(tag))
}

/// Value of `<parameter name="...">` inside the component with `componentID`.
fn component_param(
    track_params: roxmltree::Node<'_, '_>,
    component: &str,
    param: &str,
) -> Option<f64> {
    track_params
        .children()
        .filter(|c| c.has_tag_name("component") && c.attribute("componentID") == Some(component))
        .flat_map(|c| c.children().filter(|p| p.has_tag_name("parameter")))
        .find(|p| p.attribute("name") == Some(param))
        .and_then(|p| p.attribute("parameterValue"))
        .and_then(|v| v.parse::<f64>().ok())
}

/// Effects loaded in a rack: slots that hold a component.
fn count_effects(rack: roxmltree::Node<'_, '_>) -> usize {
    rack.children()
        .filter(|s| s.has_tag_name("slot") && s.descendants().any(|d| d.has_tag_name("component")))
        .count()
}

fn gain_to_db(gain: f64) -> f64 {
    if gain <= 0.000_001 {
        -96.0
    } else {
        ((20.0 * gain.log10()) * 10.0).round() / 10.0
    }
}

fn fade(node: Option<roxmltree::Node<'_, '_>>, fade_in: bool) -> Fade {
    let Some(n) = node else {
        return Fade::default();
    };
    let start = attr_u64(n, "startPoint").unwrap_or(0);
    let end = attr_u64(n, "endPoint").unwrap_or(0);
    let length = end.saturating_sub(start);
    let shape = match attr(n, "type") {
        Some("log") => FadeShape::Logarithmic,
        Some("cosine") => FadeShape::Cosine,
        _ => FadeShape::Linear,
    };
    let _ = fade_in;
    Fade {
        length,
        shape,
        curve: 0.0,
    }
}

/// Parses a `.sesx` file and builds a session. Media paths are tried
/// relative to the `.sesx` file first, then as stored.
pub fn import(path: &Path) -> Result<Imported, SesxError> {
    let text = std::fs::read_to_string(path)?;
    // Some files start with a byte order mark.
    let text = text.trim_start_matches('\u{feff}');
    let doc = roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )
    .map_err(|e| SesxError::Xml(e.to_string()))?;
    let root = doc.root_element();
    let session_node = root
        .descendants()
        .find(|n| n.has_tag_name("session"))
        .ok_or(SesxError::NoSession)?;

    let sample_rate = attr_u64(session_node, "sampleRate")
        .unwrap_or(48_000)
        .clamp(8_000, 384_000) as u32;
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Imported Session".into());
    let mut session = Session::new(name, sample_rate);
    let mut report = ImportReport::default();

    // Files.
    let mut files: HashMap<String, FileEntry> = HashMap::new();
    if let Some(list) = root.descendants().find(|n| n.has_tag_name("files")) {
        for f in list.children().filter(|c| c.has_tag_name("file")) {
            if let Some(id) = f.attribute("id") {
                files.insert(
                    id.to_string(),
                    FileEntry {
                        absolute: f.attribute("absolutePath").map(str::to_string),
                        relative: f.attribute("relativePath").map(str::to_string),
                    },
                );
            }
        }
    }

    let base = path.parent().unwrap_or(Path::new(""));
    // File id in the .sesx to our media id, created on first use.
    let mut media_ids: HashMap<String, Id> = HashMap::new();
    let mut missing = Vec::new();

    let tracks_node = child(session_node, "tracks");
    for t in tracks_node
        .iter()
        .flat_map(|n| n.children())
        .filter(|n| n.has_tag_name("audioTrack"))
    {
        let params = child(t, "trackParameters");
        let audio = child(t, "trackAudioParameters");
        let track_name = params
            .and_then(|p| child(p, "name"))
            .and_then(|n| n.text())
            .map(str::to_string)
            .unwrap_or_else(|| format!("Track {}", session.tracks.len() + 1));
        let stereo = audio.and_then(|a| a.attribute("audioChannelType")) != Some("mono");
        let mut track = Track::new(
            track_name,
            if stereo {
                ChannelLayout::Stereo
            } else {
                ChannelLayout::Mono
            },
        );
        if let Some(h) = params.and_then(|p| attr_u64(p, "trackHeight")) {
            track.height = (h as u32).clamp(40, 600);
        }
        if let Some(a) = audio {
            track.solo = a.attribute("solo") == Some("true");
            track.record_armed = a.attribute("recordArmed") == Some("true");
            track.muted = component_param(a, "Audition.Mute", "mute").is_some_and(|v| v >= 0.5);
            if let Some(g) = component_param(a, "Audition.Fader", "volume") {
                track.volume_db = gain_to_db(g);
            }
            if let Some(p) = component_param(a, "Audition.StereoPanner", "Pan") {
                let p = if p.abs() <= 1.0 { p * 100.0 } else { p };
                track.pan = p.clamp(-100.0, 100.0).round();
            }
        }
        // Track effects (clip effects are counted with their clips).
        report.skipped_effects += t
            .children()
            .filter(|n| n.has_tag_name("effectsRack"))
            .map(count_effects)
            .sum::<usize>();

        for c in t.children().filter(|n| n.has_tag_name("audioClip")) {
            let Some(file_key) = c.attribute("fileID") else {
                continue;
            };
            let source_in = attr_u64(c, "sourceInPoint").unwrap_or(0);
            let source_out = attr_u64(c, "sourceOutPoint").unwrap_or(0);
            if source_out <= source_in {
                continue;
            }
            let media_id = match media_ids.get(file_key) {
                Some(id) => *id,
                None => {
                    let entry = files.get(file_key);
                    let (media, found) =
                        resolve_media(entry, base, sample_rate, stereo, source_out);
                    if !found {
                        missing.push(media.id);
                        report.missing_files.push(media.path.clone());
                    }
                    let id = media.id;
                    session.media.push(media);
                    media_ids.insert(file_key.to_string(), id);
                    id
                }
            };
            // Make sure the stored length covers every clip that uses it.
            if let Some(m) = session.media.iter_mut().find(|m| m.id == media_id) {
                if missing.contains(&media_id) {
                    m.length = m.length.max(source_out);
                }
            }
            report.skipped_effects += c
                .descendants()
                .filter(|n| n.has_tag_name("effectsRack"))
                .map(count_effects)
                .sum::<usize>();
            track.clips.push(Clip {
                id: new_id(),
                name: c.attribute("name").unwrap_or("Clip").to_string(),
                media_id,
                start: attr_u64(c, "startPoint").unwrap_or(0),
                source_in,
                source_out,
                gain_db: 0.0,
                muted: false,
                locked: c.attribute("lockedInTime") == Some("true"),
                color: None,
                fade_in: fade(child(c, "fadeIn"), true),
                fade_out: fade(child(c, "fadeOut"), false),
                envelopes: Vec::new(),
                effects: EffectChain::default(),
                group: None,
                looped: None,
                stretch: None,
                audio_type: None,
            });
            report.clips += 1;
        }
        session.tracks.push(track);
        report.tracks += 1;
    }

    // Master track effects count as skipped too.
    if let Some(master) = tracks_node
        .iter()
        .flat_map(|n| n.children())
        .find(|n| n.has_tag_name("masterTrack"))
    {
        report.skipped_effects += master
            .descendants()
            .filter(|n| n.has_tag_name("effectsRack"))
            .map(count_effects)
            .sum::<usize>();
    }

    Ok(Imported {
        session,
        report,
        missing,
    })
}

/// Finds a media file and measures it. Missing files get a placeholder
/// built from the session so their clips still show where they were.
fn resolve_media(
    entry: Option<&FileEntry>,
    base: &Path,
    session_rate: u32,
    stereo: bool,
    min_length: u64,
) -> (MediaRef, bool) {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(e) = entry {
        if let Some(r) = &e.relative {
            candidates.push(base.join(r.replace('\\', "/")));
        }
        if let Some(a) = &e.absolute {
            candidates.push(PathBuf::from(a));
            // A Windows path read on another system: try its file name
            // next to the session as a last resort.
            if let Some(name) = a.rsplit(['\\', '/']).next() {
                candidates.push(base.join(name));
            }
        }
    }
    for p in &candidates {
        if p.is_file() {
            if let Ok(info) = probe(p) {
                return (
                    MediaRef {
                        id: new_id(),
                        path: p.clone(),
                        relative_path: None,
                        sample_rate: info.sample_rate,
                        channels: info.channels,
                        length: info.length,
                    },
                    true,
                );
            }
        }
    }
    let stored = entry
        .and_then(|e| e.absolute.clone().or_else(|| e.relative.clone()))
        .unwrap_or_default();
    (
        MediaRef {
            id: new_id(),
            path: PathBuf::from(stored),
            relative_path: None,
            sample_rate: session_rate,
            channels: if stereo { 2 } else { 1 },
            length: min_length,
        },
        false,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<!DOCTYPE sesx>
<sesx version="1.9">
  <session audioChannelType="stereo" bitDepth="24" duration="400000" sampleRate="44100">
    <tracks>
      <audioTrack id="10001" index="1">
        <trackParameters trackHeight="117">
          <name>HOST</name>
        </trackParameters>
        <trackAudioParameters audioChannelType="mono" recordArmed="false" solo="true">
          <component componentID="Audition.Fader" id="trackFader" name="volume" powered="true">
            <parameter index="0" name="volume" parameterValue="1.58489322662353515625"/>
          </component>
          <component componentID="Audition.Mute" id="trackMute" name="Mute" powered="true">
            <parameter index="1" name="mute" parameterValue="1"/>
          </component>
          <component componentID="Audition.StereoPanner" id="trackPan" name="StereoPanner" powered="true">
            <parameter index="0" name="Pan" parameterValue="-0.5"/>
          </component>
        </trackAudioParameters>
        <effectsRack preFader="true">
          <slot><component componentID="Noise.Thing" name="Noise"/></slot>
          <slot/>
          <slot><component componentID="Level.Thing" name="Level"/></slot>
        </effectsRack>
        <audioClip fileID="0" id="0" name="host take" sourceInPoint="1000" sourceOutPoint="23050" startPoint="44100" endPoint="66150" lockedInTime="true">
          <fadeIn startPoint="0" endPoint="4410" type="log"/>
          <fadeOut startPoint="17640" endPoint="22050" type="linear"/>
        </audioClip>
        <audioClip fileID="0" id="1" name="host take 2" sourceInPoint="0" sourceOutPoint="0" startPoint="0" endPoint="0"/>
      </audioTrack>
      <audioTrack id="10002" index="2">
        <trackParameters><name>MISSING</name></trackParameters>
        <trackAudioParameters audioChannelType="stereo"/>
        <audioClip fileID="1" id="0" name="gone" sourceInPoint="0" sourceOutPoint="88200" startPoint="0" endPoint="88200"/>
      </audioTrack>
      <masterTrack id="10000"><effectsRack><slot><component name="Limit"/></slot></effectsRack></masterTrack>
    </tracks>
  </session>
  <files>
    <file id="0" absolutePath="Z:\nowhere\host.wav" relativePath="Raw/host.wav"/>
    <file id="1" absolutePath="Z:\nowhere\gone.wav"/>
  </files>
</sesx>"#;

    #[test]
    fn imports_tracks_clips_and_finds_media_by_relative_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("Raw")).unwrap();
        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/media/mono_44k_24.wav");
        std::fs::copy(&src, dir.path().join("Raw/host.wav")).unwrap();
        let sesx = dir.path().join("EP1.sesx");
        // Byte order mark, as found in real .sesx files.
        std::fs::write(&sesx, format!("\u{feff}{SAMPLE}")).unwrap();

        let out = import(&sesx).unwrap();
        let s = &out.session;
        assert_eq!(s.name, "EP1");
        assert_eq!(s.sample_rate, 44_100);
        assert_eq!(out.report.tracks, 2);
        assert_eq!(out.report.clips, 2, "the empty clip is skipped");
        assert_eq!(
            out.report.skipped_effects, 3,
            "two track effects and one master effect"
        );

        let host = &s.tracks[0];
        assert_eq!(host.name, "HOST");
        assert_eq!(host.channels, ChannelLayout::Mono);
        assert_eq!(host.height, 117);
        assert!(host.solo && host.muted);
        assert_eq!(host.volume_db, 4.0);
        assert_eq!(host.pan, -50.0);
        let clip = &host.clips[0];
        assert_eq!(
            (clip.start, clip.source_in, clip.source_out),
            (44_100, 1000, 23_050)
        );
        assert!(clip.locked);
        assert_eq!(clip.fade_in.length, 4410);
        assert_eq!(clip.fade_in.shape, FadeShape::Logarithmic);
        assert_eq!(clip.fade_out.length, 4410);

        let media = s.media_by_id(clip.media_id).unwrap();
        assert_eq!(media.path, dir.path().join("Raw/host.wav"));
        assert_eq!(
            (media.sample_rate, media.channels, media.length),
            (44_100, 1, 22_050)
        );

        // The missing file still gets a placeholder so its clip shows.
        assert_eq!(out.missing.len(), 1);
        assert_eq!(
            out.report.missing_files,
            vec![PathBuf::from("Z:\\nowhere\\gone.wav")]
        );
        let gone = s.media_by_id(s.tracks[1].clips[0].media_id).unwrap();
        assert_eq!(gone.length, 88_200);
        assert!(
            crate::session::validate::validate(s).is_empty(),
            "{:?}",
            crate::session::validate::validate(s)
        );
    }

    #[test]
    fn rejects_files_that_are_not_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.sesx");
        std::fs::write(&p, "<notsesx/>").unwrap();
        assert!(matches!(import(&p), Err(SesxError::NoSession)));
        std::fs::write(&p, "not xml at all").unwrap();
        assert!(matches!(import(&p), Err(SesxError::Xml(_))));
    }

    #[test]
    fn converts_gain_to_db() {
        assert_eq!(gain_to_db(1.0), 0.0);
        assert_eq!(gain_to_db(0.5), -6.0);
        assert_eq!(gain_to_db(0.0), -96.0);
    }
}
