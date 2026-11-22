// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript take recorder, effect favorites and controls, and curve presets
// against the same fixtures the Rust test reads (shared-fixtures/auto-record.json,
// effect-ui.json and curve-presets.json).
import { describe, expect, it } from "vitest";
import rec from "../../shared-fixtures/auto-record.json";
import ui from "../../shared-fixtures/effect-ui.json";
import cur from "../../shared-fixtures/curve-presets.json";
import { Take, toleranceFor, type RecordMode } from "./auto-record";
import * as F from "../panels/voice-chain/favorites";
import * as C from "../panels/voice-chain/effect-controls";
import { clampParam } from "../panels/voice-chain/chain";
import * as P from "../panels/voice-chain/curve-presets";
import type { EffectParamInfo } from "./core";

const Q = (p: unknown) => p as EffectParamInfo;

describe("take recorder fixture", () => {
  it("scripted takes", () => {
    for (const take of rec.takes) {
      const t = new Take(take.rate);
      take.steps.forEach((s, i) => {
        const key = (s.key ?? "") as string;
        const pos = (s.pos ?? 0) as number;
        const value = (s.value ?? 0) as number;
        const want = take.out[i];
        if (s.op === "arm") t.arm(key, s.mode as RecordMode, s.base as never, s.rest as number, s.tol as number, pos, value);
        else if (s.op === "grab") t.grab(key, pos, value);
        else if (s.op === "set") t.set(key, pos, value);
        else if (s.op === "release") t.release(key, pos);
        else if (s.op === "writing") expect(t.writing(key)).toBe(want);
        else if (s.op === "has") expect(t.has(key)).toBe(want);
        else if (s.op === "finish") expect([...t.finish(pos)]).toEqual(want);
      });
    }
  });
  it("tolerance", () => {
    for (const c of rec.tolerance) expect(toleranceFor({ top: c.top, bottom: c.bottom }, c.share)).toBe(c.out);
  });
});

describe("effect UI fixture", () => {
  it("favorites", () => {
    expect([...F.STARTER_FAVORITES]).toEqual(ui.starter);
    for (const c of ui.sanitize) expect(F.sanitizeFavorites(c.raw)).toEqual(c.out);
    for (const c of ui.toggle) expect(F.toggleFavorite(c.list, c.id)).toEqual(c.out);
  });
  it("columns, typed values and clamping", () => {
    for (const c of ui.columns)
      expect(C.controlColumns(c.params.map(Q)).map((x) => ({ name: x.name, power: x.power?.key ?? null, params: x.params.map((p) => p.key) }))).toEqual(c.out);
    for (const c of ui.parse) expect(C.parseParam(Q(c.p), c.text)).toBe(c.out);
    for (const c of ui.clamps) expect(clampParam(Q(c.p), c.v === "nan" ? NaN : c.v === "inf" ? Infinity : (c.v as number))).toBe(c.out);
  });
});

describe("curve presets fixture", () => {
  it("points, custom presets and the built-in table", () => {
    for (const c of cur.points) expect(P.curvePoints(c.segs as never)).toEqual(c.out);
    for (const c of cur.custom) expect(P.curvePreset("x", c.makeup as number, c.segs as never, c.knee as number).params).toEqual(c.out);
    expect(P.DYNAMICS_CURVE_PRESETS.map((p) => ({ name: p.name, params: p.params }))).toEqual(cur.presets);
  });
});
