// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript time rules must agree with shared-fixtures/time.json,
// the same file the Rust tests read (src-tauri/tests/time_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/time.json";
import { formatDecimal, samplesToSeconds } from "./time";

const rateOf = (r: number | string): number =>
  r === "NaN"
    ? NaN
    : r === "Infinity"
      ? Infinity
      : r === "-Infinity"
        ? -Infinity
        : (r as number);

describe("shared time fixture", () => {
  it("formatDecimal", () => {
    for (const c of fixture.formatDecimal) {
      expect(formatDecimal(c.samples, c.rate), JSON.stringify(c)).toBe(c.want);
    }
  });
  it("samplesToSeconds", () => {
    for (const c of fixture.samplesToSeconds) {
      expect(samplesToSeconds(c.samples, c.rate)).toBeCloseTo(c.want, 9);
    }
  });
  it("rejects bad sample rates", () => {
    for (const c of fixture.badRates) {
      expect(
        () => samplesToSeconds(10, rateOf(c.rate)),
        JSON.stringify(c),
      ).toThrow();
    }
  });
});
