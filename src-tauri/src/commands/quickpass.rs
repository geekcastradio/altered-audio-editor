// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! QuickPass measuring: how loud each clip is, window by window. The
//! panel turns these levels into findings, so the rules stay in one
//! tested place on the front end and this side only listens.

use std::collections::HashSet;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::State;

use super::session::OpenSession;
use crate::export::mixdown::{self, MixdownRequest};
use crate::session::model::{Id, Session};

/// Floor reported for digital silence, in dBFS.
pub const FLOOR_DB: f32 = -120.0;

/// Loudness of one clip: the level of each window in dBFS, in order.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipLevels {
    pub clip_id: Id,
    /// Timeline position of the first window, in session samples.
    pub start: u64,
    /// Session samples in each window (the last one may be shorter).
    pub window: u64,
    pub db: Vec<f32>,
}

/// RMS level in dBFS of each `window`-frame block of stereo
/// interleaved audio. The last block may be shorter.
pub fn window_db(stereo: &[f32], window: usize) -> Vec<f32> {
    let window = window.max(1);
    stereo
        .chunks(window * 2)
        .map(|block| {
            let sum: f64 = block.iter().map(|s| f64::from(*s) * f64::from(*s)).sum();
            let ms = sum / block.len().max(1) as f64;
            if ms <= 0.0 {
                FLOOR_DB
            } else {
                ((10.0 * ms.log10()) as f32).max(FLOOR_DB)
            }
        })
        .collect()
}

fn measure(session: &Session, id: Id, window_ms: u32) -> Option<ClipLevels> {
    let mut s = session.clone();
    let mut start = None;
    for t in &mut s.tracks {
        t.clips.retain(|c| c.id == id);
        if let Some(c) = t.clips.first() {
            start = Some(c.start);
        }
    }
    let start = start?;
    let end = mixdown::session_end(&s);
    if end <= start {
        return None;
    }
    let window = (u64::from(s.sample_rate) * u64::from(window_ms) / 1000).max(1);
    let req = MixdownRequest {
        range: Some((start, end)),
        clip_only: true,
        ..Default::default()
    };
    let mut audio: Vec<f32> = Vec::with_capacity(((end - start) * 2) as usize);
    mixdown::render_mix(&s, &req, &AtomicBool::new(false), |_, _| {}, |block| {
        audio.extend_from_slice(block);
        Ok(())
    })
    .ok()?;
    Some(ClipLevels { clip_id: id, start, window, db: window_db(&audio, window as usize) })
}

/// Measures the given clips as they play (gain and Voice Chain included).
#[tauri::command]
pub async fn quickpass_levels(
    open: State<'_, OpenSession>,
    clip_ids: Vec<Id>,
    window_ms: u32,
) -> Result<Vec<ClipLevels>, String> {
    let session = open.0.lock().map_err(|e| e.to_string())?.session.clone();
    let wanted: HashSet<Id> = clip_ids.into_iter().collect();
    spawn_blocking(move || {
        let window_ms = window_ms.clamp(5, 500);
        let mut ids: Vec<Id> = session
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter().map(|c| c.id))
            .filter(|id| wanted.contains(id))
            .collect();
        ids.sort();
        ids.into_iter().filter_map(|id| measure(&session, id, window_ms)).collect()
    })
    .await
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_reads_as_the_floor() {
        assert_eq!(window_db(&[0.0; 40], 10), vec![FLOOR_DB, FLOOR_DB]);
    }

    #[test]
    fn full_scale_square_is_zero_db_and_half_is_minus_six() {
        let db = window_db(&[1.0; 8], 4);
        assert!(db[0].abs() < 1e-4);
        let half = window_db(&[0.5; 8], 4);
        assert!((half[0] + 6.0206).abs() < 1e-3);
    }

    #[test]
    fn last_window_may_be_short() {
        let mut a = vec![0.0f32; 8];
        a.extend([1.0; 4]);
        let db = window_db(&a, 4);
        assert_eq!(db.len(), 2);
        assert_eq!(db[0], FLOOR_DB);
        assert!(db[1].abs() < 1e-4);
    }
}
