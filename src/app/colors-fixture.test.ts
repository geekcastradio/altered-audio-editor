// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript color rules must agree with shared-fixtures/colors.json,
// the same file the Rust tests read (src-tauri/tests/colors_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/colors.json";
import { hexToRgb, hslToRgb, ramp, rgbToHex, rgbToHsl, spectrumLut, type Rgb } from "./colors";

const close = (got: number[], want: number[]) => got.forEach((v, i) => expect(v).toBeCloseTo(want[i]!, 9));

describe("shared colors fixture", () => {
  it.each(fixture.hexToRgb)("hexToRgb: $in", (c) => {
    expect(hexToRgb(c.in)).toEqual(c.want);
  });
  it.each(fixture.rgbToHex)("rgbToHex: $rgb", (c) => {
    expect(rgbToHex(c.rgb as Rgb)).toBe(c.want);
  });
  it.each(fixture.rgbToHsl)("rgbToHsl: $rgb", (c) => {
    close(rgbToHsl(c.rgb as Rgb), c.want);
  });
  it.each(fixture.hslToRgb)("hslToRgb: $h $s $l", (c) => {
    close(hslToRgb(c.h, c.s, c.l), c.want);
  });
  it.each(fixture.ramp)("ramp: $low to $high at $t", (c) => {
    close(ramp(hexToRgb(c.low)!, hexToRgb(c.high)!, c.t), c.want);
  });
  it.each(fixture.spectrumLut)("spectrumLut: '$low' to '$high'", (c) => {
    expect(Array.from(spectrumLut(c.low, c.high))).toEqual(c.want);
  });
});
