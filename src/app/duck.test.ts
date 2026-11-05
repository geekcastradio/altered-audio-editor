// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { DUCK_DEPTHS, duckKeys, duckParams, duckPlan, hasDuck, isDuckSlot, withDuck, withoutDuck } from "./duck";
import type { EffectChainDoc, SessionDoc, TrackDoc } from "../stores/session";

const track = (id: string, over: Partial<TrackDoc> = {}) =>
  ({ id, name: id, muted: false, clips: [{ id: `${id}-c`, media_id: "m" }], ...over }) as unknown as TrackDoc;
const empty: EffectChainDoc = { enabled: true, input_gain_db: 0, output_gain_db: 0, mix: 1, slots: [] };
let n = 0;
const nid = () => `s${++n}`;

describe("duck settings", () => {
  it("asks for more squeeze for a deeper duck, never past the ratio limit", () => {
    const r = DUCK_DEPTHS.map((d) => duckParams(d.db).ratio);
    expect(r[0]).toBeLessThan(r[1]!);
    expect(r[1]).toBeLessThan(r[2]!);
    expect(duckParams(99).ratio).toBeLessThanOrEqual(20);
    expect(duckParams(0).ratio).toBeGreaterThanOrEqual(1);
  });

  it("predicts the chosen depth for a voice at a normal level", () => {
    for (const d of DUCK_DEPTHS) {
      const p = duckParams(d.db);
      // Gain reduction of a compressor 25 dB over its threshold.
      const reduction = 25 * (1 - 1 / p.ratio!);
      expect(Math.abs(reduction - d.db)).toBeLessThan(0.6);
    }
  });
});

describe("duck chains", () => {
  it("adds one keyed slot per voice and replaces its own earlier slots", () => {
    const first = withDuck(empty, ["a", "b"], 10, nid)!;
    expect(first.slots.map((s) => s.sidechain)).toEqual(["a", "b"]);
    expect(first.slots.every(isDuckSlot)).toBe(true);
    const again = withDuck(first, ["a"], 6, nid)!;
    expect(again.slots).toHaveLength(1);
    expect(again.slots[0]!.params.ratio).toBe(duckParams(6).ratio);
  });

  it("keeps the other effects and takes only the duck out on remove", () => {
    const other = { id: "x", effect: "altaudio.limiter", enabled: true, params: {}, preset: null };
    const c = withDuck({ ...empty, slots: [other] }, ["a"], 10, nid)!;
    expect(c.slots).toHaveLength(2);
    expect(withoutDuck(c).slots).toEqual([other]);
  });

  it("refuses a chain that would pass 16 effects", () => {
    const full = { ...empty, slots: Array.from({ length: 16 }, (_, i) => ({ id: `f${i}`, effect: "altaudio.limiter", enabled: true, params: {}, preset: null })) };
    expect(withDuck(full, ["a"], 10, nid)).toBeNull();
  });
});

describe("duck keys", () => {
  it("offers the other voice tracks, not the music itself or muted and empty tracks", () => {
    const music = track("music", { clips: [{ id: "mc", media_id: "m", audio_type: "music" }] as never });
    const doc = { tracks: [track("host"), track("guest"), music, track("quiet", { muted: true }), track("none", { clips: [] })] } as unknown as SessionDoc;
    expect(duckKeys(doc, music).map((t) => t.id)).toEqual(["host", "guest"]);
    expect(hasDuck(music)).toBe(false);
  });
});

describe("duck plan", () => {
  const music = track("music", { clips: [{ id: "mc", media_id: "m", audio_type: "music" }] as never });
  const doc = (roles: Record<string, TrackDoc["mix_role"]>) =>
    ({ tracks: [track("host"), track("guest"), music, track("bed")].map((t) => ({ ...t, mix_role: roles[t.id] })) }) as unknown as SessionDoc;

  it("works as it always did when no roles are set", () => {
    const d = doc({});
    const p = duckPlan(d, d.tracks[2]!)!;
    expect(p.targets.map((t) => t.id)).toEqual(["music"]);
    expect(p.keys.map((t) => t.id)).toEqual(["host", "guest", "bed"]);
    expect(p.byRole).toBe(false);
    expect(duckPlan(d, null)).toBeNull();
  });

  it("ducks every Ducked track when nothing, or a Ducked track, is selected", () => {
    const d = doc({ music: "ducked", bed: "ducked" });
    expect(duckPlan(d, null)!.targets.map((t) => t.id)).toEqual(["music", "bed"]);
    expect(duckPlan(d, d.tracks[2]!)!.targets.map((t) => t.id)).toEqual(["music", "bed"]);
  });

  it("lets a selected track that is not Ducked win", () => {
    const d = doc({ music: "ducked" });
    expect(duckPlan(d, d.tracks[3]!)!.targets.map((t) => t.id)).toEqual(["bed"]);
  });

  it("listens to the Foreground tracks when there are any", () => {
    const d = doc({ music: "ducked", host: "foreground" });
    const p = duckPlan(d, null)!;
    expect(p.keys.map((t) => t.id)).toEqual(["host"]);
    expect(p.byRole).toBe(true);
  });

  it("does not listen to a muted or empty Foreground track, or to itself", () => {
    const d = doc({ music: "ducked", host: "foreground", guest: "foreground" });
    d.tracks[0]!.muted = true;
    d.tracks[1]!.clips = [];
    const p = duckPlan(d, null)!;
    expect(p.byRole).toBe(false);
    expect(p.keys.map((t) => t.id)).not.toContain("music");
  });
});
