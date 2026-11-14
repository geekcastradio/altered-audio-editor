// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { FIT_XFADE } from "../editor/musicfit";
import { formatSeconds, planDuration } from "./remix";
import { FRESH_MS, isFresh, requestSection, sectionRequests } from "./focus";

const plan = (segments: [number, number][], stretch = 1) => ({
  segments: segments.map(([start, end]) => ({ start, end })),
  length: 0,
  stretch,
  joins: Math.max(0, segments.length - 1),
  quality: 1,
  tempo: 120,
});

describe("planDuration", () => {
  it("is the clip's own length for one piece", () => {
    expect(planDuration(plan([[0, 30]]))).toBeCloseTo(30, 6);
  });
  it("takes a crossfade off at each join", () => {
    expect(planDuration(plan([[0, 20], [40, 60]]))).toBeCloseTo(40 - FIT_XFADE, 6);
    expect(planDuration(plan([[0, 10], [0, 10], [0, 10]]))).toBeCloseTo(30 - 2 * FIT_XFADE, 6);
  });
  it("follows the stretch", () => {
    expect(planDuration(plan([[0, 30]], 1.1))).toBeCloseTo(33, 6);
  });
  it("is zero for an empty plan", () => {
    expect(planDuration(plan([]))).toBe(0);
  });
});

describe("formatSeconds", () => {
  it("reads like the dialogs", () => {
    expect(formatSeconds(30)).toBe("30.000");
    expect(formatSeconds(65.25)).toBe("1:05.250");
    expect(formatSeconds(-1)).toBe("0.000");
  });
});

describe("section requests", () => {
  it("are fresh for a moment, then not", () => {
    expect(isFresh(null)).toBe(false);
    expect(isFresh({ section: "remix", at: 1000 }, 1000 + FRESH_MS - 1)).toBe(true);
    expect(isFresh({ section: "remix", at: 1000 }, 1000 + FRESH_MS)).toBe(false);
  });
  it("reach a subscriber", () => {
    let seen: string | null = null;
    const off = sectionRequests.subscribe((r) => (seen = r?.section ?? null));
    requestSection("stretch");
    expect(seen).toBe("stretch");
    off();
  });
});
