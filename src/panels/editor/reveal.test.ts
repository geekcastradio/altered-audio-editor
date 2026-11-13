// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { revealScroll } from "./reveal";

describe("revealScroll", () => {
  it("leaves the view alone when the position is already showing", () => {
    expect(revealScroll(1_000, 500, 1_200)).toBeNull();
    expect(revealScroll(1_000, 500, 1_000)).toBeNull();
    expect(revealScroll(1_000, 500, 1_500)).toBeNull();
  });

  it("goes back to the start for Home", () => {
    expect(revealScroll(50_000, 1_000, 0)).toBe(0);
  });

  it("puts the end near the right edge when asked", () => {
    expect(revealScroll(0, 1_000, 10_000, 0.9)).toBe(9_100);
  });

  it("centers a jump by default and never scrolls before zero", () => {
    expect(revealScroll(0, 1_000, 5_000)).toBe(4_500);
    expect(revealScroll(9_000, 1_000, 200)).toBe(0);
  });

  it("does nothing when the Editor has no width yet", () => {
    expect(revealScroll(0, 0, 5_000)).toBeNull();
  });
});
