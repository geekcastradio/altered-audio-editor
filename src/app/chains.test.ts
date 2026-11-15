// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { Op, SessionDoc } from "../stores/session";
import { ChainCommand, chainSummary, preflight, type ChainDeps, type ChainLoaded } from "./chains";
import { micOptions } from "./chain-actions";

const doc = { sample_rate: 48000, tracks: [] } as unknown as SessionDoc;
const op = (n: number): Op => ({ op: "test", n }) as unknown as Op;

function loaded(steps: { action: string; on_error?: "stop" | "skip" }[]): ChainLoaded {
  return { chain: { version: 1, name: "Cleanup", steps }, problems: [], resolved: steps.map(() => ({})) };
}

function deps(run: Record<string, () => Promise<{ op: Op | null; message: string }>>) {
  const applied: unknown[] = [];
  const d: ChainDeps = {
    doc: () => doc,
    apply: async (o) => {
      applied.push(o);
      return { op: "undo", of: o } as unknown as Op;
    },
    handlers: Object.fromEntries(Object.entries(run).map(([k, v]) => [k, { title: k, run: v }])),
  };
  return { d, applied };
}

describe("chain run", () => {
  it("applies every step in order and records one reversible entry", async () => {
    const { d, applied } = deps({
      a: async () => ({ op: op(1), message: "one." }),
      b: async () => ({ op: op(2), message: "two." }),
    });
    const c = new ChainCommand(loaded([{ action: "a" }, { action: "b" }]), d);
    await c.do();
    expect(applied).toEqual([op(1), op(2)]);
    expect(c.reports.map((r) => r.status)).toEqual(["done", "done"]);
    await c.undo();
    expect(applied.slice(2).map((o) => (o as { of: Op }).of)).toEqual([op(2), op(1)]);
    await c.do();
    expect(applied.slice(4)).toEqual([op(1), op(2)]);
  });

  it("a step with nothing to do changes nothing and the chain goes on", async () => {
    const { d, applied } = deps({
      a: async () => ({ op: null, message: "no dead space." }),
      b: async () => ({ op: op(2), message: "two." }),
    });
    const c = new ChainCommand(loaded([{ action: "a" }, { action: "b" }]), d);
    await c.do();
    expect(applied).toEqual([op(2)]);
    expect(c.reports[0]!.status).toBe("nothing");
  });

  it("a failing step stops the chain and takes back what already ran", async () => {
    const { d, applied } = deps({
      a: async () => ({ op: op(1), message: "one." }),
      b: async () => {
        throw new Error("A track has MIDI clips.");
      },
    });
    const c = new ChainCommand(loaded([{ action: "a" }, { action: "b" }]), d);
    await expect(c.do()).rejects.toThrow(/Step 2 \(b\) stopped the chain: A track has MIDI clips\. Nothing was changed\./);
    expect(applied).toHaveLength(2);
    expect((applied[1] as { of: Op }).of).toEqual(op(1));
  });

  it("a failing step marked skip is reported and the chain goes on", async () => {
    const { d, applied } = deps({
      a: async () => {
        throw new Error("locked");
      },
      b: async () => ({ op: op(2), message: "two." }),
    });
    const c = new ChainCommand(loaded([{ action: "a", on_error: "skip" }, { action: "b" }]), d);
    await c.do();
    expect(applied).toEqual([op(2)]);
    expect(c.reports[0]).toMatchObject({ status: "skipped", message: "locked" });
    expect(chainSummary("Cleanup", c.reports)).toContain("skipped (locked)");
  });
});

describe("preflight", () => {
  const handlers = { a: { title: "A", run: () => ({ op: null, message: "" }) } };
  it("names the problems the core found", () => {
    const l = { ...loaded([{ action: "a" }]), problems: [{ step: 0, message: "bad" }, { step: null, message: "no name" }] };
    const r = preflight(l, handlers)!;
    expect(r).toContain("Step 1: bad");
    expect(r).toContain("no name");
  });
  it("refuses a step the app cannot run before changing anything", () => {
    expect(preflight(loaded([{ action: "a" }, { action: "zzz" }]), handlers)).toContain("zzz");
    expect(preflight(loaded([{ action: "a" }]), handlers)).toBeNull();
  });
});

describe("mic mix settings", () => {
  it("maps chain names to analysis names and falls back to the defaults", () => {
    expect(micOptions({ reduce_db: 10 })).toMatchObject({ reduceDb: 10, openDb: 8, holdMs: 250 });
  });
});
