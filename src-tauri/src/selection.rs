// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Selection and playhead rules, in Rust.
//!
//! These mirror `src/app/playhead-keys.ts` and `src/stores/transport.ts`.
//! Both are tested against `shared-fixtures/selection.json`, so the two
//! copies cannot drift. When the interface moves to Rust, the TypeScript copy goes away.
//! All positions are in samples.

/// A time selection, `start` inclusive and `end` exclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub start: i64,
    pub end: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadMove {
    StepLeft,
    StepRight,
    PageLeft,
    PageRight,
    ToIn,
    ToOut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeMove {
    StartLeft,
    StartRight,
    EndLeft,
    EndRight,
}

/// Rounds half away from zero, like JavaScript's `Math.round` for the positive values used here.
fn round(n: f64) -> i64 {
    (n + 0.5).floor() as i64
}

fn clamp(n: i64, end: i64) -> i64 {
    n.min(end.max(0)).max(0)
}

/// Where the playhead goes, or `None` when the move has nowhere to go.
pub fn head_target(
    mv: HeadMove,
    pos: i64,
    end: i64,
    spp: f64,
    width: f64,
    sel: Option<Selection>,
) -> Option<i64> {
    let step = round(spp).max(1);
    let page = step.max(round(width.max(1.0) * spp));
    let has_sel = sel.filter(|s| s.end > s.start);
    Some(match mv {
        HeadMove::StepLeft => clamp(pos - step, end),
        HeadMove::StepRight => clamp(pos + step, end),
        HeadMove::PageLeft => clamp(pos - page, end),
        HeadMove::PageRight => clamp(pos + page, end),
        HeadMove::ToIn => clamp(has_sel?.start, end),
        HeadMove::ToOut => clamp(has_sel?.end, end),
    })
}

/// Moves one border of the selection by `step` samples, keeping at least one
/// sample selected and staying inside `0..end`. `None` when there is no selection.
pub fn adjust_selection(
    sel: Option<Selection>,
    mv: EdgeMove,
    step: f64,
    end: f64,
) -> Option<Selection> {
    let sel = sel.filter(|s| s.end > s.start)?;
    let n = round(step).max(1);
    let top = round(end).max(1);
    Some(match mv {
        EdgeMove::StartLeft => Selection {
            start: clamp(sel.start - n, top),
            end: sel.end,
        },
        EdgeMove::StartRight => Selection {
            start: clamp(sel.start + n, top).min(sel.end - 1),
            end: sel.end,
        },
        EdgeMove::EndLeft => Selection {
            start: sel.start,
            end: clamp(sel.end - n, top).max(sel.start + 1),
        },
        EdgeMove::EndRight => Selection {
            start: sel.start,
            end: (sel.start + 1).max(clamp(sel.end + n, top)),
        },
    })
}

/// Where Play starts: the playhead, or the start of the selection when the
/// playhead is outside it (unless playback skips the selection).
pub fn play_start(playhead: i64, sel: Option<Selection>, skips: bool) -> i64 {
    match sel {
        Some(s) if !skips && s.end > s.start => {
            if playhead >= s.start && playhead < s.end {
                playhead
            } else {
                s.start
            }
        }
        _ => playhead,
    }
}

/// What the stop-at-selection rule looks at.
#[derive(Clone, Copy, Debug)]
pub struct StopAtSelection {
    pub enabled: bool,
    pub backend_stops_at_end: bool,
    pub skips: bool,
    pub started_at: i64,
    pub position: i64,
    pub sel: Selection,
}

/// True when playback that started inside the selection has reached its end.
pub fn stop_at_selection_due(f: StopAtSelection) -> bool {
    f.enabled
        && f.sel.end > f.sel.start
        && f.backend_stops_at_end
        && !f.skips
        && f.started_at >= f.sel.start
        && f.started_at < f.sel.end
        && f.position >= f.sel.end
}
