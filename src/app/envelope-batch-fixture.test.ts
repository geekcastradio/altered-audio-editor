// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript envelope, silence, stretch input and session tab rules must
// agree with shared-fixtures/envelope-batch.json, the same file the Rust tests
// read (src-tauri/tests/envelope_batch_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/envelope-batch.json";
import * as E from "../panels/editor/envelope";
import { neighborOf, showTabs, tabLabel } from "../panels/editor/session-tabs";
import { silenceSpan } from "../panels/editor/silence-span";
import { parseSeconds, percent } from "../panels/editor/stretch";

type Pts = { position: number; value: number; curve: "linear" | "hold" | "spline" }[];
type Kind = "volume" | "pan" | "fx";
const near = (got: Pts | null, want: unknown) => {
  if (want === null) return expect(got).toBeNull();
  const w = want as Pts;
  expect(got).not.toBeNull();
  expect(got!.length).toBe(w.length);
  got!.forEach((p, i) => {
    expect(p.position).toBeCloseTo(w[i]!.position, 9);
    expect(p.value).toBeCloseTo(w[i]!.value, 9);
    expect(p.curve).toBe(w[i]!.curve);
  });
};

describe("shared envelope batch fixture", () => {
  it("value maths", () => {
    for (const c of fixture.valueToY) expect(E.valueToY("volume", c.v, c.top, c.h, c.range)).toBeCloseTo(c.want, 9);
    for (const c of fixture.yToValue) expect(E.yToValue("volume", c.y, c.top, c.h, c.range)).toBeCloseTo(c.want, 9);
    for (const c of fixture.snap) expect(E.snapValue(c.kind as Kind, c.v)).toBeCloseTo(c.want, 9);
    for (const c of fixture.clamp) expect(E.clampValue("volume", c.v, c.range)).toBeCloseTo(c.want, 9);
    for (const c of fixture.format) expect(E.formatValue(c.kind as Kind, c.v, c.unit)).toBe(c.want);
    for (const c of fixture.dbToY) expect(E.dbToY(c.db, c.top, c.h)).toBeCloseTo(c.want, 9);
    for (const c of fixture.yToDb) expect(E.yToDb(c.y, c.top, c.h)).toBeCloseTo(c.want, 9);
    for (const c of fixture.chipLabel) expect(E.chipLabel(c.v, c.p, c.f)).toBe(c.want);
    for (const c of fixture.chipBox) expect(E.chipBox(c.x0, c.x1, c.label)).toEqual(c.want);
  });
  it("point edits", () => {
    for (const c of fixture.levelAt) expect(E.levelAt(c.points as Pts, c.pos)).toBeCloseTo(c.want, 9);
    for (const c of fixture.slice) near(E.sliceVolume(c.points as Pts, c.from, c.to), c.want);
    for (const c of fixture.addPoint) {
      const r = E.addPoint(c.points as Pts, c.pos, c.value);
      near(r.points, c.want.points);
      expect(r.index).toBe(c.want.index);
    }
    for (const c of fixture.movePoint) near(E.movePoint(c.points as Pts, c.i, c.pos, c.value, c.len, "volume", c.range), c.want);
    for (const c of fixture.movePoints) {
      const how = c.how as { set?: number; add?: number };
      const valueFor = (p: { value: number }) => (how.set !== undefined ? how.set : p.value + (how.add ?? 0));
      near(E.movePoints(c.points as Pts, new Set(c.sel), c.dPos, valueFor, c.len, "volume", c.range), c.want);
    }
    for (const c of fixture.silence) near(silenceSpan(c.points as Pts, c.len, c.from, c.to, c.ramp), c.want);
  });
  it("stretch input and tabs", () => {
    for (const c of fixture.percent) expect(percent(c.ratio)).toBe(c.want);
    for (const c of fixture.parseSeconds) expect(parseSeconds(c.text)).toEqual(c.want);
    const row = (id: string) => ({ id, name: id, dirty: false, seconds: 0, sampleRate: 48000, active: false, front: false });
    for (const c of fixture.tabs) {
      const rows = c.ids.map(row);
      expect(showTabs(rows)).toBe(c.show);
      for (const [id, want] of c.neighbors) expect(neighborOf(rows, id as string)).toBe(want);
    }
    for (const c of fixture.tabLabels) expect(tabLabel({ ...row("x"), name: c.name, dirty: c.dirty })).toBe(c.want);
  });
});
