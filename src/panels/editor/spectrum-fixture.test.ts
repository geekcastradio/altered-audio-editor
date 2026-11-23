// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the spectrum view axis, zoom, scale marks, colors and selection masks
// in TypeScript against the fixture the Rust test reads too
// (shared-fixtures/spectrum-view.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/spectrum-view.json";
import * as S from "./spectrum";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const views = fx.views as S.FreqView[];

describe("spectrum fixture", () => {
  it("axis and view", () => {
    for (const c of fx.axis) {
      expect(S.freqAt(c.x, c.ny)).toBeCloseTo(c.at, 6);
      expect(S.fracOf(c.x, c.ny)).toBeCloseTo(c.of, 9);
    }
    for (const c of fx.range) expect(S.viewRange(views[c.vi]!, c.ny)).toEqual(c.out);
    for (const c of fx.vf) expect(S.viewFreq(c.x, views[c.vi]!, c.ny)).toBeCloseTo(c.out, 6);
    for (const c of fx.vfrac) expect(S.viewFrac(c.x, views[c.vi]!, c.ny)).toBeCloseTo(c.out, 9);
    for (const c of fx.edges) expect(S.rowEdges(c.rows, views[c.vi]!, c.ny)).toEqual(c.out);
  });
  it("zoom, ticks and colors", () => {
    for (const c of fx.zoom) expect(J(S.zoomFreq(views[c.vi]!, c.hz, c.f, c.ny))).toEqual(c.out);
    for (const c of fx.ticks) expect(S.freqTicks(views[c.vi]!, c.ny, c.h, c.gap)).toEqual(c.out);
    expect(Array.from(S.SPECTRUM_LUT)).toEqual(fx.lut);
  });
  it("selection masks", () => {
    fx.shapes.forEach((set, n) => {
      const shapes = set as S.SpectralShape[];
      expect(J(S.shapesBounds(shapes))).toEqual(fx.bounds[n]);
      expect(J(S.rasterize(shapes))).toEqual(fx.raster[n]);
      expect(J(S.rasterize(shapes, 8, 6))).toEqual(fx.rasterSmall[n]);
    });
  });
});
