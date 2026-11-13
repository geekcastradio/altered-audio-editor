// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { ClipDoc, MarkerDoc, SessionDoc, TrackDoc } from "../../stores/session";
import { LockedError } from "../editor/clip-edit";
import { closeDeadSpace, deadSpaces } from "./dead-space";

const fade = () => ({ length: 0, shape: "linear" as const, curve: 0 });
// One second of audio at 48 kHz unless a length is given.
function clip(id: string, start: number, length = 48_000): ClipDoc {
  return { id, name: id, media_id: "m", start, source_in: 0, source_out: length, gain_db: 0, muted: false, locked: false, color: null, fade_in: fade(), fade_out: fade() };
}
const track = (id: string, clips: ClipDoc[], extra: object = {}): TrackDoc => ({ id, name: id, clips, ...extra }) as unknown as TrackDoc;
function session(tracks: TrackDoc[], markers: MarkerDoc[] = []): SessionDoc {
  return {
    format: "altaudio.session",
    schema_version: 1,
    id: "s",
    name: "s",
    sample_rate: 48_000,
    bit_depth: 32,
    media: [{ id: "m", path: "a.wav", relative_path: null, sample_rate: 48_000, channels: 1, length: 4_800_000 }],
    tracks,
    markers,
    view: { samples_per_pixel: 100, scroll: 0, scroll_y: 0, playhead: 0, selection: null },
    libraries: [],
  } as unknown as SessionDoc;
}
const marker = (id: string, position: number): MarkerDoc => ({ id, name: id, kind: "cue", position, duration: 0, description: "", chapter: null, contracted_duration: null }) as unknown as MarkerDoc;
const o = { pause: 24_000, minRemove: 4_800 };

describe("deadSpaces", () => {
  it("finds only the time where no track has a clip", () => {
    // A speaks 0 to 1 s, B 3 to 4 s, A again 3.5 to 4.5 s, B 10 to 11 s.
    const doc = session([track("A", [clip("a1", 0), clip("a2", 168_000)]), track("B", [clip("b1", 144_000), clip("b2", 480_000)])]);
    expect(deadSpaces(doc, ["A", "B"], o)).toEqual([
      { start: 48_000, end: 144_000, remove: 72_000 },
      { start: 216_000, end: 480_000, remove: 240_000 },
    ]);
  });

  it("leaves a gap alone when another track speaks in it", () => {
    const doc = session([track("A", [clip("a1", 0), clip("a2", 480_000)]), track("B", [clip("b1", 40_000, 460_000)])]);
    expect(deadSpaces(doc, ["A", "B"], o)).toEqual([]);
  });

  it("keeps pauses that are already short, and ignores the time before the first clip", () => {
    const doc = session([track("A", [clip("a1", 96_000), clip("a2", 148_000)])]);
    // The gap is 4,000 samples, shorter than the pause; the 96,000 lead-in is not touched.
    expect(deadSpaces(doc, ["A"], o)).toEqual([]);
  });

  it("skips stretches that would give back less than the minimum", () => {
    const doc = session([track("A", [clip("a1", 0), clip("a2", 48_000 + 24_000 + 3_000)])]);
    expect(deadSpaces(doc, ["A"], o)).toEqual([]);
    expect(deadSpaces(doc, ["A"], { pause: 24_000, minRemove: 1_000 })).toHaveLength(1);
  });

  it("counts muted clips as occupying time", () => {
    const doc = session([track("A", [clip("a1", 0), { ...clip("a2", 100_000), muted: true }, clip("a3", 400_000)])]);
    expect(deadSpaces(doc, ["A"], o)).toEqual([
      { start: 48_000, end: 100_000, remove: 28_000 },
      { start: 148_000, end: 400_000, remove: 228_000 },
    ]);
  });
});

describe("closeDeadSpace", () => {
  const doc = session(
    [track("A", [clip("a1", 0), clip("a2", 168_000)]), track("B", [clip("b1", 144_000), clip("b2", 480_000)])],
    [marker("early", 10_000), marker("inside", 100_000), marker("late", 500_000)],
  );
  type Ops = { ops: { op: string; changes: { id: string; start?: number; position?: number }[] }[] };

  it("moves every clip after a closed stretch left by what was taken out, on every track together", () => {
    const r = closeDeadSpace(doc, ["A", "B"], o);
    expect(r).not.toBeNull();
    expect(r).not.toBe("blocked");
    const c = r as Exclude<typeof r, "blocked" | null>;
    // The first stretch gives back 72,000 samples and the second 240,000.
    expect(c.removed).toBe(312_000);
    expect((c.op as unknown as Ops).ops[0]!.changes).toEqual([
      { id: "a2", start: 96_000 },
      { id: "b1", start: 72_000 },
      { id: "b2", start: 168_000 },
    ]);
    expect(c.moved).toBe(3);
  });

  it("keeps the pause between the last clip before a closed stretch and the first one after", () => {
    const c = closeDeadSpace(doc, ["A", "B"], o) as { op: unknown };
    const b1 = (c.op as Ops).ops[0]!.changes.find((x) => x.id === "b1")!;
    // a1 ends at 48,000 and does not move, so b1 sits one pause after it.
    expect(b1.start! - 48_000).toBe(o.pause);
  });

  it("moves markers with the audio and pulls one inside a closed stretch to where the pause ends", () => {
    const c = closeDeadSpace(doc, ["A", "B"], o) as { op: unknown };
    const marks = (c.op as Ops).ops.find((x) => x.op === "set_markers")!.changes;
    // The first stretch runs 48,000 to 144,000 and keeps a 24,000 pause, so it is cut from 72,000.
    expect(marks.find((m) => m.id === "inside")).toEqual({ id: "inside", position: 72_000 });
    expect(marks.find((m) => m.id === "late")).toEqual({ id: "late", position: 500_000 - 312_000 });
    expect(marks.find((m) => m.id === "early")).toBeUndefined();
  });

  it("does nothing when there is nothing to close", () => {
    expect(closeDeadSpace(session([track("A", [clip("a", 0), clip("b", 50_000)])]), ["A"], o)).toBeNull();
  });

  it("is blocked by a MIDI track, which a ripple does not move", () => {
    const d = session([track("A", [clip("a", 0), clip("b", 480_000)]), track("M", [], { midi_clips: [{ id: "x" }] })]);
    expect(closeDeadSpace(d, ["A", "M"], o)).toBe("blocked");
  });

  it("refuses when a locked clip would have to move", () => {
    const d = session([track("A", [clip("a", 0), { ...clip("b", 480_000), locked: true }])]);
    expect(() => closeDeadSpace(d, ["A"], o)).toThrow(LockedError);
  });

  it("only touches the tracks it is given", () => {
    const d = session([track("A", [clip("a", 0), clip("b", 480_000)]), track("B", [clip("c", 0), clip("d", 480_000)])]);
    const c = closeDeadSpace(d, ["A"], o) as { op: { changes: { id: string }[] } };
    expect(c.op.changes.map((x) => x.id)).toEqual(["b"]);
  });
});
