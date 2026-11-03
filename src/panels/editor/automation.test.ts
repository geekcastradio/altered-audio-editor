// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { EffectInfo } from "../../app/core";
import {
  autoAdd,
  autoAt,
  autoChoices,
  autoMode,
  autoMove,
  autoMoveMany,
  autoRange,
  autoValue,
  autoY,
  nextTarget,
  putLane,
  setMode,
  modeChange,
  newLane,
  shownLanes,
  VOLUME_RANGE,
  type AutoOwner,
} from "./automation";

const catalog = [
  {
    id: "eq",
    name: "EQ",
    category: "",
    description: "",
    params: [
      { key: "freq", name: "Frequency", unit: "Hz", min: 20, max: 20000, default: 1000, step: 1, log: true, toggle: false },
      { key: "on", name: "Bypass", unit: "", min: 0, max: 1, default: 0, step: 1, log: false, toggle: true },
    ],
  },
] as unknown as EffectInfo[];

const owner = (o: Partial<AutoOwner> = {}): AutoOwner => ({ volume_db: -3, pan: 20, ...o });

describe("automation lanes", () => {
  it("offers volume, pan and gliding effect controls", () => {
    const t = owner({ effects: { slots: [{ id: "s1", effect: "eq", enabled: true, params: {}, preset: null }] } as never });
    expect(autoChoices(t, catalog).map((c) => c.label)).toEqual(["Volume", "Pan", "EQ: Frequency"]);
    const r = autoRange(t, catalog, { type: "effect_param", slot_id: "s1", param: "freq" });
    expect(r.log).toBe(true);
    // Log lanes put 632 Hz (the middle of 20 Hz to 20 kHz) halfway down.
    expect(autoY(Math.sqrt(20 * 20000), r, 0, 100)).toBeCloseTo(50, 3);
  });

  it("maps y to values and back", () => {
    expect(autoValue(0, VOLUME_RANGE, 0, 72)).toBe(12);
    expect(autoValue(72, VOLUME_RANGE, 0, 72)).toBe(-60);
    expect(autoY(0, VOLUME_RANGE, 0, 72)).toBeCloseTo(12);
  });

  it("holds flat outside the points and moves in lines between them", () => {
    const pts = autoAdd(autoAdd([], 100, -10).points, 300, 10).points;
    expect(autoAt(pts, 0, 0)).toBe(-10);
    expect(autoAt(pts, 200, 0)).toBe(0);
    expect(autoAt(pts, 900, 0)).toBe(10);
    expect(autoAt([], 5, -3)).toBe(-3);
    // A point stays between its neighbors.
    expect(autoMove(pts, 0, 500, 0)[0]!.position).toBe(299);
  });

  it("starts with an unsaved Volume lane, and adds and switches lanes", () => {
    const t = owner();
    expect(shownLanes(t)).toHaveLength(1);
    expect(nextTarget(t, catalog)).toEqual({ type: "pan" });
    const envs = putLane(t, { ...shownLanes(t)[0]!, points: [{ position: 0, value: 0, curve: "linear" }] });
    expect(envs).toHaveLength(1);
    expect(envs[0]!.id).not.toBe("");
    expect(autoMode({ ...t, envelopes: envs })).toBe("read");
    expect(autoMode({ ...t, envelopes: setMode({ ...t, envelopes: envs }, "off") })).toBe("off");
  });
});

describe("recording modes", () => {
  const t = { volume_db: 0, pan: 0 };
  it("a record setting is the mode", () => {
    expect(autoMode({ ...t, record_mode: "latch" })).toBe("latch");
  });
  it("modeChange turns lanes on for Write, Latch, Touch and Read, off for Off", () => {
    const envs = [{ ...newLane({ type: "volume" }), enabled: false }];
    const owner = { ...t, envelopes: envs };
    expect(modeChange(owner, "touch")).toMatchObject({ record: "touch", envelopes: [{ enabled: true }] });
    expect(modeChange(owner, "read")).toMatchObject({ record: "off", envelopes: [{ enabled: true }] });
    expect(modeChange(owner, "off")).toMatchObject({ record: "off", envelopes: [{ enabled: false }] });
  });
});

describe("moving several points", () => {
  const pt = (position: number, value = 0) => ({ position, value, curve: "linear" as const });
  const keep = (p: { value: number }) => p.value;
  it("shifts every selected point by the same distance", () => {
    const out = autoMoveMany([pt(100), pt(200), pt(300), pt(900)], new Set([1, 2]), 50, keep);
    expect(out.map((p) => p.position)).toEqual([100, 250, 350, 900]);
  });
  it("stops at the unselected neighbors", () => {
    const out = autoMoveMany([pt(100), pt(200), pt(300), pt(400)], new Set([1, 2]), 500, keep);
    // The last selected point (300) can move 99 before it meets the one at 400.
    expect(out.map((p) => p.position)).toEqual([100, 299, 399, 400]);
  });
  it("does not go before the start", () => {
    const out = autoMoveMany([pt(10), pt(60)], new Set([0, 1]), -500, keep);
    expect(out.map((p) => p.position)).toEqual([0, 50]);
  });
  it("sets each selected point's value from the callback and leaves the others", () => {
    const out = autoMoveMany([pt(10, -6), pt(20, -12), pt(30, 0)], new Set([0, 1]), 0, (p) => p.value + 3);
    expect(out.map((p) => p.value)).toEqual([-3, -9, 0]);
  });
});
