// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks Waveform file rows and the 3D Space room maths in TypeScript against the fixture the Rust
// test reads too (shared-fixtures/wave-space.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/wave-space.json";
import { waveRows } from "../files/wave-rows";
import type { SpaceLayout } from "../../stores/session";
import * as S from "./space";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const near = (a: number, b: number) => expect(Math.abs(a - b)).toBeLessThanOrEqual(1e-9 * (1 + Math.max(Math.abs(a), Math.abs(b))));

describe("wave rows and space fixture", () => {
  it("file rows", () => {
    for (const c of fx.rows) {
      const [cur, dirty, parked, listed] = c.args as [never, boolean, never, never];
      expect(J(waveRows(cur, dirty, parked, listed))).toEqual(c.out);
    }
  });
  it("layouts and positions", () => {
    for (const c of fx.layouts) {
      const l = c.l as SpaceLayout;
      expect(J(S.speakerSpots(l))).toEqual(c.spots);
      expect(S.LAYOUT_NAMES[l]).toBe(c.name);
      expect(S.channelNames(l)).toEqual(c.channels);
    }
    for (const c of fx.tidy) expect(J(S.tidyPos(c.p as never))).toEqual(c.out);
  });
  it("room maths", () => {
    for (const c of fx.views) {
      const to = S.toView(c.x, c.y, c.size);
      near(to[0], c.to[0]!);
      near(to[1], c.to[1]!);
      const out = S.fromView(c.x * 300, c.y * 300 - 20, c.size);
      near(out[0], c.out[0]!);
      near(out[1], c.out[1]!);
    }
    for (const c of fx.sides) {
      const got = S.sides({ ...S.tidyPos({}), x: c.x, y: c.y, width: c.width, rotation: c.rotation });
      got.forEach((g, i) => {
        near(g[0], c.out[i]![0]!);
        near(g[1], c.out[i]![1]!);
      });
    }
    for (const c of fx.deg) {
      near(S.wrapDeg(c.d), c.wrap);
      near(S.snapAngle(c.d), c.snap);
    }
    for (const c of fx.angles) near(S.angleOf(c.cx, c.cy, c.x, c.y), c.out);
    for (const c of fx.snaps) {
      const [x, y] = S.snapSpot(c.x, c.y, fx.spk, c.tol);
      near(x, c.out[0]!);
      near(y, c.out[1]!);
    }
  });
});
