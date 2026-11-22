// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript Phase Align notice and response graph maths must agree with
// shared-fixtures/align-graph.json, the same file the Rust tests read
// (src-tauri/tests/align_graph_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/align-graph.json";
import { describeAlign } from "../panels/editor/align";
import { dbAt, freqAt, range, xOf, yOf } from "../panels/voice-chain/graph";

describe("shared align and graph fixture", () => {
  it.each(fixture.describeAlign)("describeAlign: $lag $score $rate $inverted", (c) => {
    expect(describeAlign({ lag: c.lag, score: c.score, inverted: c.inverted }, c.rate, "Guest.wav", "Track 2")).toBe(c.want);
  });
  it.each(fixture.alignedStart)("start after align: $clip $lag", (c) => {
    expect(Math.max(0, c.clip - c.lag)).toBe(c.want);
  });
  it.each(fixture.xOf)("xOf: $freq", (c) => expect(xOf(c.freq)).toBeCloseTo(c.want, 9));
  it.each(fixture.freqAt)("freqAt: $x", (c) => expect(freqAt(c.x)).toBeCloseTo(c.want, 7));
  it.each(fixture.yOf)("yOf: $db $top $bottom", (c) => expect(yOf(c.db, c.top, c.bottom)).toBeCloseTo(c.want, 9));
  it.each(fixture.dbAt)("dbAt: $y $top $bottom", (c) => expect(dbAt(c.y, c.top, c.bottom)).toBeCloseTo(c.want, 9));
  it.each(fixture.range)("range: $points $gains", (c) => {
    const info = { params: c.gains.map(([min, max]) => ({ unit: "dB", hidden: false, min, max })) } as never;
    expect(range(c.points.map((db) => [0, db] as [number, number]), info)).toEqual(c.want);
  });
});
