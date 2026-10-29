// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { ClipDoc, SessionDoc, TrackDoc } from "../../stores/session";
import type { Located } from "../editor/clip-edit";
import { bandSlots, bandSplitOp, checkEdges, parseEdges } from "./band-split";

let n = 0;
const uuid = () => `id${++n}`;

const clip = (over: Partial<ClipDoc> = {}): ClipDoc =>
  ({
    id: "c1",
    name: "Voice",
    media_id: "m1",
    start: 100,
    source_in: 0,
    source_out: 5000,
    gain_db: -2,
    muted: false,
    locked: false,
    color: null,
    fade_in: { length: 0 },
    fade_out: { length: 0 },
    ...over,
  }) as ClipDoc;

const located = (c: ClipDoc, trackIndex = 1): Located =>
  ({ clip: c, track: { id: "t1", name: "Host", color: "#112233" } as TrackDoc, trackIndex }) as unknown as Located;

describe("edges", () => {
  it("reads numbers split by commas, spaces or semicolons", () => {
    expect(parseEdges("200, 2000")).toEqual([200, 2000]);
    expect(parseEdges("300hz")).toEqual([300]);
    expect(parseEdges("abc")).toBeNull();
    expect(parseEdges("  ")).toBeNull();
  });
  it("refuses edges that would not add back evenly", () => {
    expect(checkEdges([1000])).toBeNull();
    expect(checkEdges([200, 2000])).toBeNull();
    expect(checkEdges([200, 1200])).toBeNull();
    expect(checkEdges([200, 1000])).toMatch(/times apart/);
    expect(checkEdges([2000, 200])).toMatch(/lower/);
    expect(checkEdges([10])).toMatch(/between/);
    expect(checkEdges([100, 1000, 10000])).toMatch(/one frequency/);
    expect(checkEdges([])).toMatch(/one frequency/);
  });
});

describe("band slots", () => {
  it("each band gets a pair of Butterworth filters at every edge it touches", () => {
    const e = [200, 2000];
    const low = bandSlots(0, e, uuid).map((s) => [s.params.mode, s.params.freq]);
    const mid = bandSlots(1, e, uuid).map((s) => [s.params.mode, s.params.freq]);
    const high = bandSlots(2, e, uuid).map((s) => [s.params.mode, s.params.freq]);
    expect(low).toEqual([[0, 200], [0, 200]]);
    expect(mid).toEqual([[1, 200], [1, 200], [0, 2000], [0, 2000]]);
    expect(high).toEqual([[1, 2000], [1, 2000]]);
    expect(bandSlots(0, e, uuid).every((s) => s.params.design === 0 && s.params.order === 0)).toBe(true);
  });
});

describe("splitting a clip into bands", () => {
  it("adds one track per band with filters, copies the clip, and mutes the original", () => {
    const op = bandSplitOp({} as SessionDoc, [located(clip())], [200, 2000], uuid);
    expect(op?.op).toBe("batch");
    const ops = op && op.op === "batch" ? op.ops : [];
    expect(ops.filter((o) => o.op === "add_track").map((o) => (o as { name: string }).name)).toEqual(["Host Low", "Host Mid", "Host High"]);
    expect(ops.filter((o) => o.op === "set_effects")).toHaveLength(3);
    const added = ops.filter((o) => o.op === "add_clips");
    expect(added).toHaveLength(3);
    for (const a of added) expect((a as { clips: { start: number; source_out: number }[] }).clips[0]).toMatchObject({ start: 100, source_out: 5000 });
    const last = ops[ops.length - 1] as { op: string; changes: { id: string; muted?: boolean }[] };
    expect(last).toEqual({ op: "set_clips", changes: [{ id: "c1", muted: true }] });
  });
  it("leaves out reversed and locked clips, and bad edges", () => {
    expect(bandSplitOp({} as SessionDoc, [located(clip({ reversed: true }))], [1000], uuid)).toBeNull();
    expect(bandSplitOp({} as SessionDoc, [located(clip({ locked: true }))], [1000], uuid)).toBeNull();
    expect(bandSplitOp({} as SessionDoc, [located(clip())], [200, 400], uuid)).toBeNull();
  });
  it("places the new tracks right under each source track, shifting later ones", () => {
    const a = located(clip({ id: "a" }), 0);
    const b = located(clip({ id: "b" }), 3);
    const op = bandSplitOp({} as SessionDoc, [a, b], [1000], uuid);
    const idx = (op && op.op === "batch" ? op.ops : []).filter((o) => o.op === "add_track").map((o) => (o as { index: number }).index);
    expect(idx).toEqual([1, 2, 6, 7]);
  });
});
