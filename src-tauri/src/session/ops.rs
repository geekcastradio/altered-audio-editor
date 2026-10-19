// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Edits to the open session.
//!
//! The core owns the session. The interface sends an `Op`; the core
//! applies it and returns the op that reverses it, which the interface
//! keeps on its undo stack. Ids for new things come from the interface,
//! so redo recreates exactly the same tracks and clips.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::model::{
    AudioType, ChannelLayout, Clip, ClipLoop, ClipStretch, EffectChain, Envelope, EnvelopePoint,
    EnvelopeTarget, Fade, Id, Marker, MarkerKind, MediaRef, Session, SessionView, Track, TrackRole,
    STRETCH_MAX, STRETCH_MIN,
};

/// A clip to create.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClipSpec {
    pub id: Id,
    pub name: String,
    pub media_id: Id,
    pub start: u64,
    pub source_in: u64,
    pub source_out: u64,
}

impl ClipSpec {
    fn build(&self) -> Clip {
        Clip {
            id: self.id,
            name: self.name.clone(),
            media_id: self.media_id,
            start: self.start,
            source_in: self.source_in,
            source_out: self.source_out,
            gain_db: 0.0,
            muted: false,
            locked: false,
            color: None,
            fade_in: Fade::default(),
            fade_out: Fade::default(),
            envelopes: Vec::new(),
            effects: EffectChain::default(),
            group: None,
            looped: None,
            stretch: None,
            audio_type: None,
        }
    }
}

/// Track settings to change. Fields left out stay as they are.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrackChanges {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solo: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record_armed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_db: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polarity_inverted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collapsed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor_input: Option<bool>,
    /// Hardware input; `Some(None)` clears it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<Option<String>>,
    /// Standard or remote guest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<TrackRole>,
    /// The host profile applied; an empty id clears it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_profile_id: Option<String>,
}

/// Changes to one clip. Fields left out are unchanged. `track_id` moves
/// the clip to another track.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClipChange {
    pub id: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub track_id: Option<Id>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_in: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_out: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gain_db: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_in: Option<Fade>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fade_out: Option<Fade>,
    /// `#rrggbb`, or an empty string for the track color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Puts the clip in this group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<Id>,
    /// Takes the clip out of its group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ungroup: Option<bool>,
    /// Replaces the clip's volume envelope points (values in dB,
    /// positions relative to the clip start). Empty removes the envelope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<Vec<EnvelopePoint>>,
    /// Replaces the clip's pan envelope points (-100 is left, 100 is
    /// right). Empty removes the envelope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pan: Option<Vec<EnvelopePoint>>,
    /// Loops the clip, or changes its loop length or phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub looped: Option<ClipLoop>,
    /// Stops looping: the clip plays its source region once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unloop: Option<bool>,
    /// Stretches the clip, or changes its stretch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stretch: Option<ClipStretch>,
    /// Stops stretching: the clip plays at its natural length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unstretch: Option<bool>,
    /// Sets what the clip holds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_type: Option<AudioType>,
    /// Clears the clip's audio type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clear_audio_type: Option<bool>,
}

/// Changes to one marker. Fields left out are unchanged.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MarkerChange {
    pub id: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<MarkerKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// `#rrggbb`, or an empty string for the type's color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Ad slots: the contracted length of the read, in session samples.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contracted_duration: Option<u64>,
    /// Removes the contracted length (when a marker stops being an ad slot).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clear_contracted: Option<bool>,
}

/// Whose Voice Chain an edit changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum EffectTarget {
    Track(Id),
    Clip(Id),
    Master,
}

/// One edit. See the module docs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    /// Replaces a track's, clip's or the master's whole Voice Chain.
    SetEffects {
        target: EffectTarget,
        chain: EffectChain,
    },
    AddTrack {
        id: Id,
        name: String,
        stereo: bool,
        index: usize,
    },
    /// Puts back a removed track exactly as it was.
    RestoreTrack {
        track: Box<Track>,
        index: usize,
    },
    RemoveTrack {
        id: Id,
    },
    MoveTrack {
        id: Id,
        to: usize,
    },
    SetTrack {
        id: Id,
        changes: TrackChanges,
    },
    AddMedia {
        media: MediaRef,
    },
    RemoveMedia {
        id: Id,
    },
    AddClips {
        track_id: Id,
        clips: Vec<ClipSpec>,
    },
    /// Puts back removed clips exactly as they were.
    RestoreClips {
        clips: Vec<(Id, Clip)>,
    },
    RemoveClips {
        ids: Vec<Id>,
    },
    /// Moves, trims, fades and other changes to existing clips.
    SetClips {
        changes: Vec<ClipChange>,
    },
    /// Adds markers, or puts removed ones back.
    AddMarkers {
        markers: Vec<Marker>,
    },
    RemoveMarkers {
        ids: Vec<Id>,
    },
    SetMarkers {
        changes: Vec<MarkerChange>,
    },
    /// Replaces the Playlist panel's order.
    SetPlaylist {
        ids: Vec<Id>,
    },
    /// Changes the session rate. Every timeline position (clips, fades,
    /// envelopes, markers, view) is converted so it stays at the same
    /// time. `exact` is filled in by the inverse so undo restores the
    /// original sample values with no rounding drift.
    SetSampleRate {
        sample_rate: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        exact: Option<Vec<u64>>,
    },
    /// Session name and bit depth. Fields left out are unchanged.
    SetSessionInfo {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bit_depth: Option<u16>,
    },
    /// Master output level in dB.
    SetMasterVolume {
        volume_db: f64,
    },
    /// Several ops as one step. Applied in order, reversed in reverse.
    Batch {
        ops: Vec<Op>,
    },
}

#[derive(Debug, Error, PartialEq)]
pub enum OpError {
    #[error("No track with id {0}.")]
    NoTrack(Id),
    #[error("No clip with id {0}.")]
    NoClip(Id),
    #[error("No media file with id {0}.")]
    NoMedia(Id),
    #[error("Something with id {0} already exists.")]
    DuplicateId(Id),
    #[error("The media file is still used by a clip.")]
    MediaInUse,
    #[error("A clip must play at least one sample.")]
    EmptyClip,
    #[error("Sample rate must be between 8000 and 384000 Hz.")]
    BadSampleRate,
    #[error("Bit depth must be 16, 24 or 32.")]
    BadBitDepth,
    #[error("The session changed since this step was recorded.")]
    BadTiming,
    #[error("The clip is locked in place.")]
    ClipLocked,
    #[error("A clip cannot reach past the end of its file.")]
    PastSourceEnd,
    #[error("No marker with id {0}.")]
    NoMarker(Id),
}

fn track_index(s: &Session, id: Id) -> Result<usize, OpError> {
    s.tracks
        .iter()
        .position(|t| t.id == id)
        .ok_or(OpError::NoTrack(id))
}

fn id_taken(s: &Session, id: Id) -> bool {
    s.tracks
        .iter()
        .any(|t| t.id == id || t.clips.iter().any(|c| c.id == id))
        || s.media.iter().any(|m| m.id == id)
        || s.buses.iter().any(|b| b.id == id)
        || s.markers.iter().any(|m| m.id == id)
}

fn sort_markers(s: &mut Session) {
    s.markers.sort_by_key(|m| m.position);
}

/// Applies an op and returns its inverse. On error the session is unchanged.
pub fn apply(s: &mut Session, op: &Op) -> Result<Op, OpError> {
    match op {
        Op::SetEffects { target, chain } => {
            let slot = match *target {
                EffectTarget::Track(id) => {
                    let i = track_index(s, id)?;
                    &mut s.tracks[i].effects
                }
                EffectTarget::Clip(id) => {
                    &mut s
                        .tracks
                        .iter_mut()
                        .flat_map(|t| t.clips.iter_mut())
                        .find(|c| c.id == id)
                        .ok_or(OpError::NoClip(id))?
                        .effects
                }
                EffectTarget::Master => &mut s.master.effects,
            };
            let mut chain = chain.clone();
            chain.slots.truncate(crate::dsp::effects::MAX_SLOTS);
            let old = std::mem::replace(slot, chain);
            Ok(Op::SetEffects {
                target: *target,
                chain: old,
            })
        }
        Op::AddTrack {
            id,
            name,
            stereo,
            index,
        } => {
            if id_taken(s, *id) {
                return Err(OpError::DuplicateId(*id));
            }
            let layout = if *stereo {
                ChannelLayout::Stereo
            } else {
                ChannelLayout::Mono
            };
            let mut t = Track::new(name.clone(), layout);
            t.id = *id;
            s.tracks.insert((*index).min(s.tracks.len()), t);
            Ok(Op::RemoveTrack { id: *id })
        }
        Op::RestoreTrack { track, index } => {
            if id_taken(s, track.id) {
                return Err(OpError::DuplicateId(track.id));
            }
            s.tracks
                .insert((*index).min(s.tracks.len()), (**track).clone());
            Ok(Op::RemoveTrack { id: track.id })
        }
        Op::RemoveTrack { id } => {
            let i = track_index(s, *id)?;
            let track = s.tracks.remove(i);
            Ok(Op::RestoreTrack {
                track: Box::new(track),
                index: i,
            })
        }
        Op::MoveTrack { id, to } => {
            let from = track_index(s, *id)?;
            let t = s.tracks.remove(from);
            let to = (*to).min(s.tracks.len());
            s.tracks.insert(to, t);
            Ok(Op::MoveTrack { id: *id, to: from })
        }
        Op::SetTrack { id, changes } => {
            let i = track_index(s, *id)?;
            let t = &mut s.tracks[i];
            let mut old = TrackChanges::default();
            if let Some(v) = &changes.name {
                old.name = Some(std::mem::replace(&mut t.name, v.clone()));
            }
            if let Some(v) = changes.muted {
                old.muted = Some(std::mem::replace(&mut t.muted, v));
            }
            if let Some(v) = changes.solo {
                old.solo = Some(std::mem::replace(&mut t.solo, v));
            }
            if let Some(v) = changes.record_armed {
                old.record_armed = Some(std::mem::replace(&mut t.record_armed, v));
            }
            if let Some(v) = changes.height {
                old.height = Some(std::mem::replace(&mut t.height, v.clamp(40, 600)));
            }
            if let Some(v) = &changes.color {
                old.color = Some(std::mem::replace(&mut t.color, v.clone()));
            }
            if let Some(v) = changes.volume_db {
                old.volume_db = Some(std::mem::replace(&mut t.volume_db, v));
            }
            if let Some(v) = changes.pan {
                old.pan = Some(std::mem::replace(&mut t.pan, v.clamp(-100.0, 100.0)));
            }
            if let Some(v) = changes.polarity_inverted {
                old.polarity_inverted = Some(std::mem::replace(&mut t.polarity_inverted, v));
            }
            if let Some(v) = changes.collapsed {
                old.collapsed = Some(std::mem::replace(&mut t.collapsed, v));
            }
            if let Some(v) = changes.monitor_input {
                old.monitor_input = Some(std::mem::replace(&mut t.monitor_input, v));
            }
            if let Some(v) = &changes.input {
                old.input = Some(std::mem::replace(&mut t.input, v.clone()));
            }
            if let Some(v) = changes.role {
                old.role = Some(std::mem::replace(&mut t.role, v));
            }
            if let Some(v) = &changes.host_profile_id {
                let next = Some(v.clone()).filter(|v| !v.is_empty());
                old.host_profile_id =
                    Some(std::mem::replace(&mut t.host_profile_id, next).unwrap_or_default());
            }
            Ok(Op::SetTrack {
                id: *id,
                changes: old,
            })
        }
        Op::AddMedia { media } => {
            if id_taken(s, media.id) {
                return Err(OpError::DuplicateId(media.id));
            }
            s.media.push(media.clone());
            Ok(Op::RemoveMedia { id: media.id })
        }
        Op::RemoveMedia { id } => {
            let used = s
                .tracks
                .iter()
                .any(|t| t.clips.iter().any(|c| c.media_id == *id));
            if used {
                return Err(OpError::MediaInUse);
            }
            let i = s
                .media
                .iter()
                .position(|m| m.id == *id)
                .ok_or(OpError::NoMedia(*id))?;
            let media = s.media.remove(i);
            Ok(Op::AddMedia { media })
        }
        Op::AddClips { track_id, clips } => {
            let ti = track_index(s, *track_id)?;
            for c in clips {
                if id_taken(s, c.id) {
                    return Err(OpError::DuplicateId(c.id));
                }
                if s.media_by_id(c.media_id).is_none() {
                    return Err(OpError::NoMedia(c.media_id));
                }
                if c.source_out <= c.source_in {
                    return Err(OpError::EmptyClip);
                }
            }
            s.tracks[ti].clips.extend(clips.iter().map(ClipSpec::build));
            Ok(Op::RemoveClips {
                ids: clips.iter().map(|c| c.id).collect(),
            })
        }
        Op::RestoreClips { clips } => {
            for (track_id, c) in clips {
                track_index(s, *track_id)?;
                if id_taken(s, c.id) {
                    return Err(OpError::DuplicateId(c.id));
                }
            }
            for (track_id, c) in clips {
                let ti = track_index(s, *track_id)?;
                s.tracks[ti].clips.push(c.clone());
            }
            Ok(Op::RemoveClips {
                ids: clips.iter().map(|(_, c)| c.id).collect(),
            })
        }
        Op::RemoveClips { ids } => {
            for id in ids {
                if !s.tracks.iter().any(|t| t.clips.iter().any(|c| c.id == *id)) {
                    return Err(OpError::NoClip(*id));
                }
            }
            let mut removed = Vec::new();
            for t in &mut s.tracks {
                let (gone, keep): (Vec<_>, Vec<_>) =
                    t.clips.drain(..).partition(|c| ids.contains(&c.id));
                t.clips = keep;
                removed.extend(gone.into_iter().map(|c| (t.id, c)));
            }
            Ok(Op::RestoreClips { clips: removed })
        }
        Op::SetSampleRate { sample_rate, exact } => {
            if !(8_000..=384_000).contains(sample_rate) {
                return Err(OpError::BadSampleRate);
            }
            let from = s.sample_rate;
            let to = *sample_rate;
            let mut before = Vec::new();
            for_each_time(s, |v| before.push(*v));
            match exact {
                Some(values) => {
                    if values.len() != before.len() {
                        return Err(OpError::BadTiming);
                    }
                    let mut it = values.iter();
                    for_each_time(s, |v| *v = *it.next().expect("length checked"));
                }
                None => for_each_time(s, |v| *v = rescale(*v, from, to)),
            }
            let v = &mut s.view;
            v.scroll = rescale(v.scroll, from, to);
            v.playhead = rescale(v.playhead, from, to);
            if let Some(sel) = v.selection.as_mut() {
                sel.start = rescale(sel.start, from, to);
                sel.end = rescale(sel.end, from, to);
            }
            if from > 0 {
                v.samples_per_pixel *= f64::from(to) / f64::from(from);
            }
            s.sample_rate = to;
            Ok(Op::SetSampleRate {
                sample_rate: from,
                exact: Some(before),
            })
        }
        Op::SetClips { changes } => {
            // Work on a copy so a failure part way leaves nothing half done.
            let mut work = s.clone();
            let mut inverses = Vec::with_capacity(changes.len());
            for ch in changes {
                inverses.push(set_clip(&mut work, ch)?);
            }
            *s = work;
            inverses.reverse();
            Ok(Op::SetClips { changes: inverses })
        }
        Op::AddMarkers { markers } => {
            for m in markers {
                if id_taken(s, m.id) {
                    return Err(OpError::DuplicateId(m.id));
                }
            }
            s.markers.extend(markers.iter().cloned());
            sort_markers(s);
            Ok(Op::RemoveMarkers {
                ids: markers.iter().map(|m| m.id).collect(),
            })
        }
        Op::RemoveMarkers { ids } => {
            for id in ids {
                if !s.markers.iter().any(|m| m.id == *id) {
                    return Err(OpError::NoMarker(*id));
                }
            }
            let (gone, kept): (Vec<Marker>, Vec<Marker>) =
                s.markers.drain(..).partition(|m| ids.contains(&m.id));
            s.markers = kept;
            Ok(Op::AddMarkers { markers: gone })
        }
        Op::SetPlaylist { ids } => {
            let old = std::mem::replace(&mut s.playlist, ids.clone());
            Ok(Op::SetPlaylist { ids: old })
        }
        Op::SetMarkers { changes } => {
            for ch in changes {
                if !s.markers.iter().any(|m| m.id == ch.id) {
                    return Err(OpError::NoMarker(ch.id));
                }
            }
            let mut inverses = Vec::with_capacity(changes.len());
            for ch in changes {
                let m = s
                    .markers
                    .iter_mut()
                    .find(|m| m.id == ch.id)
                    .expect("checked above");
                let mut inv = MarkerChange {
                    id: ch.id,
                    ..MarkerChange::default()
                };
                if let Some(v) = &ch.name {
                    inv.name = Some(std::mem::replace(&mut m.name, v.clone()));
                }
                if let Some(v) = ch.kind {
                    inv.kind = Some(std::mem::replace(&mut m.kind, v));
                }
                if let Some(v) = ch.position {
                    inv.position = Some(std::mem::replace(&mut m.position, v));
                }
                if let Some(v) = ch.duration {
                    inv.duration = Some(std::mem::replace(&mut m.duration, v));
                }
                if let Some(v) = &ch.description {
                    inv.description = Some(std::mem::replace(&mut m.description, v.clone()));
                }
                if let Some(v) = &ch.color {
                    let new = if v.is_empty() { None } else { Some(v.clone()) };
                    inv.color = Some(std::mem::replace(&mut m.color, new).unwrap_or_default());
                }
                if ch.contracted_duration.is_some() || ch.clear_contracted == Some(true) {
                    let new = if ch.clear_contracted == Some(true) {
                        None
                    } else {
                        ch.contracted_duration
                    };
                    match std::mem::replace(&mut m.contracted_duration, new) {
                        Some(old) => inv.contracted_duration = Some(old),
                        None => inv.clear_contracted = Some(true),
                    }
                }
                inverses.push(inv);
            }
            sort_markers(s);
            inverses.reverse();
            Ok(Op::SetMarkers { changes: inverses })
        }
        Op::SetSessionInfo { name, bit_depth } => {
            if let Some(b) = bit_depth {
                if ![16, 24, 32].contains(b) {
                    return Err(OpError::BadBitDepth);
                }
            }
            let old_name = name
                .as_ref()
                .map(|n| std::mem::replace(&mut s.name, n.trim().to_string()));
            let old_bits = bit_depth.map(|b| std::mem::replace(&mut s.bit_depth, b));
            Ok(Op::SetSessionInfo {
                name: old_name,
                bit_depth: old_bits,
            })
        }
        Op::SetMasterVolume { volume_db } => {
            let old = std::mem::replace(&mut s.master.volume_db, volume_db.clamp(-96.0, 24.0));
            Ok(Op::SetMasterVolume { volume_db: old })
        }
        Op::Batch { ops } => {
            // Work on a copy so a failure part way leaves nothing half done.
            let mut work = s.clone();
            let mut inverses = Vec::with_capacity(ops.len());
            for op in ops {
                inverses.push(apply(&mut work, op)?);
            }
            *s = work;
            inverses.reverse();
            Ok(Op::Batch { ops: inverses })
        }
    }
}

fn set_clip(s: &mut Session, ch: &ClipChange) -> Result<ClipChange, OpError> {
    let (ti, ci) = s
        .tracks
        .iter()
        .enumerate()
        .find_map(|(ti, t)| {
            t.clips
                .iter()
                .position(|c| c.id == ch.id)
                .map(|ci| (ti, ci))
        })
        .ok_or(OpError::NoClip(ch.id))?;
    let to_track = match ch.track_id {
        Some(id) => Some(track_index(s, id)?),
        None => None,
    };
    let moves = ch.start.is_some()
        || ch.source_in.is_some()
        || ch.source_out.is_some()
        || ch.looped.is_some()
        || ch.unloop == Some(true)
        || ch.stretch.is_some()
        || ch.unstretch == Some(true)
        || to_track.is_some_and(|t| t != ti);
    let unlocking = ch.locked == Some(false);
    if moves && s.tracks[ti].clips[ci].locked && !unlocking {
        return Err(OpError::ClipLocked);
    }

    let mut inv = ClipChange {
        id: ch.id,
        ..ClipChange::default()
    };
    let c = &mut s.tracks[ti].clips[ci];
    fn swap<T: Clone>(field: &mut T, new: &Option<T>, old: &mut Option<T>) {
        if let Some(v) = new {
            *old = Some(std::mem::replace(field, v.clone()));
        }
    }
    swap(&mut c.name, &ch.name, &mut inv.name);
    swap(&mut c.start, &ch.start, &mut inv.start);
    swap(&mut c.source_in, &ch.source_in, &mut inv.source_in);
    swap(&mut c.source_out, &ch.source_out, &mut inv.source_out);
    swap(&mut c.muted, &ch.muted, &mut inv.muted);
    swap(&mut c.locked, &ch.locked, &mut inv.locked);
    swap(&mut c.fade_in, &ch.fade_in, &mut inv.fade_in);
    swap(&mut c.fade_out, &ch.fade_out, &mut inv.fade_out);
    let gain = ch.gain_db.map(|g| g.clamp(-96.0, 24.0));
    swap(&mut c.gain_db, &gain, &mut inv.gain_db);
    if ch.group.is_some() || ch.ungroup == Some(true) {
        let new = if ch.ungroup == Some(true) {
            None
        } else {
            ch.group
        };
        match std::mem::replace(&mut c.group, new) {
            Some(old) => inv.group = Some(old),
            None => inv.ungroup = Some(true),
        }
    }
    if ch.looped.is_some() || ch.unloop == Some(true) {
        let new = if ch.unloop == Some(true) {
            None
        } else {
            ch.looped
        };
        match std::mem::replace(&mut c.looped, new) {
            Some(old) => inv.looped = Some(old),
            None => inv.unloop = Some(true),
        }
    }
    if ch.stretch.is_some() || ch.unstretch == Some(true) {
        let new = if ch.unstretch == Some(true) {
            None
        } else {
            ch.stretch.map(|s| ClipStretch {
                ratio: s.ratio.clamp(STRETCH_MIN, STRETCH_MAX),
                ..s
            })
        };
        match std::mem::replace(&mut c.stretch, new) {
            Some(old) => inv.stretch = Some(old),
            None => inv.unstretch = Some(true),
        }
    }
    if ch.audio_type.is_some() || ch.clear_audio_type == Some(true) {
        let new = if ch.clear_audio_type == Some(true) {
            None
        } else {
            ch.audio_type
        };
        match std::mem::replace(&mut c.audio_type, new) {
            Some(old) => inv.audio_type = Some(old),
            None => inv.clear_audio_type = Some(true),
        }
    }
    if let Some(points) = &ch.volume {
        inv.volume = Some(set_points(c, EnvelopeTarget::Volume, points));
    }
    if let Some(points) = &ch.pan {
        inv.pan = Some(set_points(c, EnvelopeTarget::Pan, points));
    }
    if let Some(col) = &ch.color {
        let new = if col.is_empty() {
            None
        } else {
            Some(col.clone())
        };
        let old = std::mem::replace(&mut c.color, new);
        inv.color = Some(old.unwrap_or_default());
    }

    if c.source_out <= c.source_in || c.looped.is_some_and(|l| l.length == 0) {
        return Err(OpError::EmptyClip);
    }
    if let Some(l) = &mut c.looped {
        // The phase always lands inside the region.
        l.phase %= c.source_out - c.source_in;
    }
    let media_id = c.media_id;
    let source_out = c.source_out;
    if let Some(m) = s.media_by_id(media_id) {
        if m.length > 0 && source_out > m.length {
            return Err(OpError::PastSourceEnd);
        }
    }

    if let Some(to) = to_track.filter(|t| *t != ti) {
        let clip = s.tracks[ti].clips.remove(ci);
        s.tracks[to].clips.push(clip);
        inv.track_id = Some(s.tracks[ti].id);
    }
    Ok(inv)
}

/// Lowest and highest volume envelope values, in dB. The lowest is silence.
pub const VOLUME_DB_MIN: f64 = -60.0;
pub const VOLUME_DB_MAX: f64 = 12.0;

/// Lowest and highest clip pan envelope values: full left, full right.
pub const PAN_MIN: f64 = -100.0;
pub const PAN_MAX: f64 = 100.0;

/// The id a clip's envelope for `target` uses. Fixed per clip and
/// target, so removing and re-adding an envelope gives the same session
/// back.
pub fn envelope_id(clip_id: Id, target: &EnvelopeTarget) -> Id {
    match target {
        EnvelopeTarget::Pan => Uuid::from_u128(clip_id.as_u128() ^ 1),
        _ => clip_id,
    }
}

/// Sets a clip's points for a volume or pan envelope and returns the
/// old ones. Values are clamped to the target's range.
fn set_points(
    c: &mut Clip,
    target: EnvelopeTarget,
    points: &[EnvelopePoint],
) -> Vec<EnvelopePoint> {
    let (lo, hi) = match target {
        EnvelopeTarget::Pan => (PAN_MIN, PAN_MAX),
        _ => (VOLUME_DB_MIN, VOLUME_DB_MAX),
    };
    let i = c.envelopes.iter().position(|e| e.target == target);
    let old = i.map(|i| c.envelopes[i].points.clone()).unwrap_or_default();
    let mut new: Vec<EnvelopePoint> = points
        .iter()
        .map(|p| EnvelopePoint {
            value: p.value.clamp(lo, hi),
            ..*p
        })
        .collect();
    new.sort_by_key(|p| p.position);
    match (i, new.is_empty()) {
        (Some(i), true) => {
            c.envelopes.remove(i);
        }
        (Some(i), false) => c.envelopes[i].points = new,
        (None, true) => {}
        (None, false) => c.envelopes.push(Envelope {
            id: envelope_id(c.id, &target),
            target,
            enabled: true,
            points: new,
        }),
    }
    old
}

fn rescale(value: u64, from: u32, to: u32) -> u64 {
    if from == to || from == 0 {
        return value;
    }
    ((u128::from(value) * u128::from(to) + u128::from(from) / 2) / u128::from(from)) as u64
}

/// Visits every timeline value stored in session samples, in a fixed
/// order. Source offsets are in file samples and are not visited.
fn for_each_time(s: &mut Session, mut f: impl FnMut(&mut u64)) {
    fn envelopes(list: &mut [crate::session::model::Envelope], f: &mut impl FnMut(&mut u64)) {
        for e in list {
            for p in &mut e.points {
                f(&mut p.position);
            }
        }
    }
    for t in &mut s.tracks {
        for c in &mut t.clips {
            f(&mut c.start);
            f(&mut c.fade_in.length);
            f(&mut c.fade_out.length);
            envelopes(&mut c.envelopes, &mut f);
        }
        envelopes(&mut t.envelopes, &mut f);
    }
    for b in &mut s.buses {
        envelopes(&mut b.envelopes, &mut f);
    }
    envelopes(&mut s.master.envelopes, &mut f);
    for m in &mut s.markers {
        f(&mut m.position);
        f(&mut m.duration);
        if let Some(d) = m.contracted_duration.as_mut() {
            f(d);
        }
    }
}

/// View state is not an edit: no undo, no dirty flag.
pub fn set_view(s: &mut Session, view: SessionView) {
    s.view = view;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::model::new_id;

    fn media(id: Id) -> MediaRef {
        MediaRef {
            id,
            path: "C:/Shows/ep.wav".into(),
            relative_path: None,
            sample_rate: 48_000,
            channels: 2,
            length: 480_000,
        }
    }

    fn roundtrip(s: &mut Session, op: Op) {
        let before = s.clone();
        let inverse = apply(s, &op).unwrap();
        let after = s.clone();
        let redo = apply(s, &inverse).unwrap();
        assert_eq!(*s, before, "undo of {op:?}");
        apply(s, &redo).unwrap();
        assert_eq!(*s, after, "redo of {op:?}");
    }

    #[test]
    fn voice_chains_set_and_undo() {
        let mut s = Session::new("Test", 48_000);
        let t = new_id();
        roundtrip(
            &mut s,
            Op::AddTrack {
                id: t,
                name: "Host".into(),
                stereo: false,
                index: 0,
            },
        );
        let mut chain = EffectChain::default();
        for _ in 0..20 {
            chain.slots.push(crate::session::model::EffectSlot {
                id: new_id(),
                effect: "altaudio.gain".into(),
                enabled: true,
                params: Default::default(),
                preset: None,
            });
        }
        roundtrip(
            &mut s,
            Op::SetEffects {
                target: EffectTarget::Track(t),
                chain: chain.clone(),
            },
        );
        assert_eq!(s.tracks[0].effects.slots.len(), 16, "a chain holds 16");
        roundtrip(
            &mut s,
            Op::SetEffects {
                target: EffectTarget::Master,
                chain,
            },
        );
        let missing = Op::SetEffects {
            target: EffectTarget::Clip(new_id()),
            chain: EffectChain::default(),
        };
        assert!(matches!(apply(&mut s, &missing), Err(OpError::NoClip(_))));
    }

    #[test]
    fn tracks_add_move_set_remove_and_undo() {
        let mut s = Session::new("Test", 48_000);
        let (a, b, c) = (new_id(), new_id(), new_id());
        for (i, id) in [a, b, c].into_iter().enumerate() {
            roundtrip(
                &mut s,
                Op::AddTrack {
                    id,
                    name: format!("T{i}"),
                    stereo: true,
                    index: i,
                },
            );
        }
        roundtrip(&mut s, Op::MoveTrack { id: c, to: 0 });
        assert_eq!(s.tracks.iter().map(|t| t.id).collect::<Vec<_>>(), [c, a, b]);
        roundtrip(
            &mut s,
            Op::SetTrack {
                id: a,
                changes: TrackChanges {
                    name: Some("Host".into()),
                    muted: Some(true),
                    ..Default::default()
                },
            },
        );
        assert_eq!(s.tracks[1].name, "Host");
        roundtrip(&mut s, Op::RemoveTrack { id: b });
        assert_eq!(s.tracks.len(), 2);
    }

    #[test]
    fn clips_need_media_and_undo_cleanly() {
        let mut s = Session::new("Test", 48_000);
        let (t, m, c) = (new_id(), new_id(), new_id());
        apply(
            &mut s,
            &Op::AddTrack {
                id: t,
                name: "Host".into(),
                stereo: false,
                index: 0,
            },
        )
        .unwrap();
        let spec = ClipSpec {
            id: c,
            name: "ep".into(),
            media_id: m,
            start: 0,
            source_in: 0,
            source_out: 480_000,
        };
        assert_eq!(
            apply(
                &mut s,
                &Op::AddClips {
                    track_id: t,
                    clips: vec![spec.clone()]
                }
            ),
            Err(OpError::NoMedia(m))
        );
        roundtrip(
            &mut s,
            Op::Batch {
                ops: vec![
                    Op::AddMedia { media: media(m) },
                    Op::AddClips {
                        track_id: t,
                        clips: vec![spec],
                    },
                ],
            },
        );
        assert_eq!(s.tracks[0].clips.len(), 1);
        assert_eq!(
            apply(&mut s, &Op::RemoveMedia { id: m }),
            Err(OpError::MediaInUse)
        );
        roundtrip(&mut s, Op::RemoveClips { ids: vec![c] });
        // Removing a track with clips brings the clips back on undo.
        roundtrip(&mut s, Op::RemoveTrack { id: t });
    }

    #[test]
    fn failed_batch_changes_nothing() {
        let mut s = Session::new("Test", 48_000);
        let before = s.clone();
        let err = apply(
            &mut s,
            &Op::Batch {
                ops: vec![
                    Op::AddTrack {
                        id: new_id(),
                        name: "ok".into(),
                        stereo: true,
                        index: 0,
                    },
                    Op::RemoveTrack { id: new_id() },
                ],
            },
        );
        assert!(err.is_err());
        assert_eq!(s, before);
    }

    #[test]
    fn master_volume_undoes() {
        let mut s = Session::new("Test", 48_000);
        roundtrip(&mut s, Op::SetMasterVolume { volume_db: -3.0 });
        assert_eq!(s.master.volume_db, -3.0);
    }

    #[test]
    fn duplicate_ids_are_refused() {
        let mut s = Session::new("Test", 48_000);
        let id = new_id();
        let add = Op::AddTrack {
            id,
            name: "a".into(),
            stereo: true,
            index: 0,
        };
        apply(&mut s, &add).unwrap();
        assert_eq!(apply(&mut s, &add), Err(OpError::DuplicateId(id)));
    }

    #[test]
    fn ops_serialize_with_a_tag() {
        let op = Op::MoveTrack {
            id: new_id(),
            to: 2,
        };
        let json = serde_json::to_value(&op).unwrap();
        assert_eq!(json["op"], "move_track");
        let back: Op = serde_json::from_value(json).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn sample_rate_change_keeps_times_and_undoes_exactly() {
        let mut s = Session::new("Test", 48_000);
        let (t, m) = (new_id(), new_id());
        apply(
            &mut s,
            &Op::Batch {
                ops: vec![
                    Op::AddTrack {
                        id: t,
                        name: "Host".into(),
                        stereo: false,
                        index: 0,
                    },
                    Op::AddMedia { media: media(m) },
                    Op::AddClips {
                        track_id: t,
                        clips: vec![ClipSpec {
                            id: new_id(),
                            name: "a".into(),
                            media_id: m,
                            start: 48_001,
                            source_in: 10,
                            source_out: 4_800,
                        }],
                    },
                ],
            },
        )
        .unwrap();
        s.tracks[0].clips[0].fade_in.length = 4_800;
        s.view.playhead = 96_000;
        let before = s.clone();

        let undo = apply(
            &mut s,
            &Op::SetSampleRate {
                sample_rate: 44_100,
                exact: None,
            },
        )
        .unwrap();
        let c = &s.tracks[0].clips[0];
        assert_eq!(c.start, 44_101, "one second plus a sample, rounded");
        assert_eq!(c.fade_in.length, 4_410);
        assert_eq!(
            (c.source_in, c.source_out),
            (10, 4_800),
            "file samples stay"
        );
        assert_eq!(s.view.playhead, 88_200);

        apply(&mut s, &undo).unwrap();
        assert_eq!(s.tracks, before.tracks, "undo restores exact positions");
        assert_eq!(s.sample_rate, 48_000);
        assert_eq!(s.view.playhead, 96_000);
    }

    #[test]
    fn session_info_sets_name_and_bit_depth() {
        let mut s = Session::new("Test", 48_000);
        roundtrip(
            &mut s,
            Op::SetSessionInfo {
                name: Some("EP 441".into()),
                bit_depth: Some(24),
            },
        );
        assert_eq!((s.name.as_str(), s.bit_depth), ("EP 441", 24));
        assert_eq!(
            apply(
                &mut s,
                &Op::SetSessionInfo {
                    name: None,
                    bit_depth: Some(20)
                }
            ),
            Err(OpError::BadBitDepth)
        );
    }

    #[test]
    fn set_clips_moves_trims_and_respects_locks() {
        let mut s = Session::new("Test", 48_000);
        let (a, b, m, c) = (new_id(), new_id(), new_id(), new_id());
        let mut ops = vec![Op::AddMedia { media: media(m) }];
        for id in [a, b] {
            ops.push(Op::AddTrack {
                id,
                name: "T".into(),
                stereo: false,
                index: 0,
            });
        }
        ops.push(Op::AddClips {
            track_id: a,
            clips: vec![ClipSpec {
                id: c,
                name: "c".into(),
                media_id: m,
                start: 0,
                source_in: 0,
                source_out: 48_000,
            }],
        });
        apply(&mut s, &Op::Batch { ops }).unwrap();

        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    track_id: Some(b),
                    start: Some(1_000),
                    source_in: Some(100),
                    gain_db: Some(-3.0),
                    ..ClipChange::default()
                }],
            },
        );
        let t = s.tracks.iter().find(|t| t.id == b).unwrap();
        assert_eq!(t.clips[0].start, 1_000);
        assert_eq!(t.clips[0].source_in, 100);

        let past = Op::SetClips {
            changes: vec![ClipChange {
                id: c,
                source_out: Some(480_001),
                ..ClipChange::default()
            }],
        };
        assert_eq!(apply(&mut s, &past), Err(OpError::PastSourceEnd));
        let empty = Op::SetClips {
            changes: vec![ClipChange {
                id: c,
                source_out: Some(100),
                ..ClipChange::default()
            }],
        };
        assert_eq!(apply(&mut s, &empty), Err(OpError::EmptyClip));

        apply(
            &mut s,
            &Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    locked: Some(true),
                    ..ClipChange::default()
                }],
            },
        )
        .unwrap();
        let mv = Op::SetClips {
            changes: vec![ClipChange {
                id: c,
                start: Some(5),
                ..ClipChange::default()
            }],
        };
        assert_eq!(apply(&mut s, &mv), Err(OpError::ClipLocked));
    }

    #[test]
    fn markers_add_set_remove_and_undo() {
        let mut s = Session::new("Test", 48_000);
        let (a, b) = (new_id(), new_id());
        let mk = |id, position| Marker {
            id,
            name: "M".into(),
            kind: MarkerKind::Cue,
            position,
            duration: 0,
            description: String::new(),
            chapter: None,
            contracted_duration: None,
            color: None,
        };
        roundtrip(
            &mut s,
            Op::AddMarkers {
                markers: vec![mk(a, 500), mk(b, 100)],
            },
        );
        assert_eq!(s.markers[0].id, b, "kept in time order");
        roundtrip(
            &mut s,
            Op::SetMarkers {
                changes: vec![MarkerChange {
                    id: b,
                    position: Some(900),
                    name: Some("Ad".into()),
                    kind: Some(MarkerKind::AdSlot),
                    ..MarkerChange::default()
                }],
            },
        );
        assert_eq!(s.markers[1].name, "Ad");
        let before = s.clone();
        let inv = apply(&mut s, &Op::RemoveMarkers { ids: vec![a] }).unwrap();
        assert_eq!(s.markers.len(), 1);
        apply(&mut s, &inv).unwrap();
        assert_eq!(s, before);
    }

    #[test]
    fn grouping_clips_undoes() {
        let mut s = Session::new("Test", 48_000);
        let (t, m, c, g) = (new_id(), new_id(), new_id(), new_id());
        apply(
            &mut s,
            &Op::Batch {
                ops: vec![
                    Op::AddMedia { media: media(m) },
                    Op::AddTrack {
                        id: t,
                        name: "A".into(),
                        stereo: false,
                        index: 0,
                    },
                    Op::AddClips {
                        track_id: t,
                        clips: vec![ClipSpec {
                            id: c,
                            name: "c".into(),
                            media_id: m,
                            start: 0,
                            source_in: 0,
                            source_out: 100,
                        }],
                    },
                ],
            },
        )
        .unwrap();
        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    group: Some(g),
                    ..ClipChange::default()
                }],
            },
        );
        assert_eq!(s.tracks[0].clips[0].group, Some(g));
        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    ungroup: Some(true),
                    ..ClipChange::default()
                }],
            },
        );
        assert_eq!(s.tracks[0].clips[0].group, None);
    }

    #[test]
    fn clip_volume_points_set_and_undo() {
        use crate::session::model::EnvelopeCurve;
        let mut s = Session::new("Test", 48_000);
        let (t, m, c) = (new_id(), new_id(), new_id());
        apply(
            &mut s,
            &Op::Batch {
                ops: vec![
                    Op::AddMedia { media: media(m) },
                    Op::AddTrack {
                        id: t,
                        name: "A".into(),
                        stereo: false,
                        index: 0,
                    },
                    Op::AddClips {
                        track_id: t,
                        clips: vec![ClipSpec {
                            id: c,
                            name: "c".into(),
                            media_id: m,
                            start: 0,
                            source_in: 0,
                            source_out: 48_000,
                        }],
                    },
                ],
            },
        )
        .unwrap();
        let pt = |position, value| EnvelopePoint {
            position,
            value,
            curve: EnvelopeCurve::Linear,
        };
        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    volume: Some(vec![pt(24_000, -99.0), pt(0, 0.0)]),
                    ..ClipChange::default()
                }],
            },
        );
        let env = &s.tracks[0].clips[0].envelopes[0];
        assert_eq!(env.points[0].position, 0, "sorted by position");
        assert_eq!(env.points[1].value, VOLUME_DB_MIN, "clamped");
        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    volume: Some(vec![]),
                    ..ClipChange::default()
                }],
            },
        );
        assert!(s.tracks[0].clips[0].envelopes.is_empty());

        // Pan sits beside volume, with its own id and range.
        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    volume: Some(vec![pt(0, -6.0)]),
                    pan: Some(vec![pt(0, -250.0), pt(48_000, 40.0)]),
                    ..ClipChange::default()
                }],
            },
        );
        let envs = &s.tracks[0].clips[0].envelopes;
        assert_eq!(envs.len(), 2);
        let pan = envs
            .iter()
            .find(|e| e.target == EnvelopeTarget::Pan)
            .unwrap();
        assert_eq!(pan.points[0].value, PAN_MIN, "clamped to full left");
        assert_ne!(pan.id, c, "pan has its own id");
        assert_eq!(pan.id, envelope_id(c, &EnvelopeTarget::Pan));
    }

    #[test]
    fn loop_set_and_undo() {
        let mut s = Session::new("Test", 48_000);
        let (t, m, c) = (new_id(), new_id(), new_id());
        apply(
            &mut s,
            &Op::Batch {
                ops: vec![
                    Op::AddMedia { media: media(m) },
                    Op::AddTrack {
                        id: t,
                        name: "A".into(),
                        stereo: false,
                        index: 0,
                    },
                    Op::AddClips {
                        track_id: t,
                        clips: vec![ClipSpec {
                            id: c,
                            name: "c".into(),
                            media_id: m,
                            start: 0,
                            source_in: 0,
                            source_out: 48_000,
                        }],
                    },
                ],
            },
        )
        .unwrap();
        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    looped: Some(ClipLoop {
                        length: 200_000,
                        phase: 50_000,
                    }),
                    ..ClipChange::default()
                }],
            },
        );
        let l = s.tracks[0].clips[0].looped.unwrap();
        assert_eq!(l.length, 200_000);
        assert_eq!(l.phase, 2_000, "phase wraps into the region");
        assert_eq!(
            s.tracks[0].clips[0].timeline_length(48_000, 48_000),
            200_000
        );
        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    unloop: Some(true),
                    ..ClipChange::default()
                }],
            },
        );
        assert!(s.tracks[0].clips[0].looped.is_none());
        let zero = apply(
            &mut s,
            &Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    looped: Some(ClipLoop {
                        length: 0,
                        phase: 0,
                    }),
                    ..ClipChange::default()
                }],
            },
        );
        assert!(zero.is_err(), "a loop needs a length");
    }

    #[test]
    fn stretch_set_clamped_and_undone() {
        let mut s = Session::new("Test", 48_000);
        let (t, m, c) = (new_id(), new_id(), new_id());
        apply(
            &mut s,
            &Op::Batch {
                ops: vec![
                    Op::AddMedia { media: media(m) },
                    Op::AddTrack {
                        id: t,
                        name: "A".into(),
                        stereo: false,
                        index: 0,
                    },
                    Op::AddClips {
                        track_id: t,
                        clips: vec![ClipSpec {
                            id: c,
                            name: "c".into(),
                            media_id: m,
                            start: 0,
                            source_in: 0,
                            source_out: 48_000,
                        }],
                    },
                ],
            },
        )
        .unwrap();
        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    stretch: Some(ClipStretch {
                        ratio: 1.25,
                        keep_pitch: true,
                    }),
                    ..ClipChange::default()
                }],
            },
        );
        assert_eq!(s.tracks[0].clips[0].timeline_length(48_000, 48_000), 60_000);
        apply(
            &mut s,
            &Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    stretch: Some(ClipStretch {
                        ratio: 99.0,
                        keep_pitch: false,
                    }),
                    ..ClipChange::default()
                }],
            },
        )
        .unwrap();
        assert_eq!(s.tracks[0].clips[0].stretch.unwrap().ratio, STRETCH_MAX);
        roundtrip(
            &mut s,
            Op::SetClips {
                changes: vec![ClipChange {
                    id: c,
                    unstretch: Some(true),
                    ..ClipChange::default()
                }],
            },
        );
        assert!(s.tracks[0].clips[0].stretch.is_none());
    }
}
