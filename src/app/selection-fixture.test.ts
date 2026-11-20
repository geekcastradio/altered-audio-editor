// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript selection rules must agree with shared-fixtures/selection.json,
// the same file the Rust tests read (src-tauri/tests/selection_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/selection.json";
import { playStart, stopAtSelectionDue } from "../stores/transport";
import { adjustSelection, headTarget, type EdgeMove, type HeadMove } from "./playhead-keys";

type Pair = [number, number] | null;
const sel = (p: Pair) => (p ? { start: p[0], end: p[1] } : null);
const out = (s: { start: number; end: number } | null) => (s ? [s.start, s.end] : null);

describe("shared selection fixture", () => {
  it.each(fixture.adjustSelection)("adjustSelection: $name", (c) => {
    expect(out(adjustSelection(sel(c.sel as Pair), c.move as EdgeMove, c.step, c.end))).toEqual(c.want);
  });
  it.each(fixture.headTarget)("headTarget: $name", (c) => {
    expect(headTarget(c.move as HeadMove, { pos: c.pos, end: c.end, spp: c.spp, width: c.width, sel: sel(c.sel as Pair) })).toEqual(c.want);
  });
  it.each(fixture.playStart)("playStart: $name", (c) => {
    expect(playStart(c.playhead, sel(c.sel as Pair), c.skips)).toBe(c.want);
  });
  it.each(fixture.stopAtSelectionDue)("stopAtSelectionDue: $name", (c) => {
    expect(
      stopAtSelectionDue({
        enabled: c.enabled, backendStopsAtEnd: c.backendStopsAtEnd, skips: c.skips,
        startedAt: c.startedAt, position: c.position, sel: sel(c.sel as Pair)!,
      }),
    ).toBe(c.want);
  });
});
