// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it, vi } from "vitest";
import { AutoTake } from "./auto-take";
import type { Op, SessionDoc } from "../stores/session";

function setup(mode: "write" | "latch" | "touch" | undefined, lane = true) {
  let pos = 0;
  const track: Record<string, unknown> = {
    id: "t1",
    name: "T",
    volume_db: -6,
    pan: 0,
    record_mode: mode,
    envelopes: lane ? [{ id: "e1", target: { type: "volume" }, enabled: true, points: [{ position: 0, value: -6, curve: "linear" }, { position: 100_000, value: -6, curve: "linear" }] }] : [],
  };
  const doc = { sample_rate: 48_000, tracks: [track], buses: [] } as unknown as SessionDoc;
  const direct: Op[] = [];
  const commits: { label: string; op: Op }[] = [];
  const live = vi.fn();
  const clearLive = vi.fn();
  const t = new AutoTake({
    doc: () => doc,
    position: () => pos,
    thin: () => 0,
    applyDirect: async (op) => void direct.push(op),
    commit: async (label, op) => void commits.push({ label, op }),
    live,
    clearLive,
  });
  return { t, direct, commits, live, clearLive, at: (p: number) => (pos = p) };
}

describe("AutoTake", () => {
  it("ignores a track that only plays its lanes", () => {
    const s = setup(undefined);
    s.t.start();
    expect(s.t.control("t1", { type: "volume" }, "move", -3)).toBe(false);
    expect(s.direct).toEqual([]);
  });

  it("does nothing when playback is not running", () => {
    const s = setup("touch");
    expect(s.t.control("t1", { type: "volume" }, "grab", -3)).toBe(false);
  });

  it("Touch: holds the lane while held, writes one undo step at stop", async () => {
    const s = setup("touch");
    s.t.start();
    s.at(1000);
    expect(s.t.control("t1", { type: "volume" }, "grab", -6)).toBe(true);
    s.at(2000);
    s.t.control("t1", { type: "volume" }, "move", -12);
    s.at(3000);
    s.t.control("t1", { type: "volume" }, "release", -12);
    expect(s.live).toHaveBeenLastCalledWith("t1", { type: "volume" }, null);
    s.at(5000);
    await s.t.stop();
    expect(s.clearLive).toHaveBeenCalled();
    // The fader goes back to -6 without an undo step.
    expect(s.direct[s.direct.length - 1]).toEqual({ op: "set_track", id: "t1", changes: { volume_db: -6 } });
    expect(s.commits).toHaveLength(1);
    expect(s.commits[0]!.label).toBe("Record Automation");
    const op = s.commits[0]!.op as unknown as { op: string; owner: string; envelopes: { points: { position: number; value: number }[] }[] };
    expect(op.op).toBe("set_envelopes");
    const pts = op.envelopes[0]!.points;
    expect(pts.find((p) => p.position === 2000)!.value).toBe(-12);
    expect(pts.find((p) => p.position === 3000)!.value).toBe(-12);
    // The lane's own -6 returns after the glide.
    expect(pts.find((p) => p.position > 3000 && p.value === -6)).toBeTruthy();
  });

  it("Write: puts the control's own value over the lane from the start", async () => {
    const s = setup("write");
    s.at(0);
    s.t.start();
    expect(s.live).toHaveBeenCalledWith("t1", { type: "volume" }, -6);
    s.at(1000);
    s.t.control("t1", { type: "volume" }, "move", -10);
    expect(s.live).toHaveBeenLastCalledWith("t1", { type: "volume" }, -10);
    s.at(2000);
    await s.t.stop();
    expect(s.commits).toHaveLength(1);
  });

  it("writes nothing when nothing was touched", async () => {
    const s = setup("touch");
    s.t.start();
    await s.t.stop();
    expect(s.commits).toEqual([]);
  });

  it("Touch with no lane yet makes one", async () => {
    const s = setup("touch", false);
    s.t.start();
    s.at(100);
    s.t.control("t1", { type: "volume" }, "grab", -6);
    s.at(200);
    s.t.control("t1", { type: "volume" }, "move", -9);
    s.at(300);
    s.t.control("t1", { type: "volume" }, "release", -9);
    await s.t.stop();
    const op = s.commits[0]!.op as unknown as { envelopes: { target: { type: string }; points: unknown[] }[] };
    expect(op.envelopes).toHaveLength(1);
    expect(op.envelopes[0]!.target.type).toBe("volume");
  });
});

describe("AutoTake with an effect setting", () => {
  const info = { id: "altaudio.dynamics", name: "Dynamics", params: [{ key: "threshold_db", name: "Threshold", min: -60, max: 0, step: 0.1, default: -20, log: false, unit: "dB" }] };
  function fx() {
    let pos = 0;
    const track: Record<string, unknown> = {
      id: "t1",
      name: "T",
      volume_db: 0,
      pan: 0,
      record_mode: "touch",
      effects: { enabled: true, input_gain_db: 0, output_gain_db: 0, mix: 1, slots: [{ id: "s1", effect: "altaudio.dynamics", enabled: true, params: { threshold_db: -20 } }] },
      envelopes: [],
    };
    const doc = { sample_rate: 48_000, tracks: [track], buses: [] } as unknown as SessionDoc;
    const direct: Op[] = [];
    const commits: { label: string; op: Op }[] = [];
    const t = new AutoTake({
      doc: () => doc,
      position: () => pos,
      thin: () => 0,
      applyDirect: async (op) => void direct.push(op),
      commit: async (label, op) => void commits.push({ label, op }),
      live: vi.fn(),
      clearLive: vi.fn(),
      catalog: () => [info] as never,
    });
    return { t, direct, commits, at: (p: number) => (pos = p) };
  }
  const target = { type: "effect_param" as const, slot_id: "s1", param: "threshold_db" };

  it("records a slider move into a new lane and puts the slider back", async () => {
    const s = fx();
    s.t.start();
    s.at(1000);
    expect(s.t.control("t1", target, "grab", -20)).toBe(true);
    s.at(2000);
    s.t.control("t1", target, "move", -30);
    s.at(3000);
    s.t.control("t1", target, "release", -30);
    s.at(4000);
    await s.t.stop();
    const ops = s.direct as unknown as { op: string; chain?: { slots: { params: Record<string, number> }[] } }[];
    expect(ops.some((o) => o.op === "set_effects" && o.chain?.slots[0]?.params.threshold_db === -30)).toBe(true);
    expect(ops[ops.length - 1]!.chain!.slots[0]!.params.threshold_db).toBe(-20);
    expect(s.commits).toHaveLength(1);
    const op = s.commits[0]!.op as unknown as { op: string; envelopes: { target: unknown; points: { value: number }[] }[] };
    expect(op.op).toBe("set_envelopes");
    expect(op.envelopes[0]!.target).toEqual(target);
    expect(op.envelopes[0]!.points.some((p) => p.value === -30)).toBe(true);
  });
});

