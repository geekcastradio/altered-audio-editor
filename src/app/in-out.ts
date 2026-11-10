// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** In and Out points are the two ends of the time selection. An end that was never set sits at the start or the end of the content. */
export type Span = { start: number; end: number } | null;

/** A span only counts when it has length. */
function span(start: number, end: number): Span {
  return end > start ? { start, end } : null;
}

/** I: the selection starts at `at`. With no Out, or an Out at or before `at`, it runs to the end of the content. */
export function markIn(sel: Span, at: number, contentEnd: number): Span {
  const from = Math.max(0, Math.round(at));
  const end = sel && sel.end > from ? sel.end : contentEnd;
  return span(from, end);
}

/** O: the selection ends at `at`. With no In, or an In at or after `at`, it starts at zero. */
export function markOut(sel: Span, at: number): Span {
  const to = Math.max(0, Math.round(at));
  const start = sel && sel.start < to ? sel.start : 0;
  return span(start, to);
}

/** Shift+I: forget the In point, so the selection starts at zero again. Nothing is left when that covers all the content. */
export function clearIn(sel: Span, contentEnd: number): Span {
  if (!sel) return null;
  return sel.end >= contentEnd ? null : span(0, sel.end);
}

/** Shift+O: forget the Out point, so the selection runs to the end of the content again. Nothing is left when that covers all the content. */
export function clearOut(sel: Span, contentEnd: number): Span {
  if (!sel) return null;
  return sel.start <= 0 ? null : span(sel.start, contentEnd);
}
