// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript Batch Convert naming and saved-chain rules against the same fixture the
// Rust test reads (shared-fixtures/batch-rules.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/batch-rules.json";
import * as Bt from "../panels/batch/batch";
import * as Ch from "../panels/batch/batch-chain";

const J = (v: unknown) => JSON.parse(JSON.stringify(v));
const toCtx = (c: { number: number; count: number; y: number; m: number; d: number }) => ({ number: c.number, count: c.count, date: new Date(c.y, c.m - 1, c.d) });
const mkRows = (list: unknown) => (list as [string, string][]).map(([id, source]) => ({ id, source, status: "waiting" as const, progress: 0, error: null }));
const chainOut = (c: Ch.BatchChain) => ({ name: c.name, process: c.process, level: c.level, formatKind: c.format ? c.format.kind : null });
const asChain = (c: { name: string; process: string | null; level: unknown; formatKind: string | null }) =>
  ({ name: c.name, process: c.process, level: c.level, format: c.formatKind ? ({ ...Bt.sanitizeBatch({ format: { kind: c.formatKind } }).format }) : null }) as Ch.BatchChain;

describe("batch rules fixture", () => {
  it("levels and settings", () => {
    for (const c of fx.levels) {
      const l = Bt.sanitizeLevel(c.raw);
      expect(J(l)).toEqual(c.out);
      expect(J(Bt.shapingFor(l))).toEqual(c.shaping);
    }
    for (const c of fx.shapeIn) expect(J(Bt.shapingFor(c.level))).toEqual(c.out);
    for (const c of fx.batches) {
      const { format: _f, ...rest } = Bt.sanitizeBatch(c.raw);
      expect(J(rest)).toEqual(c.out);
    }
    for (const c of fx.eff) expect(Bt.effectiveTemplate(c as never)).toBe(c.out);
  });
  it("names", () => {
    for (const c of fx.expand) expect(Bt.expandTemplate(c.template!, c.stem, c.kind as never, toCtx(c.ctx))).toBe(c.out);
    for (const c of fx.dests) expect(Bt.destFor(c.source, { template: c.template!, folder: c.folder, kind: c.kind } as never, toCtx(c.ctx))).toBe(c.out);
    for (const c of fx.fileNames) expect(Bt.fileName(c.p)).toBe(c.out);
    for (const c of fx.tagCases) expect(J(Bt.tagsFor(c.fields as never, c.source as string, c.kind as never, toCtx(c.ctx)))).toEqual(c.out);
  });
  it("the list", () => {
    for (const c of fx.conflictCases) expect([...Bt.conflicts(mkRows(c.rows) as never, new Map(c.dests as never), c.overwrite)]).toEqual(c.out);
    for (const c of fx.addCases) {
      let n = 0;
      const r = Bt.addRows(mkRows(c.rows) as never, c.paths, () => `id${++n}`);
      expect({ rows: r.rows.map((x) => ({ id: x.id, source: x.source })), added: r.added }).toEqual(c.out);
    }
  });
  it("saved chains", () => {
    for (const c of fx.chains) expect(J(Ch.sanitizeChains(c.raw).map(chainOut))).toEqual(c.out);
    const settings = Bt.sanitizeBatch(fx.settingsRaw);
    for (const c of fx.chainFrom) expect(J(chainOut(Ch.chainFrom(c.name as string, settings, c.withFormat as boolean)))).toEqual(c.out);
    let saved: Ch.BatchChain[] = [];
    for (const c of fx.saves as { name: string; out: unknown }[]) {
      const r = Ch.saveChain(saved, Ch.chainFrom(c.name, settings, c.name === "c"));
      expect(r ? J(r.map(chainOut)) : null).toEqual(c.out);
      if (r) saved = r;
    }
    let big: Ch.BatchChain[] = [];
    for (let i = 0; i < 45; i++) big = Ch.saveChain(big, Ch.chainFrom(`N ${i % 43}`, settings, false)) ?? big;
    expect(J(big.map(chainOut))).toEqual(fx.bigRun);
    for (const c of fx.removes) expect(J(Ch.removeChain(saved, c.name!).map(chainOut))).toEqual(c.out);
    for (const c of fx.applies) {
      const a = Ch.applyChain(Bt.sanitizeBatch(fx.settingsRaw), asChain(c.chain as never));
      expect({ process: a.process, level: J(a.level), formatKind: a.format.kind }).toEqual(c.out);
    }
    for (const c of fx.describes) expect(Ch.describeChain(asChain(c.chain as never))).toBe(c.out);
    expect(J(Ch.BUILTIN_CHAINS.map(chainOut))).toEqual(fx.builtin);
  });
});
