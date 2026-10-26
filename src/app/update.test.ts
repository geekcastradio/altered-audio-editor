// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { updateDue, WEEK_MS } from "./update";

describe("updateDue", () => {
  it("never runs while the setting is off", () => {
    expect(updateDue(false, 0, 1e12)).toBe(false);
  });

  it("runs the first time it is on", () => {
    expect(updateDue(true, 0, 1e12)).toBe(true);
  });

  it("waits a week between checks", () => {
    const last = 1e12;
    expect(updateDue(true, last, last + WEEK_MS - 1)).toBe(false);
    expect(updateDue(true, last, last + WEEK_MS)).toBe(true);
  });

  it("runs when the clock went backwards", () => {
    expect(updateDue(true, 2e12, 1e12)).toBe(true);
  });
});
