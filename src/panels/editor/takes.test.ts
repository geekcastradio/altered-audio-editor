// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { ClipDoc, Op, SessionDoc, TrackDoc } from "../../stores/session";
import { deleteTakeOps, flattenOps, isOffTake, makeTakesOps, nextTakeNumber, pickRangeOps, silentGaps, takeSets } from "./takes";

const fade = () => ({ length: 0, shape: "linear" as const, curve: 0 });

function clip(id: string, start: number, len = 48_000, take?: { set: string; number: number; active: boolean }): ClipDoc {
  return { id, name: id, media_id: "m48", start, source_in: 0, source_out: len, gain_db: 0, muted: false, locked: false, color: null, fade_in: fade(), fade_out: fade(), ...(take ? { take } : {}) };
}

function session(clips: ClipDoc[]): SessionDoc {
  return {
    format: "altaudio.session",
    schema_version: 1,
    id: "s",
    name: "s",
    sample_rate: 48_000,
    bit_depth: 32,
    media: [{ id: "m48", path: "a.wav", relative_path: null, sample_rate: 48_000, channels: 1, length: 480_000 }],
    tracks: [{ id: "T", name: "T", clips } as unknown as TrackDoc],
    markers: [],
    view: { samples_per_pixel: 100, scroll: 0, scroll_y: 0, playhead: 0, selection: null },
    libraries: [],
  };
}

let n = 0;
const newId = () => `n${++n}`;

describe("takeSets", () => {
  it("groups clips by set into lanes in take order", () => {
    const doc = session([clip("b", 0, 48_000, { set: "S", number: 2, active: false }), clip("a", 0, 48_000, { set: "S", number: 1, active: true }), clip("x", 100_000)]);
    const sets = takeSets(doc);
    expect(sets).toHaveLength(1);
    expect(sets[0]!.lanes.map((l) => l.number)).toEqual([1, 2]);
    expect([sets[0]!.start, sets[0]!.end]).toEqual([0, 48_000]);
    expect(nextTakeNumber(doc, "S")).toBe(3);
    expect(isOffTake(doc.tracks[0]!.clips[0]!)).toBe(true);
  });
});

describe("makeTakesOps", () => {
  it("numbers by start and plays only the first of overlapping clips", () => {
    const doc = session([clip("b", 1_000), clip("a", 0), clip("c", 200_000)]);
    const op = makeTakesOps(doc, ["a", "b", "c"], () => "S") as Extract<Op, { op: "set_clips" }>;
    expect(op.changes.map((c) => [c.id, c.take])).toEqual([
      ["a", { set: "S", number: 1, active: true }],
      ["b", { set: "S", number: 2, active: false }],
      ["c", { set: "S", number: 3, active: true }],
    ]);
  });
  it("needs two free clips", () => {
    expect(makeTakesOps(session([clip("a", 0)]), ["a"], () => "S")).toBeNull();
    expect(makeTakesOps(session([clip("a", 0, 48_000, { set: "S", number: 1, active: true }), clip("b", 0)]), ["a", "b"], () => "T")).toBeNull();
  });
});

describe("pickRangeOps", () => {
  const doc = session([clip("a", 0, 48_000, { set: "S", number: 1, active: true }), clip("b", 0, 48_000, { set: "S", number: 2, active: false })]);
  it("splits both takes at the range and switches the middle", () => {
    const ops = pickRangeOps(doc, "S", 2, 12_000, 24_000, newId);
    const pieces = ops.flatMap((o) => (o.op === "restore_clips" ? o.clips : []));
    expect(pieces).toHaveLength(6);
    const set = ops[ops.length - 1] as Extract<Op, { op: "set_clips" }>;
    expect(set.op).toBe("set_clips");
    const on = set.changes.filter((c) => c.take!.active);
    const off = set.changes.filter((c) => !c.take!.active);
    expect(on.map((c) => c.take!.number)).toEqual([2]);
    expect(off.map((c) => c.take!.number)).toEqual([1]);
  });
  it("cuts the range back to what the chosen take covers", () => {
    const short = session([clip("a", 0, 48_000, { set: "S", number: 1, active: true }), clip("b", 0, 6_000, { set: "S", number: 2, active: false })]);
    const ops = pickRangeOps(short, "S", 2, 0, 24_000, newId);
    // Only take 1 is cut, at 6000; take 1 keeps playing after that.
    const ends = ops.flatMap((o) => (o.op === "restore_clips" ? o.clips : [])).map(([, c]) => [c.start, c.source_out]);
    expect(ends).toEqual([[0, 6_000], [6_000, 48_000]]);
    const set = ops[ops.length - 1] as Extract<Op, { op: "set_clips" }>;
    expect(set.changes.map((c) => [c.take!.number, c.take!.active]).sort()).toEqual([[1, false], [2, true]]);
  });
  it("does nothing for an empty range or unknown set", () => {
    expect(pickRangeOps(doc, "S", 1, 10, 10, newId)).toEqual([]);
    expect(pickRangeOps(doc, "nope", 1, 0, 10, newId)).toEqual([]);
  });
});

describe("flatten and delete", () => {
  const doc = session([clip("a", 0, 48_000, { set: "S", number: 1, active: true }), clip("b", 0, 48_000, { set: "S", number: 2, active: false })]);
  it("drops silent pieces and clears the mark on the rest", () => {
    expect(flattenOps(doc, "S")).toEqual([
      { op: "remove_clips", ids: ["b"] },
      { op: "set_clips", changes: [{ id: "a", clear_take: true }] },
    ]);
  });
  it("deletes one take", () => {
    expect(deleteTakeOps(doc, "S", 2)).toEqual([{ op: "remove_clips", ids: ["b"] }]);
    expect(deleteTakeOps(doc, "S", 9)).toEqual([]);
  });
  it("finds stretches where nothing plays", () => {
    const gap = session([clip("a", 0, 12_000, { set: "S", number: 1, active: true }), clip("b", 0, 48_000, { set: "S", number: 2, active: false })]);
    expect(silentGaps(gap, "S")).toEqual([[12_000, 48_000]]);
    expect(silentGaps(doc, "S")).toEqual([]);
  });
});
