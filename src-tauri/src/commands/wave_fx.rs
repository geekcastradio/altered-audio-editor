// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Voice Chain in the Waveform editor: the chain plays as a preview on
//! the open file, and Apply writes it into the audio. Apply renders the
//! selection (or the whole file) through the chain into a new file; the
//! interface swaps that stretch in as one undoable edit, like any other
//! destructive edit.

use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Manager, State};

use super::spectral::{is_mono, render, snapshot, write_stretch, SpectralResult};
use super::wave::OpenWave;
use crate::dsp::effects::ChainProcessor;
use crate::export::mixdown;
use crate::session::model::EffectChain;

/// Audio before the range the chain hears first, so filters and level
/// detectors have settled when the range starts.
const WARM_SECONDS: f64 = 0.25;
/// Blend from the untouched audio into the processed audio, and back,
/// at each end of the range, so a change in level there is not a step.
const EDGE_SECONDS: f64 = 0.005;

/// Runs `chain` over timeline samples `start..end` of `audio`, a stereo
/// render that starts `warm` frames before `start` and runs `latency`
/// frames past `end`. Returns the processed range, `end - start` frames.
pub fn process_range(
    chain: &mut ChainProcessor,
    audio: &[f32],
    warm: usize,
    frames: usize,
    edge: usize,
) -> Vec<f32> {
    let latency = chain.latency();
    let mut wet = audio.to_vec();
    for block in wet.chunks_mut(4096 * 2) {
        chain.process(block);
    }
    let dry = audio.as_chunks::<2>().0;
    let wet = wet.as_chunks::<2>().0;
    let edge = edge.min(frames / 2);
    let mut out = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let d = dry.get(warm + i).copied().unwrap_or([0.0; 2]);
        let w = wet.get(warm + latency + i).copied().unwrap_or([0.0; 2]);
        let k = if edge == 0 {
            1.0
        } else {
            let from_edge = i.min(frames - 1 - i);
            (from_edge as f32 / edge as f32).min(1.0)
        };
        out.push(d[0] + (w[0] - d[0]) * k);
        out.push(d[1] + (w[1] - d[1]) * k);
    }
    out
}

/// Renders `start..end` of the open file through `chain` into a new
/// file in the app's cache folder, for the interface to swap in.
#[tauri::command]
pub async fn wave_apply_chain(
    app: AppHandle,
    wave: State<'_, OpenWave>,
    chain: EffectChain,
    start: u64,
    end: u64,
) -> Result<SpectralResult, String> {
    let session = snapshot(&wave)?;
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("wave-fx");
    spawn_blocking(move || -> Result<SpectralResult, String> {
        let rate = session.sample_rate;
        let total = mixdown::session_end(&session);
        let end = end.min(total);
        if end <= start {
            return Err("Select some audio first, or apply to the whole file.".into());
        }
        let Some(mut c) = ChainProcessor::new(&chain, rate) else {
            return Err(
                "The Voice Chain is empty or bypassed, so there is nothing to apply.".into(),
            );
        };
        let warm = (f64::from(rate) * WARM_SECONDS) as usize;
        let latency = c.latency();
        let from = start as i64 - warm as i64;
        let to = end as i64 + latency as i64;
        let audio = render(&session, from, to);
        let frames = (end - start) as usize;
        let edge = (f64::from(rate) * EDGE_SECONDS) as usize;
        let out = process_range(&mut c, &audio, warm, frames, edge);
        let media = write_stretch(
            &dir,
            "voice-chain",
            &out,
            0,
            frames,
            rate,
            is_mono(&session),
        )?;
        Ok(SpectralResult { media, start, end })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::model::EffectSlot;

    fn gain_chain(db: f64) -> EffectChain {
        let mut c = EffectChain::default();
        c.slots.push(EffectSlot {
            id: crate::session::model::new_id(),
            effect: "altaudio.gain".into(),
            enabled: true,
            params: [("gain".to_string(), db)].into_iter().collect(),
            preset: None,
        });
        c
    }

    #[test]
    fn processes_the_range_and_blends_at_the_edges() {
        let mut c = ChainProcessor::new(&gain_chain(-6.0206), 48_000).unwrap();
        let audio = vec![1.0f32; 300 * 2];
        let out = process_range(&mut c, &audio, 100, 100, 10);
        assert_eq!(out.len(), 200);
        assert!((out[0] - 1.0).abs() < 1e-6, "starts as it was");
        assert!((out[100] - 0.5).abs() < 1e-3, "middle is processed");
        assert!((out[199] - 1.0).abs() < 0.06, "ends close to as it was");
    }

    #[test]
    fn look_ahead_is_trimmed_so_audio_stays_in_place() {
        let mut lim = EffectChain::default();
        lim.slots.push(EffectSlot {
            id: crate::session::model::new_id(),
            effect: "altaudio.limiter".into(),
            enabled: true,
            params: [("ceiling".to_string(), 0.0), ("lookahead".to_string(), 5.0)]
                .into_iter()
                .collect(),
            preset: None,
        });
        let mut c = ChainProcessor::new(&lim, 48_000).unwrap();
        let lat = c.latency();
        assert!(lat > 0);
        // A click 50 frames into the range.
        let warm = 1_000;
        let frames = 2_000;
        let mut audio = vec![0f32; (warm + frames + lat) * 2];
        audio[(warm + 50) * 2] = 0.5;
        let out = process_range(&mut c, &audio, warm, frames, 0);
        let at = out.iter().step_by(2).position(|v| v.abs() > 0.4);
        assert_eq!(at, Some(50));
    }
}
