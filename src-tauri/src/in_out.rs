// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! In and Out points, in Rust.
//!
//! These mirror `src/app/in-out.ts`. Both are tested against
//! `shared-fixtures/in-out.json`, so the two copies cannot drift. When the
//! interface moves to Rust, the TypeScript copy goes away.
//! In and Out are the two ends of the time selection. An end that was never
//! set sits at the start or the end of the content. All positions are in samples.

use crate::selection::Selection;

/// A span only counts when it has length.
fn span(start: i64, end: i64) -> Option<Selection> {
    (end > start).then_some(Selection { start, end })
}

/// I: the selection starts at `at`. With no Out, or an Out at or before `at`,
/// it runs to the end of the content.
pub fn mark_in(sel: Option<Selection>, at: f64, content_end: i64) -> Option<Selection> {
    let from = (at.round() as i64).max(0);
    let end = sel.filter(|s| s.end > from).map_or(content_end, |s| s.end);
    span(from, end)
}

/// O: the selection ends at `at`. With no In, or an In at or after `at`, it
/// starts at zero.
pub fn mark_out(sel: Option<Selection>, at: f64) -> Option<Selection> {
    let to = (at.round() as i64).max(0);
    let start = sel.filter(|s| s.start < to).map_or(0, |s| s.start);
    span(start, to)
}

/// Shift+I: forget the In point, so the selection starts at zero again.
/// Nothing is left when that covers all the content.
pub fn clear_in(sel: Option<Selection>, content_end: i64) -> Option<Selection> {
    let s = sel?;
    if s.end >= content_end {
        None
    } else {
        span(0, s.end)
    }
}

/// Shift+O: forget the Out point, so the selection runs to the end of the
/// content again. Nothing is left when that covers all the content.
pub fn clear_out(sel: Option<Selection>, content_end: i64) -> Option<Selection> {
    let s = sel?;
    if s.start <= 0 {
        None
    } else {
        span(s.start, content_end)
    }
}
