// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript Source Channels and channel switch rules against the same fixture the
// Rust test reads (shared-fixtures/channel-rules.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/channel-rules.json";
import * as M from "../panels/properties/channel-mix";
import * as W from "../panels/editor/wave-channels";

const J = (v: unknown) => JSON.parse(JSON.stringify(v));

describe("channel rules fixture", () => {
  it("labels and mixes", () => {
    for (const c of fx.labels) expect(M.channelLabels(c.n)).toEqual(c.out);
    for (const c of fx.defaults) expect(J(M.defaultMix(c.n).sends)).toEqual(c.out);
    for (const c of fx.owns) expect(J(M.mixOf(c.own as never, c.n).sends)).toEqual(c.out);
  });
  it("presets", () => {
    for (const c of fx.pres) expect(J(M.presets(c.n).map((p) => ({ id: p.id, label: p.label, sends: p.mix.sends })))).toEqual(c.out);
    for (const c of fx.matches) expect(M.presetFor({ sends: c.sends }, c.n)).toBe(c.out);
  });
  it("channel switches", () => {
    for (const c of fx.allOn) expect(W.allOn(c.n)).toEqual(c.out);
    for (const c of fx.channels) {
      expect(c.c.map((_, k) => W.toggleChannel(c.c, k))).toEqual(c.toggles);
      expect(c.c.map((_, k) => W.soloChannel(c.c, k))).toEqual(c.solos);
      expect(W.oneChannel(c.c)).toBe(c.one);
      expect(W.someOff(c.c)).toBe(c.off);
      expect(J(W.editTake(c.c))).toEqual(c.edit);
      expect(J([W.pasteTake(c.c, undefined), W.pasteTake(c.c, 0), W.pasteTake(c.c, c.c.length - 1)])).toEqual(c.paste);
      expect(W.channelLabel(c.c)).toBe(c.label);
    }
  });
});
