// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { DEFAULT_CHOICES, describeReport, editFor, isNoChange, sanitizeChoices } from "./pitch-edit";

describe("pitch choices", () => {
  it("keeps what is valid and puts back defaults for the rest", () => {
    const c = sanitizeChoices({ mode: "bend", key: 14.4, scale: "nope", strength: 250, speedMs: -3, maxCents: "x", bendStart: -900, bendMiddle: 30 });
    expect(c.mode).toBe("bend");
    expect(c.key).toBe(11);
    expect(c.scale).toBe("chromatic");
    expect(c.strength).toBe(100);
    expect(c.speedMs).toBe(0);
    expect(c.maxCents).toBe(DEFAULT_CHOICES.maxCents);
    expect(c.bendStart).toBe(-700);
    expect(c.bendMiddle).toBe(30);
    expect(sanitizeChoices(null)).toEqual(DEFAULT_CHOICES);
  });
});

describe("the edit", () => {
  it("turns percent into a fraction for tuning", () => {
    expect(editFor({ ...DEFAULT_CHOICES, strength: 60, key: 2, scale: "major" })).toEqual({ kind: "tune", key: 2, scale: "major", strength: 0.6, speedMs: 40, maxCents: 200 });
  });
  it("makes a line with an optional middle point for a bend", () => {
    expect(editFor({ ...DEFAULT_CHOICES, mode: "bend", bendStart: 0, bendEnd: 100 })).toEqual({ kind: "bend", points: [[0, 0], [1, 100]] });
    expect(editFor({ ...DEFAULT_CHOICES, mode: "bend", bendStart: 0, bendMiddle: 50, bendEnd: 0 })).toEqual({ kind: "bend", points: [[0, 0], [0.5, 50], [1, 0]] });
  });
  it("knows when nothing would change", () => {
    expect(isNoChange({ ...DEFAULT_CHOICES, strength: 0 })).toBe(true);
    expect(isNoChange(DEFAULT_CHOICES)).toBe(false);
    expect(isNoChange({ ...DEFAULT_CHOICES, mode: "bend" })).toBe(true);
    expect(isNoChange({ ...DEFAULT_CHOICES, mode: "bend", bendMiddle: 20 })).toBe(false);
  });
});

describe("the report", () => {
  it("says how much had a clear pitch and how much moved", () => {
    expect(describeReport({ frames: 100, voiced: 60, moved: 30 })).toMatch(/60 percent.*30 percent/);
    expect(describeReport({ frames: 100, voiced: 60, moved: 0 })).toMatch(/none of it needed/);
  });
});
