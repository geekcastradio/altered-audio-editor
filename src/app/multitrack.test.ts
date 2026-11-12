// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { describeMix, describeTalk, formatDuration, micTracks, mixOps, type MixAnalysis } from "./multitrack";
import type { SessionDoc } from "../stores/session";

const track = (id: string, extra: Record<string, unknown> = {}) => ({ id, name: id.toUpperCase(), clips: [{ id: `c-${id}` }], ...extra });
const doc = (tracks: unknown[]) => ({ sample_rate: 48_000, tracks, buses: [] }) as unknown as SessionDoc;

const analysis: MixAnalysis = {
  totalSeconds: 100,
  tracks: [
    { track: "a", name: "Ann", points: [{ position: 0, value: 0, curve: "linear" }, { position: 480, value: -12, curve: "linear" }], talkSeconds: 70, share: 0.7 },
    { track: "b", name: "Bo", points: [{ position: 0, value: -12, curve: "linear" }], talkSeconds: 30, share: 0.3 },
  ],
};

describe("multitrack", () => {
  it("offers audio tracks that have clips", () => {
    const d = doc([track("a"), track("m", { kind: "midi" }), { id: "e", name: "E", clips: [] }]);
    expect(micTracks(d).map((t) => t.id)).toEqual(["a"]);
  });

  it("formats durations", () => {
    expect(formatDuration(65)).toBe("1:05");
    expect(formatDuration(3725)).toBe("1:02:05");
    expect(formatDuration(-3)).toBe("0:00");
  });

  it("lists voices by talk time", () => {
    expect(describeTalk(analysis)).toEqual(["Ann: 1:10 (70%)", "Bo: 0:30 (30%)"]);
  });

  it("writes one batch with a volume lane per track and keeps other lanes", () => {
    const pan = { id: "p1", target: { type: "pan" }, enabled: true, points: [] };
    const d = doc([track("a", { envelopes: [pan] }), track("b")]);
    const op = mixOps(d, analysis);
    expect(op?.op).toBe("batch");
    const first = (op as { ops: { owner: string; envelopes: { target: { type: string }; points: unknown[] }[] }[] }).ops[0]!;
    expect(first.owner).toBe("a");
    expect(first.envelopes.map((e) => e.target.type)).toEqual(["pan", "volume"]);
    expect(first.envelopes[1]!.points).toHaveLength(2);
  });

  it("replaces an existing volume lane instead of adding a second", () => {
    const vol = { id: "v1", target: { type: "volume" }, enabled: false, points: [{ position: 5, value: -3, curve: "linear" }] };
    const d = doc([track("a", { envelopes: [vol] }), track("b")]);
    const op = mixOps(d, { ...analysis, tracks: [analysis.tracks[0]!] });
    expect(op?.op).toBe("set_envelopes");
    const envs = (op as { envelopes: { id: string; enabled: boolean; points: unknown[] }[] }).envelopes;
    expect(envs).toHaveLength(1);
    expect(envs[0]!.id).toBe("v1");
    expect(envs[0]!.enabled).toBe(true);
    expect(envs[0]!.points).toHaveLength(2);
  });

  it("does nothing when no track matches", () => {
    expect(mixOps(doc([track("z")]), analysis)).toBeNull();
    expect(describeMix(analysis)).toContain("2 tracks");
  });
});
