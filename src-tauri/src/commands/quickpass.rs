// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! QuickPass measuring: how loud each clip is, window by window. The
//! panel turns these levels into findings, so the rules stay in one
//! tested place on the front end and this side only listens. Clicks and
//! clipping are found here (they need every sample), and repaired into a
//! copy of the clip's audio.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Emitter, State};

use super::session::OpenSession;
use crate::dsp::effects::{find_spots, repair_spots, Spots};
use crate::export::mixdown::{self, MixdownRequest};
use crate::export::region;
use crate::session::model::{Id, Session};

/// Scan progress, in clips, sent as `quickpass-progress` events.
#[derive(Clone, Serialize)]
struct ScanProgress {
    done: usize,
    total: usize,
    /// Spots found so far, for scans that count them as they go (clicks and
    /// clipping). `None` for level scans, whose findings are worked out
    /// after every clip has been measured.
    found: Option<usize>,
}

/// The stop flag of the scan that is running, so Cancel can really stop it and a new scan
/// never waits behind an old one.
static CURRENT: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

const STOPPED: &str = "Scan stopped.";

/// Starts a scan: stops any scan still running and hands back the flag for the new one.
fn begin_scan() -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    let mut current = CURRENT.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(old) = current.replace(flag.clone()) {
        old.store(true, Ordering::Relaxed);
    }
    flag
}

/// Stops the scan that is running. Does nothing when none is.
#[tauri::command]
pub fn quickpass_cancel() {
    if let Some(flag) = CURRENT.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
}

/// Clips that could not be rendered. A scan that read nothing at all must say so, not look like
/// "nothing found".
#[derive(Default)]
struct Failures {
    count: usize,
    first: Option<String>,
}

impl Failures {
    fn note(&mut self, error: String) {
        self.count += 1;
        self.first.get_or_insert(error);
    }

    fn verdict(&self, read: usize) -> Result<(), String> {
        match (&self.first, read) {
            (Some(e), 0) => Err(format!(
                "The scan could not read {} clip{}: {e}",
                self.count,
                if self.count == 1 { "" } else { "s" }
            )),
            _ => Ok(()),
        }
    }
}

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
    /// RMS level of each window, dBFS.
    pub db: Vec<f32>,
    /// Highest sample in each window, dBFS. Used to find clipping.
    pub peak: Vec<f32>,
}

/// Highest absolute sample in dBFS of each `window`-frame block of
/// stereo interleaved audio.
pub fn window_peak(stereo: &[f32], window: usize) -> Vec<f32> {
    stereo
        .chunks(window.max(1) * 2)
        .map(|block| {
            let p = block.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            if p <= 0.0 {
                FLOOR_DB
            } else {
                (20.0 * p.log10()).max(FLOOR_DB)
            }
        })
        .collect()
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

/// One clip as it plays (gain and Voice Chain included), stereo at the
/// session rate, with its timeline start.
fn render_clip(
    session: &Session,
    id: Id,
    cancel: &AtomicBool,
) -> Result<Option<(u64, Vec<f32>)>, String> {
    let mut s = session.clone();
    let mut start = None;
    for t in &mut s.tracks {
        t.clips.retain(|c| c.id == id);
        if let Some(c) = t.clips.first() {
            start = Some(c.start);
        }
    }
    let Some(start) = start else {
        return Ok(None);
    };
    let end = mixdown::session_end(&s);
    if end <= start {
        return Ok(None);
    }
    let req = MixdownRequest {
        range: Some((start, end)),
        clip_only: true,
        ..Default::default()
    };
    let mut audio: Vec<f32> = Vec::with_capacity(((end - start) * 2) as usize);
    mixdown::render_mix(
        &s,
        &req,
        cancel,
        |_, _| {},
        |block| {
            audio.extend_from_slice(block);
            Ok(())
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(Some((start, audio)))
}

fn measure(
    session: &Session,
    id: Id,
    window_ms: u32,
    cancel: &AtomicBool,
) -> Result<Option<ClipLevels>, String> {
    let Some((start, audio)) = render_clip(session, id, cancel)? else {
        return Ok(None);
    };
    let window = (u64::from(session.sample_rate) * u64::from(window_ms) / 1000).max(1);
    Ok(Some(ClipLevels {
        clip_id: id,
        start,
        window,
        db: window_db(&audio, window as usize),
        peak: window_peak(&audio, window as usize),
    }))
}

/// Clicks or clipped spots found in one clip.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipSpots {
    pub clip_id: Id,
    /// Each spot as [first, last] timeline sample.
    pub spans: Vec<[u64; 2]>,
}

/// The session to scan: the multitrack one, or the Waveform editor's file
/// (as a one-track session) when `wave`.
fn pick_session(
    open: &State<'_, OpenSession>,
    wave_doc: &State<'_, super::wave::OpenWave>,
    wave: bool,
) -> Result<crate::session::model::Session, String> {
    if wave {
        return wave_doc
            .0
            .lock()
            .map_err(|e| e.to_string())?
            .as_ref()
            .map(|w| w.session.clone())
            .ok_or_else(|| "No file is open in the Waveform editor.".to_string());
    }
    Ok(open.0.lock().map_err(|e| e.to_string())?.session.clone())
}

/// Finds clicks or clipping in the given clips, as they play.
#[tauri::command]
pub async fn quickpass_spots(
    app: AppHandle,
    open: State<'_, OpenSession>,
    wave_doc: State<'_, super::wave::OpenWave>,
    wave: Option<bool>,
    clip_ids: Vec<Id>,
    spots: Spots,
) -> Result<Vec<ClipSpots>, String> {
    let session = pick_session(&open, &wave_doc, wave.unwrap_or(false))?;
    let wanted: HashSet<Id> = clip_ids.into_iter().collect();
    let cancel = begin_scan();
    spawn_blocking(move || {
        let rate = session.sample_rate;
        let mut ids: Vec<Id> = session
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter().map(|c| c.id))
            .filter(|id| wanted.contains(id))
            .collect();
        ids.sort();
        let total = ids.len();
        let mut so_far = 0usize;
        let mut out: Vec<ClipSpots> = Vec::new();
        let mut failed = Failures::default();
        for (n, id) in ids.into_iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                return Err(STOPPED.to_string());
            }
            match render_clip(&session, id, &cancel) {
                Ok(Some((start, audio))) => {
                    let spans: Vec<[u64; 2]> = find_spots(&audio, 2, rate, spots)
                        .into_iter()
                        .map(|(a, b)| [start + a, start + b])
                        .collect();
                    so_far += spans.len();
                    out.push(ClipSpots { clip_id: id, spans });
                }
                Ok(None) => {}
                Err(e) => {
                    if cancel.load(Ordering::Relaxed) {
                        return Err(STOPPED.to_string());
                    }
                    failed.note(e);
                }
            }
            let _ = app.emit(
                "quickpass-progress",
                ScanProgress {
                    done: n + 1,
                    total,
                    found: Some(so_far),
                },
            );
        }
        failed.verdict(out.len())?;
        Ok(out)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// QuickPass repair: copies source frames `from..to` of `source` to a new
/// 32-bit float WAV in `folder`, with `spans` (frames from `from`)
/// repaired and everything else untouched. The original file is never
/// changed; the clip is pointed at the copy, so undo brings it back.
#[tauri::command]
pub async fn quickpass_repair_copy(
    source: PathBuf,
    from: u64,
    to: u64,
    folder: PathBuf,
    name: String,
    spots: Spots,
    spans: Vec<[u64; 2]>,
) -> Result<PathBuf, String> {
    spawn_blocking(move || {
        std::fs::create_dir_all(&folder)
            .map_err(|e| format!("Could not create {}: {e}", folder.display()))?;
        let path = mixdown::unique_path(&folder, &name, "wav");
        region::copy_region(&source, from, to, &path).map_err(|e| e.to_string())?;
        let fail = |e: hound::Error| {
            let _ = std::fs::remove_file(&path);
            e.to_string()
        };
        let mut r = hound::WavReader::open(&path).map_err(fail)?;
        let spec = r.spec();
        let mut audio: Vec<f32> = r.samples::<f32>().collect::<Result<_, _>>().map_err(fail)?;
        drop(r);
        let list: Vec<(u64, u64)> = spans.iter().map(|s| (s[0], s[1])).collect();
        repair_spots(
            &mut audio,
            usize::from(spec.channels),
            spec.sample_rate,
            spots,
            &list,
        );
        let mut w = hound::WavWriter::create(&path, spec).map_err(fail)?;
        for v in audio {
            w.write_sample(v).map_err(fail)?;
        }
        w.finalize().map_err(fail)?;
        Ok(path)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Measures the given clips as they play (gain and Voice Chain included).
#[tauri::command]
pub async fn quickpass_levels(
    app: AppHandle,
    open: State<'_, OpenSession>,
    wave_doc: State<'_, super::wave::OpenWave>,
    wave: Option<bool>,
    clip_ids: Vec<Id>,
    window_ms: u32,
) -> Result<Vec<ClipLevels>, String> {
    let session = pick_session(&open, &wave_doc, wave.unwrap_or(false))?;
    let wanted: HashSet<Id> = clip_ids.into_iter().collect();
    let cancel = begin_scan();
    spawn_blocking(move || {
        let window_ms = window_ms.clamp(5, 500);
        let mut ids: Vec<Id> = session
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter().map(|c| c.id))
            .filter(|id| wanted.contains(id))
            .collect();
        ids.sort();
        let total = ids.len();
        let mut out: Vec<ClipLevels> = Vec::new();
        let mut failed = Failures::default();
        for (n, id) in ids.into_iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                return Err(STOPPED.to_string());
            }
            match measure(&session, id, window_ms, &cancel) {
                Ok(Some(levels)) => out.push(levels),
                Ok(None) => {}
                Err(e) => {
                    if cancel.load(Ordering::Relaxed) {
                        return Err(STOPPED.to_string());
                    }
                    failed.note(e);
                }
            }
            let _ = app.emit(
                "quickpass-progress",
                ScanProgress {
                    done: n + 1,
                    total,
                    found: None,
                },
            );
        }
        failed.verdict(out.len())?;
        Ok(out)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_scan_stops_the_old_one_and_cancel_stops_the_current_one() {
        let first = begin_scan();
        assert!(!first.load(Ordering::Relaxed));
        let second = begin_scan();
        assert!(first.load(Ordering::Relaxed));
        assert!(!second.load(Ordering::Relaxed));
        quickpass_cancel();
        assert!(second.load(Ordering::Relaxed));
    }

    #[test]
    fn a_scan_that_read_nothing_says_why() {
        let mut failed = Failures::default();
        assert!(failed.verdict(0).is_ok());
        failed.note("no such file".into());
        failed.note("other".into());
        let e = failed.verdict(0).unwrap_err();
        assert!(e.contains("2 clips") && e.contains("no such file"), "{e}");
        assert!(failed.verdict(3).is_ok());
    }

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
    fn peaks_find_the_loudest_sample() {
        let mut a = vec![0.0f32; 8];
        a[5] = -1.0;
        let p = window_peak(&a, 2);
        assert_eq!(p.len(), 2);
        assert_eq!(p[0], FLOOR_DB);
        assert!(p[1].abs() < 1e-4);
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
