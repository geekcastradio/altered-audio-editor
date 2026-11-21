// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Multitrack > Reduce Bleed: which clips can be cleaned, and the notice
//! afterwards. The Rust copy of `bleedWork` and `bleedSummary` in
//! `src/app/bleed.ts`, plus `mixdownBytes` from `src/app/session-properties.ts`;
//! all are tested against `shared-fixtures/bleed-notice.json`.

use crate::js_compat::js_to_fixed1;

/// Clips below this much change are left as they are.
pub const MIN_REMOVED_DB: f64 = 0.2;

/// What the filter needs to know about a clip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipFlags {
    pub locked: bool,
    pub looped: bool,
    pub stretched: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BleedWork {
    /// Indexes of the clips that can be cleaned.
    pub work: Vec<usize>,
    pub locked: usize,
    /// Looped or stretched clips, which the core cannot copy as they play.
    pub unsuitable: usize,
}

/// Splits a track's clips into the ones that can be cleaned and the ones left alone.
pub fn bleed_work(clips: &[ClipFlags]) -> BleedWork {
    let open: Vec<usize> = (0..clips.len()).filter(|&i| !clips[i].locked).collect();
    let work: Vec<usize> = open
        .iter()
        .copied()
        .filter(|&i| !clips[i].looped && !clips[i].stretched)
        .collect();
    BleedWork {
        locked: clips.len() - open.len(),
        unsuitable: open.len() - work.len(),
        work,
    }
}

fn clips_were(n: usize) -> &'static str {
    if n == 1 {
        "clip was"
    } else {
        "clips were"
    }
}

/// The notice after a run. `removed` holds each cleaned clip's change in dB.
pub fn bleed_summary(removed: &[f64], locked: usize, unsuitable: usize, changed: usize) -> String {
    let mut notes: Vec<String> = Vec::new();
    if locked > 0 {
        notes.push(format!("{locked} locked {} left alone", clips_were(locked)));
    }
    if unsuitable > 0 {
        notes.push(format!(
            "{unsuitable} looped or stretched {} left alone",
            clips_were(unsuitable)
        ));
    }
    let tail = if notes.is_empty() {
        String::new()
    } else {
        format!(" {}.", notes.join(", "))
    };
    if changed == 0 {
        return format!(
            "That mic picked up almost none of the other one, so nothing was changed.{tail}"
        );
    }
    let most = removed.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    format!(
        "Cleaned {changed} {}, up to {} dB quieter overall. One undo puts the originals back. Listen before you keep it.{tail}",
        if changed == 1 { "clip" } else { "clips" },
        js_to_fixed1(most)
    )
}

/// Bytes for a mixdown of `samples` at the session rate.
pub fn mixdown_bytes(samples: f64, bit_depth: f64, channels: f64) -> f64 {
    samples * channels * (bit_depth / 8.0)
}
