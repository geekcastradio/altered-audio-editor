// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { ClipDoc, SessionDoc, TrackDoc } from "../../stores/session";
import { isolatedClips, removeIsolated } from "./isolated-clips";

const fade = () => ({ length: 0, shape: "linear" as const, curve: 0 });
// One second at 48 kHz unless a length is given.
function clip(id: string, start: number, length = 48_000, extra: Partial<ClipDoc> = {}): ClipDoc {
  return { id, name: id, media_id: "m", start, source_in: 0, source_out: length, gain_db: 0, muted: false, locked: false, color: null, fade_in: fade(), fade_out: fade(), ...extra };
}
const track = (id: string, clips: ClipDoc[]): TrackDoc => ({ id, name: id, clips }) as unknown as TrackDoc;
function session(tracks: TrackDoc[]): SessionDoc {
  return {
    format: "altaudio.session",
    schema_version: 1,
    id: "s",
    name: "s",
    sample_rate: 48_000,
    bit_depth: 32,
    media: [{ id: "m", path: "a.wav", relative_path: null, sample_rate: 48_000, channels: 1, length: 4_800_000 }],
    tracks,
    markers: [],
    view: { samples_per_pixel: 100, scroll: 0, scroll_y: 0, playhead: 0, selection: null },
    libraries: [],
  } as unknown as SessionDoc;
}
// Short means under 0.8 s; isolated means 2 s of room each side.
const o = { maxLen: 38_400, gap: 96_000 };

describe("isolatedClips", () => {
  it("flags a short clip with room on both sides", () => {
    const doc = session([track("A", [clip("a1", 0, 480_000), clip("bump", 800_000, 10_000), clip("a2", 1_600_000, 480_000)])]);
    expect(isolatedClips(doc, o).map((x) => x.id)).toEqual(["bump"]);
  });

  it("ignores long clips and short ones that sit close to speech", () => {
    const doc = session([track("A", [clip("a1", 0, 480_000), clip("near", 500_000, 10_000), clip("a2", 1_600_000, 480_000), clip("long", 3_000_000, 100_000)])]);
    expect(isolatedClips(doc, o)).toEqual([]);
  });

  it("treats the first and last clips of a track as having open room outside", () => {
    const doc = session([track("A", [clip("first", 0, 10_000), clip("a1", 480_000, 480_000), clip("last", 2_000_000, 10_000)])]);
    expect(isolatedClips(doc, o).map((x) => x.id)).toEqual(["first", "last"]);
  });

  it("measures the gap from the latest end before it, not just the previous clip", () => {
    // A long clip starts early and runs past the short one's start on the same track.
    const doc = session([track("A", [clip("long", 0, 480_000), clip("inner", 100_000, 10_000)])]);
    expect(isolatedClips(doc, o)).toEqual([]);
  });

  it("notes when another track has audio under the clip", () => {
    const doc = session([track("A", [clip("blip", 800_000, 10_000)]), track("B", [clip("b1", 790_000, 100_000)])]);
    const r = isolatedClips(doc, o);
    expect(r.find((x) => x.id === "blip")?.overlapsOthers).toBe(true);
  });

  it("skips locked and muted clips", () => {
    const doc = session([track("A", [clip("l", 0, 10_000, { locked: true }), clip("m", 400_000, 10_000, { muted: true })])]);
    expect(isolatedClips(doc, o)).toEqual([]);
  });

  it("lists clips from every track in timeline order", () => {
    const doc = session([track("A", [clip("a", 900_000, 10_000)]), track("B", [clip("b", 100_000, 10_000)])]);
    expect(isolatedClips(doc, o).map((x) => x.id)).toEqual(["b", "a"]);
  });
});

describe("removeIsolated", () => {
  it("removes only the chosen clips and leaves locked ones", () => {
    const doc = session([track("A", [clip("a", 0), clip("b", 200_000), clip("c", 400_000, 48_000, { locked: true })])]);
    expect(removeIsolated(doc, ["a", "c"])).toEqual({ op: "remove_clips", ids: ["a"] });
  });

  it("returns null when nothing is left to remove", () => {
    const doc = session([track("A", [clip("a", 0, 48_000, { locked: true })])]);
    expect(removeIsolated(doc, ["a"])).toBeNull();
    expect(removeIsolated(doc, [])).toBeNull();
  });
});
