// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the view helpers in TypeScript against the fixture the Rust test
// reads too (shared-fixtures/view-helpers.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/view-helpers.json";
import { revealScroll } from "./reveal";
import { neighborOf, showTabs, tabLabel } from "./session-tabs";
import { baseName, draggedOffset, laneBar } from "./video-lane";
import { allOn, channelLabel, editTake, oneChannel, pasteTake, someOff, soloChannel, toggleChannel } from "./wave-channels";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("view helpers fixture", () => {
  it("reveals and tabs", () => {
    for (const c of fx.reveal) expect(J(revealScroll(c.scroll, c.span, c.to, c.at))).toEqual(c.out);
    fx.rows.forEach((rows, i) => {
      const t = fx.tabs[i]!;
      expect(showTabs(rows as never)).toBe(t.show);
      expect(["a", "b", "c", "zz"].map((id) => J(neighborOf(rows as never, id)))).toEqual(t.nb);
      expect(rows.map((r) => tabLabel(r as never))).toEqual(t.labels);
    });
  });
  it("video bar", () => {
    for (const c of fx.bars) expect(J(laneBar(c.offset, c.duration, c.spp, c.scroll, c.rate, c.w))).toEqual(c.out);
    for (const c of fx.drag) expect(draggedOffset(c.a, c.b, c.c, c.d)).toBe(c.out);
    for (const c of fx.base) expect(baseName(c.p)).toBe(c.out);
  });
  it("channels", () => {
    for (const e of fx.ch) {
      const c = e.c;
      expect(allOn(c.length)).toEqual(e.allOn);
      expect(J(oneChannel(c))).toEqual(e.one);
      expect(someOff(c)).toBe(e.some);
      expect(J(editTake(c))).toEqual(e.edit);
      expect(channelLabel(c)).toBe(e.label);
      expect(c.map((_, k) => toggleChannel(c, k))).toEqual(e.toggles);
      expect(c.map((_, k) => soloChannel(c, k))).toEqual(e.solos);
      expect([undefined, 0, 1, 2].map((f) => J(pasteTake(c, f)))).toEqual(e.pastes);
    }
    expect(allOn(0)).toEqual(fx.allOn0);
  });
});
