// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Reads the same fixture as src-tauri/tests/envelope_fx_fixture.rs.
import { describe, expect, it } from "vitest";
import raw from "../../../shared-fixtures/envelope-fx.json";
import * as E from "./envelope";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const fx = raw as any;
const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("effect envelopes fixture", () => {
  E.setFxCatalog(fx.catalog);
  it("ranges, points, rest values, names and changes match Rust", () => {
    for (const c of fx.cases) {
      const clip = c.clip ?? undefined;
      expect(J(E.rangeOf(c.kind, clip, c.lane))).toEqual(c.range);
      expect(E.restValue(c.kind, clip, c.lane)).toBe(c.rest);
      expect(E.envName(c.kind, clip, c.lane)).toBe(c.name);
      if (c.clip) {
        expect(J(E.pointsOf(c.clip, c.kind, c.lane))).toEqual(c.points);
        if (!(c.kind === "fx" && !c.lane)) expect(J(E.envChange(c.clip, c.kind, [{ position: 1, value: 0.5, curve: "linear" }], c.lane))).toEqual(c.change);
      }
    }
  });
  it("choices, picked lanes and controls match Rust", () => {
    for (const c of fx.choices) expect(J(E.fxChoices(c.clip))).toEqual(c.out);
    for (const c of fx.laneOf) expect(J(E.fxLaneOf(c.clip, c.picked ? { [c.clip.id]: c.picked } : {}))).toEqual(c.out);
    for (const c of fx.fxp) {
      const r = E.fxParam(c.clip, c.lane);
      expect(r ? { effect: r.info.name, param: r.param.name, min: r.param.min, max: r.param.max, step: r.param.step, value: r.value } : null).toEqual(c.out);
    }
  });
});
