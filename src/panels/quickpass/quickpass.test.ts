// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { BUILTIN_PRESETS, cutRange, DEFAULTS, findSilences, sanitizeSettings, shiftFindings, silentRuns, suggestLevels, type Finding } from "./quickpass";

const W = 0.1; // seconds per window
const s = { ...DEFAULTS, quietDb: -45, contentDb: -35, minQuietSec: 0.3, minContentSec: 0.2 };
const quiet = (n: number) => Array<number>(n).fill(-60);
const loud = (n: number) => Array<number>(n).fill(-20);

describe("silentRuns", () => {
  it("finds quiet stretches that last long enough", () => {
    expect(silentRuns([...loud(5), ...quiet(4), ...loud(5)], W, s)).toEqual([[5, 9]]);
  });
  it("ignores quiet that is too short", () => {
    expect(silentRuns([...loud(5), ...quiet(2), ...loud(5)], W, s)).toEqual([]);
  });
  it("keeps a silence going through a very short burst of sound", () => {
    expect(silentRuns([...quiet(4), ...loud(1), ...quiet(4)], W, s)).toEqual([[0, 9]]);
  });
  it("splits at speech that lasts long enough", () => {
    expect(silentRuns([...quiet(4), ...loud(3), ...quiet(4)], W, s)).toEqual([
      [0, 4],
      [7, 11],
    ]);
  });
  it("a silence at the very end counts", () => {
    expect(silentRuns([...loud(3), ...quiet(5)], W, s)).toEqual([[3, 8]]);
  });
  it("sound between the two levels does not start a silence", () => {
    expect(silentRuns(Array<number>(10).fill(-40), W, s)).toEqual([]);
  });
});

describe("findSilences", () => {
  it("places findings on the timeline", () => {
    const f = findSilences(
      [{ clipId: "c", start: 1000, window: 100, db: [...loud(2), ...quiet(5), ...loud(1)] }],
      new Map([["c", { trackId: "t", name: "Host" }]]),
      s,
      1000,
    );
    expect(f).toEqual([{ id: "c:1200", trackId: "t", clipName: "Host", start: 1200, end: 1700 }]);
  });
});

describe("suggestLevels", () => {
  it("sets quiet above the noise floor and below speech", () => {
    const db = [...Array<number>(30).fill(-62), ...Array<number>(70).fill(-22)];
    const r = suggestLevels(db)!;
    expect(r.quietDb).toBeGreaterThan(-62);
    expect(r.contentDb).toBeLessThan(-22);
    expect(r.contentDb).toBeGreaterThanOrEqual(r.quietDb);
  });
  it("gives up on too little or flat audio", () => {
    expect(suggestLevels([-30, -30])).toBeNull();
    expect(suggestLevels(Array<number>(100).fill(-30))).toBeNull();
  });
});

describe("sanitizeSettings", () => {
  it("fills in and clamps", () => {
    expect(sanitizeSettings(null)).toEqual(DEFAULTS);
    expect(sanitizeSettings({ action: "bogus", keepMs: -5 })).toMatchObject({ action: "remove", keepMs: 0 });
    expect(sanitizeSettings({ quietDb: -30, contentDb: -50, ripple: "all" })).toMatchObject({ quietDb: -30, contentDb: -30, ripple: "all" });
  });
});

describe("shiftFindings", () => {
  const f = (id: string, trackId: string, start: number, end: number): Finding => ({ id, trackId, clipName: "", start, end });
  it("moves later ones left on the same track only", () => {
    const gone = f("a", "t1", 100, 200);
    const out = shiftFindings([gone, f("b", "t1", 300, 400), f("c", "t2", 300, 400)], gone, "track");
    expect(out.map((x) => [x.id, x.start])).toEqual([
      ["b", 200],
      ["c", 300],
    ]);
  });
  it("moves every track when the gap closes on all", () => {
    const gone = f("a", "t1", 100, 200);
    expect(shiftFindings([f("c", "t2", 300, 400)], gone, "all")[0]!.start).toBe(200);
  });
});

import { applyLocal, clipsOn, markersFor, removeOp, splitOp } from "./quickpass";
import type { ClipDoc, SessionDoc, TrackDoc } from "../../stores/session";

const fd = () => ({ length: 0, shape: "linear" as const, curve: 0 });
const mkClip = (id: string, start: number, out = 48_000): ClipDoc => ({ id, name: id, media_id: "m", start, source_in: 0, source_out: out, gain_db: 0, muted: false, locked: false, color: null, fade_in: fd(), fade_out: fd() });
const mkDoc = (clips: ClipDoc[]): SessionDoc =>
  ({
    format: "altaudio.session",
    schema_version: 1,
    id: "s",
    name: "s",
    sample_rate: 48_000,
    bit_depth: 32,
    media: [{ id: "m", path: "a.wav", relative_path: null, sample_rate: 48_000, channels: 1, length: 480_000 }],
    tracks: [{ id: "t", name: "t", clips } as unknown as TrackDoc, { id: "u", name: "u", clips: [mkClip("bed", 0, 48_000)] } as unknown as TrackDoc],
    markers: [],
    view: { samples_per_pixel: 100, scroll: 0, scroll_y: 0, playhead: 0, selection: null },
    libraries: [],
  }) as unknown as SessionDoc;
let n = 0;
const nid = () => `n${++n}`;
const span = (d: SessionDoc, id: string) => {
  const c = d.tracks.flatMap((t) => t.clips).find((x) => x.id === id)!;
  return [c.start, c.source_out - c.source_in] as const;
};
const fnd = (start: number, end: number) => ({ id: `f${start}`, trackId: "t", clipName: "a", start, end });

describe("acting on findings", () => {
  it("removes two silences from one clip in a single edit", () => {
    const doc = mkDoc([mkClip("a", 0, 48_000)]);
    const op = removeOp(doc, [fnd(6_000, 12_000), fnd(24_000, 30_000)], { ripple: "track", keepMs: 0 }, nid)!;
    const after = applyLocal(doc, op);
    const t = after.tracks[0]!;
    const total = t.clips.reduce((s, c) => s + (c.source_out - c.source_in), 0);
    expect(total).toBe(48_000 - 12_000);
    expect(span(after, "bed")).toEqual([0, 48_000]);
    const starts = t.clips.map((c) => c.start).sort((a, b) => a - b);
    expect(starts).toEqual([0, 6_000, 18_000]);
  });

  it("splits out a silence and finds the piece again", () => {
    const doc = mkDoc([mkClip("a", 0, 48_000)]);
    const f = fnd(10_000, 20_000);
    const after = applyLocal(doc, splitOp(doc, [f], nid)!);
    expect(after.tracks[0]!.clips).toHaveLength(3);
    expect(clipsOn(after, [f])).toHaveLength(1);
  });

  it("names markers after the ones already there", () => {
    const doc = mkDoc([]);
    const m = markersFor({ ...doc, markers: [{ ...markersFor(doc, [fnd(0, 10)], nid)[0]! }] }, [fnd(20, 30), fnd(40, 50)], nid);
    expect(m.map((x) => x.name)).toEqual(["Silence 2", "Silence 3"]);
    expect(m[0]!.duration).toBe(10);
  });
});

describe("keeping some pause", () => {
  it("cuts only the middle of a silence", () => {
    expect(cutRange({ start: 0, end: 48_000 }, 100, 48_000)).toEqual([2_400, 45_600]);
  });
  it("leaves a silence that is shorter than the pause alone", () => {
    expect(cutRange({ start: 0, end: 4_000 }, 100, 48_000)).toBeNull();
  });
  it("removing with a kept pause shortens the silence", () => {
    const doc = mkDoc([mkClip("a", 0, 48_000)]);
    const op = removeOp(doc, [fnd(10_000, 20_000)], { ripple: "track", keepMs: 100 }, nid)!;
    const t = applyLocal(doc, op).tracks[0]!;
    expect(t.clips.reduce((n, c) => n + (c.source_out - c.source_in), 0)).toBe(48_000 - (10_000 - 4_800));
  });
});

describe("presets", () => {
  it("are all valid settings with unique names", () => {
    const names = BUILTIN_PRESETS.map((p) => p.name);
    expect(new Set(names).size).toBe(names.length);
    for (const p of BUILTIN_PRESETS) expect(sanitizeSettings(p.settings)).toEqual(p.settings);
  });
});
