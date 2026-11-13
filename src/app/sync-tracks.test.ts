// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { ClipDoc, SessionDoc, TrackDoc } from "../stores/session";
import { LockedError } from "../panels/editor/clip-edit";
import { describeSync, moveTrack, trust, type TrackSync } from "./sync-tracks";

const fade = () => ({ length: 0, shape: "linear" as const, curve: 0 });
const clip = (id: string, start: number, extra: Partial<ClipDoc> = {}): ClipDoc => ({ id, name: id, media_id: "m", start, source_in: 0, source_out: 48_000, gain_db: 0, muted: false, locked: false, color: null, fade_in: fade(), fade_out: fade(), ...extra });
const track = (id: string, clips: ClipDoc[], extra: object = {}): TrackDoc => ({ id, name: id, clips, ...extra }) as unknown as TrackDoc;
const session = (tracks: TrackDoc[]) => ({ sample_rate: 48_000, tracks, media: [], markers: [] }) as unknown as SessionDoc;
const result = (extra: Partial<TrackSync> = {}): TrackSync => ({ offset: 0, driftPpm: 0, confidence: 0.6, inverted: false, windows: 8, ...extra });

describe("moveTrack", () => {
  it("moves every clip earlier when the track is late", () => {
    const doc = session([track("B", [clip("b1", 100_000), clip("b2", 300_000)])]);
    expect(moveTrack(doc, "B", 60_000)).toEqual({
      op: "set_clips",
      changes: [
        { id: "b1", start: 40_000 },
        { id: "b2", start: 240_000 },
      ],
    });
  });

  it("moves them later when the track is early", () => {
    const doc = session([track("B", [clip("b1", 0)])]);
    expect(moveTrack(doc, "B", -48_000)).toEqual({ op: "set_clips", changes: [{ id: "b1", start: 48_000 }] });
  });

  it("will not push a clip before the start of the session", () => {
    const doc = session([track("B", [clip("b1", 10_000)])]);
    expect(moveTrack(doc, "B", 20_000)).toBe("before-start");
  });

  it("does nothing when already in line, and refuses MIDI and locked clips", () => {
    const doc = session([track("B", [clip("b1", 100_000)])]);
    expect(moveTrack(doc, "B", 0)).toBeNull();
    expect(moveTrack(session([track("M", [clip("m1", 100_000)], { midi_clips: [{}] })]), "M", 5_000)).toBe("blocked");
    expect(() => moveTrack(session([track("L", [clip("l1", 100_000, { locked: true })])]), "L", 5_000)).toThrow(LockedError);
  });
});

describe("describeSync", () => {
  it("says which way to move and by how much", () => {
    expect(describeSync(result({ offset: 65_760 }), 48_000)[0]).toContain("1.370 s late");
    expect(describeSync(result({ offset: -48_000 }), 48_000)[0]).toContain("1.000 s early");
    expect(describeSync(result(), 48_000)[0]).toBe("The tracks already line up.");
  });

  it("reports drift only when it matters", () => {
    expect(describeSync(result({ driftPpm: 2 }), 48_000)[1]).toContain("stay together");
    expect(describeSync(result({ driftPpm: 100 }), 48_000)[1]).toContain("360 ms every hour");
  });

  it("mentions an upside down track", () => {
    expect(describeSync(result({ inverted: true }), 48_000).join(" ")).toContain("upside down");
  });
});

describe("trust", () => {
  it("grades the confidence", () => {
    expect(trust(result({ confidence: 0.7 }))).toBe("strong");
    expect(trust(result({ confidence: 0.3 }))).toBe("fair");
    expect(trust(result({ confidence: 0.1 }))).toBe("weak");
  });
});
