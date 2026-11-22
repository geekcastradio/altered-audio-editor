// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks host profiles and Sound Types in TypeScript against the fixture the Rust test reads too
// (shared-fixtures/sound-host.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/sound-host.json";
import * as S from "../panels/audio-fx/soundtypes";
import * as H from "./host-profiles";

const counter = () => {
  let n = 0;
  return () => `n${++n}`;
};
const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("sound types and host profiles fixture", () => {
  it("host profiles", () => {
    for (const c of fx.profiles) {
      const p = H.profileById(c.p.id)!;
      expect(J(p)).toEqual(c.p);
      expect(J(H.profileChain(p, counter()))).toEqual(c.chain);
      expect(J(H.profileOp({ id: "t1" } as never, p, counter()))).toEqual(c.op);
    }
    expect(J(H.profileOp({ id: "t1" } as never, null, counter()))).toEqual(fx.clear);
    for (const c of fx.byId) expect(J(H.profileById(c.id))).toEqual(c.out);
  });
  it("sections and setups", () => {
    for (const c of fx.sections) for (const o of c.outs) expect(J(S.sectionEffects(c.type as S.SoundType, c.sec, o.a))).toEqual(o.out);
    for (const c of fx.setups) {
      expect(J(S.presetSetup(c.type as S.SoundType, c.id))).toEqual(c.out);
      expect(J(S.presetSetup(c.type as S.SoundType, "zzz"))).toEqual(c.fallback);
    }
    for (const c of fx.made) expect(J(S.makeSlots(c.setup as never, counter()))).toEqual(c.slots);
  });
  it("chains", () => {
    for (const c of fx.chains) {
      const chain = c.chain as never;
      const setup = S.readSetup(chain);
      expect(J(setup)).toEqual(c.setup);
      expect(S.isCustom(chain, setup)).toBe(c.custom);
      const m = fx.made[c.i]!.setup as never;
      expect(S.isCustom(chain, m)).toBe(c.customOther);
      expect(J(S.withSetup(chain, m, counter()))).toEqual(c.with);
      expect(J(S.withSetup(chain, m, counter(), 3))).toEqual(c.withMax);
      expect(J(S.withoutSetup(chain))).toEqual(c.without);
    }
    expect(J(S.readSetup(null))).toEqual(fx.nulls.readNull);
    expect(S.isCustom(null, null)).toBe(fx.nulls.customNull);
  });
});
