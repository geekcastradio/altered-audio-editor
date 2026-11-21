// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { parseNumber, parsePan } from "./dial-input";

describe("typed dial values", () => {
  it("reads signed numbers, with or without dB", () => {
    expect(parseNumber("-2")).toBe(-2);
    expect(parseNumber("+3.5")).toBe(3.5);
    expect(parseNumber("−2.0 dB")).toBe(-2);
    expect(parseNumber("-6dB")).toBe(-6);
    expect(parseNumber("0")).toBe(0);
    expect(parseNumber("2,5")).toBe(2.5);
  });

  it("rejects text that is not a number", () => {
    expect(parseNumber("")).toBeNull();
    expect(parseNumber("loud")).toBeNull();
    expect(parseNumber("1 2")).toBeNull();
  });

  it("reads pan as L, R, C or a number", () => {
    expect(parsePan("C")).toBe(0);
    expect(parsePan("l30")).toBe(-30);
    expect(parsePan("R20")).toBe(20);
    expect(parsePan("-40")).toBe(-40);
    expect(parsePan("x")).toBeNull();
  });
});
