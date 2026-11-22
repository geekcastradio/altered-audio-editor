// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript clip number rules against the same fixture the Rust test reads
// (shared-fixtures/clip-math.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/clip-math.json";
import * as C from "../panels/editor/clip-edit";
import { clipLength, stretchOf } from "../stores/session";

const base = { id: "c", name: "c", media_id: "m", gain_db: 0 };

describe("clip math fixture", () => {
  it("constants and shapes", () => {
    expect(C.MIN_CLIP).toBe(fx.min);
    expect(C.TOP_LAYER).toBe(fx.top);
    expect(C.FADE_SHAPES).toEqual(fx.shapes);
  });
  it("lengths and ends", () => {
    for (const c of fx.lengths) {
      const clip = {
        ...base,
        start: c.start,
        source_in: c.source_in,
        source_out: c.source_out,
        fade_in: { length: 0, shape: "cosine" },
        fade_out: { length: 0, shape: "cosine" },
        ...(c.looped !== null ? { looped: { length: c.looped, phase: 0 } } : {}),
        ...(c.stretch !== null ? { stretch: { ratio: c.stretch } } : {}),
      } as never;
      const media = c.rate !== null ? ({ id: "m", sample_rate: c.rate } as never) : undefined;
      expect(clipLength(clip, media, c.session)).toBe(c.length);
      expect(C.clipEnd(clip, new Map(media ? [["m", media]] : []) as never, c.session)).toBe(c.end);
      expect(stretchOf(clip)).toBe(c.ratio);
    }
  });
  it("midi spans", () => {
    for (const c of fx.spans) expect(C.midiSpan({ sample_rate: c.rate, tempo: (c.tempo ?? undefined) as never }, { start: c.start, length: c.length })).toEqual(c.out);
  });
  it("snapping", () => {
    for (const c of fx.near) expect(C.nearest(c.v, c.targets, c.tol)).toBe(c.out);
    for (const c of fx.snaps) expect(C.snapToMarkers(c.pos, c.markers, c.tol)).toBe(c.out);
  });
  it("fade shapes", () => {
    for (const c of fx.fades) {
      const clip = { ...base, start: 0, source_in: 0, source_out: 1, fade_in: c.fin, fade_out: c.fout } as never;
      const r = C.fadeShapeChanges([clip], c.shape);
      expect([r[0]?.fade_in !== undefined, r[0]?.fade_out !== undefined]).toEqual(c.out);
    }
  });
});
