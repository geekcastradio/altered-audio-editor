// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the plug-in slot bookkeeping and chain drag edits in TypeScript
// against the fixture the Rust test reads too (shared-fixtures/plugin-sync.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/plugin-sync.json";
import * as S from "../panels/voice-chain/slot-drag";
import * as P from "./plugin-sync";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const docs = fx.docs as unknown as Record<string, never>;

describe("plug-in sync fixture", () => {
  it("lists slots", () => {
    for (const c of fx.slots) {
      const d = docs[c.name]!;
      expect(J(P.instrumentSlots(d).map((x) => [x.trackId, x.slot]))).toEqual(c.instruments);
      expect(J(P.allSlots(d))).toEqual(c.all);
      expect(J(P.pluginSlots(d))).toEqual(c.plugins);
    }
  });
  it("builds sync edits", () => {
    for (const c of fx.sync) {
      const events = c.events.map((e) => ({ ...e, edits: e.edits as [string, number][] }));
      expect(J(P.syncOps(docs[c.name]!, events as never, new Map(c.states as [string, string][])))).toEqual(c.out);
    }
  });
});

describe("slot drag fixture", () => {
  it("keys and chains", () => {
    for (const c of fx.owners) expect(S.ownerKey(c.t as never)).toBe(c.key);
    for (const c of fx.parsed) expect(J(S.parseOwner(c.key))).toEqual(c.out);
    for (const c of fx.chains) expect(J(S.chainAt(docs[c.name]!, c.t as never))).toEqual(c.out);
    for (const c of fx.rows) expect(J(S.rowTarget(c.id as string, c.bus as boolean))).toEqual(c.out);
  });
  it("drop edits", () => {
    for (const c of fx.drops) {
      const r = S.dropEdit(docs.full!, c.from as never, c.slot as never, { owner: c.owner, index: c.index }, () => "NEW");
      expect(J(r)).toEqual(c.out);
    }
  });
});
