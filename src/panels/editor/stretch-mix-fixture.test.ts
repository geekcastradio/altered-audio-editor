// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the stretch edits and Auto Mic Mix helpers in TypeScript against the
// fixture the Rust test reads too (shared-fixtures/stretch-mix.json).
import { describe, expect, it, vi } from "vitest";
import fx from "../../../shared-fixtures/stretch-mix.json";
import * as M from "../../app/multitrack";
import * as S from "./stretch";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const doc = fx.doc as never;

describe("stretch fixture", () => {
  it("stretches and unstretches", () => {
    for (const c of fx.stretch) expect(J(S.stretchTo(c.c as never, (c.m ?? undefined) as never, 48000, c.length, c.kp))).toEqual(c.out);
    for (const c of fx.un) expect(J(S.unstretch(c.c as never, (c.m ?? undefined) as never, 48000))).toEqual(c.out);
    for (const c of fx.natural) expect(S.naturalLength(c.c as never, { sample_rate: 48000 } as never, 48000)).toBe(c.out);
  });
  it("labels and typed times", () => {
    for (const c of fx.percent) expect(S.percent(c.r)).toBe(c.out);
    for (const c of fx.parse) expect(J(S.parseSeconds(c.t))).toEqual(c.out);
  });
});

describe("auto mic mix fixture", () => {
  it("lists and describes", () => {
    expect(J(M.micTracks(doc))).toEqual(fx.mic);
    for (const c of fx.dur) expect(M.formatDuration(c.s)).toBe(c.out);
  });
  it("writes lanes", () => {
    for (const c of fx.mix) {
      let n = 0;
      vi.stubGlobal("crypto", { randomUUID: () => `NEW-${++n}` });
      expect(J(M.mixOps(doc, c.a as never))).toEqual(c.ops);
      expect(M.describeTalk(c.a as never)).toEqual(c.talk);
      expect(M.describeMix(c.a as never)).toBe(c.line);
    }
    vi.unstubAllGlobals();
  });
});
