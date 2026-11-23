// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Small view helpers in Rust: scroll to reveal a position, session tabs, the
//! Video Reference bar, and the Waveform editor's active channels. Mirrors
//! `reveal.ts`, `session-tabs.ts`, `video-lane.ts` and the pure functions of
//! `wave-channels.ts` (under `src/panels/editor/`), tested against
//! `shared-fixtures/view-helpers.json`.

use crate::channel_rules::channel_labels;
use crate::js_compat::js_round;

/// Where to scroll so a position that jumped out of view is on screen. None
/// when it is already in view.
pub fn reveal_scroll(scroll: f64, span: f64, to: f64, at: f64) -> Option<f64> {
    if span.is_nan() || span <= 0.0 {
        return None;
    }
    if to >= scroll && to <= scroll + span {
        return None;
    }
    Some(js_round(to - span * at).max(0.0))
}

/// Tabs show once a second session is open.
pub fn show_tabs(rows: usize) -> bool {
    rows >= 2
}

/// The tab to switch to when the tab at `id` closes: its right neighbor, else
/// its left, else none. `ids` are the open tabs in order.
pub fn neighbor_of(ids: &[String], id: &str) -> Option<String> {
    let i = ids.iter().position(|x| x == id)?;
    if ids.len() < 2 {
        return None;
    }
    ids.get(i + 1)
        .or_else(|| i.checked_sub(1).and_then(|p| ids.get(p)))
        .cloned()
}

/// What a tab says: the session's name, with a star while it has unsaved changes.
pub fn tab_label(name: &str, dirty: bool) -> String {
    if dirty {
        format!("{name} *")
    } else {
        name.to_string()
    }
}

/// Narrowest Video Reference bar drawn.
pub const MIN_BAR: f64 = 24.0;

/// The bar for a video that starts at `offset` seconds and lasts `duration`
/// seconds (0 when unknown): `(left, width)` in pixels, or None when wholly
/// out of view.
pub fn lane_bar(
    offset: f64,
    duration: f64,
    spp: f64,
    scroll: f64,
    rate: f64,
    width_px: f64,
) -> Option<(f64, f64)> {
    if spp.is_nan() || spp <= 0.0 || rate.is_nan() || rate <= 0.0 {
        return None;
    }
    let start = (offset * rate - scroll) / spp;
    let end = if duration > 0.0 {
        start + (duration * rate) / spp
    } else {
        width_px
    };
    if end <= 0.0 || start >= width_px {
        return None;
    }
    let left = start.max(0.0);
    let right = width_px.min(end);
    Some((left, MIN_BAR.max(right - left)))
}

/// The new start time after dragging the bar `dx` pixels; never before the session start.
pub fn dragged_offset(offset: f64, dx: f64, spp: f64, rate: f64) -> f64 {
    let next = offset + (dx * spp) / rate;
    (js_round(next * 1000.0) / 1000.0).max(0.0)
}

/// The file name at the end of a path, either slash.
pub fn base_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// Every channel of an `n`-channel file on.
pub fn all_on(n: usize) -> Vec<bool> {
    vec![true; n.max(1)]
}

/// Switches channel `k` on or off. Switching off the only one left switches the others on.
pub fn toggle_channel(c: &[bool], k: usize) -> Vec<bool> {
    let next: Vec<bool> = c
        .iter()
        .enumerate()
        .map(|(i, &on)| if i == k { !on } else { on })
        .collect();
    if next.iter().any(|&b| b) {
        next
    } else {
        (0..c.len()).map(|i| i != k).collect()
    }
}

/// Channel `k` alone, or every channel again when it already is.
pub fn solo_channel(c: &[bool], k: usize) -> Vec<bool> {
    let only: Vec<bool> = (0..c.len()).map(|i| i == k).collect();
    if c == only.as_slice() {
        all_on(c.len())
    } else {
        only
    }
}

/// The one channel on, or None when more (or all) are.
pub fn one_channel(c: &[bool]) -> Option<usize> {
    let on: Vec<usize> = c
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(i, _)| i)
        .collect();
    (on.len() == 1 && c.len() > 1).then(|| on[0])
}

/// True when some channels are off, so edits reach only part of the file.
pub fn some_off(c: &[bool]) -> bool {
    c.len() > 1 && !c.iter().all(|&b| b)
}

/// Where one output channel comes from: the file before or after the edit, and its channel.
pub type ChannelTake = Vec<(bool, usize)>;

/// For an edit in place: channels that are on take the edited audio, the
/// others keep their own. None with all on.
pub fn edit_take(c: &[bool]) -> Option<ChannelTake> {
    if c.iter().all(|&b| b) {
        return None;
    }
    Some(c.iter().enumerate().map(|(k, &on)| (on, k)).collect())
}

/// For a paste over the audio: each channel that is on takes the pasted
/// audio's `from` channel (its own when the copy had every channel), the
/// others keep their own. None with all on and a full copy.
pub fn paste_take(c: &[bool], from: Option<usize>) -> Option<ChannelTake> {
    if c.iter().all(|&b| b) && from.is_none() {
        return None;
    }
    Some(
        c.iter()
            .enumerate()
            .map(|(k, &on)| {
                if on {
                    (true, from.unwrap_or(k))
                } else {
                    (false, k)
                }
            })
            .collect(),
    )
}

/// " (Left Channel)", " (C, LFE)" and so on for edit labels, or "" with all on.
pub fn channel_label(c: &[bool]) -> String {
    if c.iter().all(|&b| b) {
        return String::new();
    }
    if c.len() == 2 {
        return if c[0] {
            " (Left Channel)"
        } else {
            " (Right Channel)"
        }
        .to_string();
    }
    let names = channel_labels(c.len());
    let on: Vec<String> = c
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(k, _)| names[k].clone())
        .collect();
    format!(" ({})", on.join(", "))
}
