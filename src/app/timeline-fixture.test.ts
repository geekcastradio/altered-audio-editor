// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript timeline view rules must agree with
// shared-fixtures/timeline.json, the same file the Rust tests read
// (src-tauri/tests/timeline_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/timeline.json";
import * as T from "../panels/editor/timeline";

const vp = (v: { spp: number; scroll: number }, w: { spp: number; scroll: number }) => {
  expect(v.spp).toBeCloseTo(w.spp, 9);
  expect(v.scroll).toBeCloseTo(w.scroll, 9);
};

describe("shared timeline fixture", () => {
  it("view maths", () => {
    for (const c of fixture.xOf) expect(T.xOf(c.s, c.v)).toBeCloseTo(c.want, 9);
    for (const c of fixture.sampleAt) expect(T.sampleAt(c.x, c.v)).toBe(c.want);
    for (const c of fixture.edgeScroll) expect(T.edgeScrollPx(c.x, c.left, c.right, c.band)).toBeCloseTo(c.want, 9);
    for (const c of fixture.maxSpp) expect(T.maxSppFor(c.w, c.c, c.r)).toBeCloseTo(c.want, 9);
    for (const c of fixture.zoomAround) vp(T.zoomAround(c.v, c.factor, c.ax, c.maxSpp, c.minSpp), c.want);
    for (const c of fixture.fit) vp(T.fit(c.l, c.w, c.room), c.want);
    for (const c of fixture.overviewWindow) expect(T.overviewWindow(c.v, c.lw, c.total, c.sw)).toEqual(c.want);
    for (const c of fixture.dragEdge) vp(T.dragOverviewEdge(c.v, c.edge as "left" | "right", c.x, c.lw, c.total, c.sw), c.want);
    for (const c of fixture.trackColor) expect(T.defaultTrackColor(c.row)).toBe(c.want);
    for (const c of fixture.trackHeight) expect(T.trackHeight({ height: c.height as number, collapsed: c.collapsed as boolean })).toBe(c.want);
  });
  it("ruler and time text", () => {
    for (const c of fixture.rulerTicks) {
      const got = T.rulerTicks(c.v, c.w, c.rate, c.gap);
      expect(got.length).toBe(c.want.length);
      got.forEach((t, i) => {
        expect(t.x).toBeCloseTo(c.want[i]!.x, 9);
        expect(t.label).toBe(c.want[i]!.label);
      });
    }
    for (const c of fixture.formatTime) expect(T.formatTime(c.sec, c.step)).toBe(c.want);
    for (const c of fixture.formatClock) expect(T.formatClock(c.s, c.r)).toBe(c.want);
    for (const c of fixture.formatTimecode) expect(T.formatTimecode(c.s, c.r, c.fps)).toBe(c.want);
  });
  it("peaks", () => {
    for (const c of fixture.pickLevel) {
      const levels = c.blocks.map((block) => ({ block, data: new Int16Array(0) }));
      const got = T.pickLevel(levels, c.fpp);
      if (c.want === null) expect(got).toBeNull();
      else expect(got!.block).toBe(c.blocks[c.want]);
    }
    for (const c of fixture.peakRange) {
      const [lo, hi] = T.peakRange({ block: c.block, data: Int16Array.from(c.data) }, c.channels, c.ch, c.from, c.to);
      expect(lo).toBeCloseTo(c.want[0]!, 9);
      expect(hi).toBeCloseTo(c.want[1]!, 9);
    }
  });
});
