// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Clip > Phase Align: how far to move one clip so it lines up with
//! another recording of the same sound. The interface makes the move
//! (and any polarity flip) as one undoable edit.

use tauri::async_runtime::spawn_blocking;
use tauri::State;

use super::effects::{isolate, render};
use super::session::OpenSession;
use crate::dsp::align::{align, Alignment};
use crate::dsp::sync::{sync, SyncResult};
use crate::session::model::{Id, Session};

/// Longest stretch compared, from where the two clips start to overlap.
const MAX_COMPARE_SECONDS: u64 = 30;

/// The clip's timeline span.
fn span(session: &Session, id: Id) -> Option<(u64, u64)> {
    let c = session
        .tracks
        .iter()
        .flat_map(|t| &t.clips)
        .find(|c| c.id == id)?;
    let rate = session
        .media
        .iter()
        .find(|m| m.id == c.media_id)
        .map_or(session.sample_rate, |m| m.sample_rate);
    Some((
        c.start,
        c.start + c.timeline_length(rate, session.sample_rate),
    ))
}

fn mono(stereo: &[f32]) -> Vec<f32> {
    stereo
        .as_chunks::<2>()
        .0
        .iter()
        .map(|f| (f[0] + f[1]) * 0.5)
        .collect()
}

/// Where `other` sits against `reference`, looking up to `max_ms` either way.
#[tauri::command]
pub async fn phase_align(
    open: State<'_, OpenSession>,
    reference: Id,
    other: Id,
    max_ms: f64,
) -> Result<Alignment, String> {
    let session = open.0.lock().map_err(|e| e.to_string())?.session.clone();
    spawn_blocking(move || {
        let rate = session.sample_rate;
        let (a0, a1) = span(&session, reference).ok_or("The reference clip is gone.")?;
        let (b0, b1) = span(&session, other).ok_or("The clip to move is gone.")?;
        let max_lag = (max_ms.clamp(1.0, 2_000.0) / 1000.0 * f64::from(rate)) as u64;
        // Compare where both play, widened by the search so a late clip
        // still has its matching part inside.
        let start = a0.max(b0).saturating_sub(max_lag);
        let end = a1.min(b1) + max_lag;
        let end = end.min(start + MAX_COMPARE_SECONDS * u64::from(rate));
        if a0.max(b0) >= a1.min(b1) {
            return Err(
                "The two clips do not play at the same time, so there is nothing to line up."
                    .into(),
            );
        }
        let ra = mono(&render(
            &isolate(&session, None, Some(reference)),
            start,
            end,
        )?);
        let rb = mono(&render(&isolate(&session, None, Some(other)), start, end)?);
        let found = align(&ra, &rb, max_lag as usize).ok_or("One of the clips is silent there.")?;
        Ok(found)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Rate the two tracks are compared at: speech lives well below 8 kHz, and a lower rate keeps
/// half an hour of two tracks small.
const SYNC_RATE: u32 = 16_000;
/// Longest stretch of each track Sync Tracks listens to.
const SYNC_MAX_SECONDS: u64 = 1_800;
const SYNC_CHUNK_SECONDS: u64 = 60;

/// Where the last clip on a track ends.
fn track_end(session: &Session, id: Id) -> Option<u64> {
    let track = session.tracks.iter().find(|t| t.id == id)?;
    track
        .clips
        .iter()
        .map(|c| {
            let rate = session
                .media
                .iter()
                .find(|m| m.id == c.media_id)
                .map_or(session.sample_rate, |m| m.sample_rate);
            c.start + c.timeline_length(rate, session.sample_rate)
        })
        .max()
}

/// Stereo to mono at `1 / factor` of the rate, averaging each block of frames.
fn decimate(stereo: &[f32], factor: usize) -> Vec<f32> {
    let scale = 1.0 / (2 * factor) as f32;
    stereo
        .chunks_exact(2 * factor)
        .map(|c| c.iter().sum::<f32>() * scale)
        .collect()
}

/// One track's audio from the start of the session to `end`, mono and decimated, rendered a
/// minute at a time so long sessions stay small.
fn render_track(session: &Session, id: Id, end: u64, factor: usize) -> Result<Vec<f32>, String> {
    let only = isolate(session, Some(id), None);
    let step = factor as u64;
    let chunk = (SYNC_CHUNK_SECONDS * u64::from(session.sample_rate) / step) * step;
    let mut out = Vec::with_capacity((end / step) as usize);
    let mut at = 0;
    while at < end {
        let to = (at + chunk).min(end);
        out.extend(decimate(&render(&only, at, to)?, factor));
        at = to;
    }
    Ok(out)
}

/// Multitrack > Sync Tracks: how far `other` trails `reference`, and how fast the gap grows.
/// Measures only; the interface moves the track.
#[tauri::command]
pub async fn sync_tracks(
    open: State<'_, OpenSession>,
    reference: Id,
    other: Id,
) -> Result<SyncResult, String> {
    let session = open.0.lock().map_err(|e| e.to_string())?.session.clone();
    spawn_blocking(move || {
        let rate = session.sample_rate;
        let a = track_end(&session, reference).ok_or("The first track has no audio.")?;
        let b = track_end(&session, other).ok_or("The second track has no audio.")?;
        let end = a.max(b).min(SYNC_MAX_SECONDS * u64::from(rate));
        let factor = (rate / SYNC_RATE).max(1) as usize;
        let ra = render_track(&session, reference, end, factor)?;
        let rb = render_track(&session, other, end, factor)?;
        let mut found = sync(&ra, &rb, rate / factor as u32).map_err(|e| e.to_string())?;
        found.offset *= factor as i64;
        Ok(found)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimate_averages_blocks_of_frames() {
        let stereo = [1.0, 1.0, 3.0, 3.0, 5.0, 5.0, 7.0, 9.0];
        assert_eq!(decimate(&stereo, 2), vec![2.0, 6.5]);
        assert_eq!(decimate(&stereo, 3), vec![3.0]);
    }
}
