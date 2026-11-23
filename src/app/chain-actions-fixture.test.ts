// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the chain actions in TypeScript against the fixture the Rust test reads
// too (shared-fixtures/chain-actions.json). The broadcast region is fixed so the
// result does not depend on this computer's time zone.
import { describe, expect, it, vi } from "vitest";
vi.mock("./loudness", async (orig) => ({ ...(await orig<typeof import("./loudness")>()), guessRegion: () => "europe" }));
import fx from "../../shared-fixtures/chain-actions.json";
import { LockedError } from "../panels/editor/clip-edit";
import * as C from "./chain-actions";

// Non-finite numbers are written as text in the fixture, and as null in a report.
const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null, (_k, x) => (typeof x === "number" && !Number.isFinite(x) ? "__nonfinite__" + String(x) : x)));
const counter = () => {
  let n = 0;
  return () => `n${++n}0`;
};
const doc = fx.doc as never;
const run = (fn: () => unknown) => {
  try {
    return J(fn());
  } catch (e) {
    return { error: (e as Error).message, locked: e instanceof LockedError };
  }
};

describe("chain actions fixture", () => {
  it("loudness and mic", () => {
    for (const c of fx.lt) {
      const r = c.report;
      const report = { integrated: r.integrated ?? -Infinity, integratedMono: r.integratedMono ?? -Infinity, truePeak: r.truePeak ?? -Infinity, range: 5, samplePeak: 0, seconds: 100, cancelled: false };
      expect(J(C.loudnessTargetOp(report as never, c.master, c.target, c.tol))).toEqual(c.out);
    }
    for (const c of fx.mic) expect(J(C.micOptions(c.p as never))).toEqual(c.out);
  });
  it("voice chain presets", () => {
    for (const c of fx.vc) expect(run(() => C.voiceChainPresetOp(doc, c.preset!, c.which, fx.saved as never, new Set(fx.knowns[c.ki]), counter()))).toEqual(c.out);
  });
  it("quickpass steps", () => {
    expect([...C.quickPassClips(doc).entries()].map(([id, v]) => [id, v.trackId, v.name])).toEqual(fx.clips);
    for (const c of fx.qs) {
      const settings = run(() => C.quickPassSettings(c.name, c.ripple));
      expect(settings).toEqual(c.settings);
      if (!settings.error) expect(run(() => C.quickPassOp(doc, fx.levels as never, settings, counter()))).toEqual(c.op);
    }
  });
});
