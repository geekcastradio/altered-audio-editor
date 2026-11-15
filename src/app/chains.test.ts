// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { Op, SessionDoc } from "../stores/session";
import { ChainCommand, chainSummary, preflight, type ChainDeps, type ChainLoaded } from "./chains";
import { loudnessTargetOp, micOptions, quickPassOp, quickPassSettings, voiceChainPresetOp } from "./chain-actions";
import { BUILTIN_PRESETS as VOICE_PRESETS, type ChainPreset } from "../panels/voice-chain/chain";
import { applyLocal } from "../panels/quickpass/quickpass";
import type { LoudnessReport } from "./loudness";

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

describe("loudness target step", () => {
  const rep = (integrated: number, truePeak: number) => ({ integrated, integratedMono: integrated, truePeak }) as unknown as LoudnessReport;

  it("sets the Master so the mix lands on the target", () => {
    const r = loudnessTargetOp(rep(-22, -10), 0, "podcast-stereo", 0.5);
    expect(r.op).toEqual({ op: "set_master_volume", volume_db: 6 });
  });

  it("adds to where the Master already is", () => {
    const r = loudnessTargetOp(rep(-20, -10), -3, "podcast-stereo", 0.5);
    expect(r.op).toEqual({ op: "set_master_volume", volume_db: 1 });
  });

  it("leaves a mix that is already close alone", () => {
    expect(loudnessTargetOp(rep(-16.3, -5), 0, "podcast-stereo", 0.5).op).toBeNull();
  });

  it("stops at the peak limit and says so", () => {
    const r = loudnessTargetOp(rep(-24, -3), 0, "podcast-stereo", 0.5);
    expect(r.op).toEqual({ op: "set_master_volume", volume_db: 2 });
    expect(r.message).toContain("peak limit");
  });

  it("does nothing for a silent mix", () => {
    expect(loudnessTargetOp(rep(Number.NEGATIVE_INFINITY, -120), 0, "podcast-stereo", 0.5).op).toBeNull();
  });
});

describe("voice chain preset and QuickPass steps", () => {
  const known = new Set(VOICE_PRESETS.flatMap((p) => p.chain.slots.map((s) => s.effect)));
  let n = 0;
  const nid = () => `id${n++}`;
  const clip = (id: string, start: number, length: number) => ({ id, name: id, media_id: "m", start, source_in: 0, source_out: length, gain_db: 0, muted: false, locked: false, color: null, fade_in: { length: 0, shape: "linear", curve: 0 }, fade_out: { length: 0, shape: "linear", curve: 0 } });
  const mk = (): SessionDoc =>
    ({
      sample_rate: 48000,
      media: [{ id: "m", path: "a.wav", relative_path: null, sample_rate: 48000, channels: 1, length: 480000 }],
      tracks: [
        { id: "t1", name: "Host", clips: [clip("a", 0, 480000)] },
        { id: "t2", name: "Guest", clips: [clip("b", 0, 480000)] },
        { id: "t3", name: "Empty", clips: [] },
      ],
    }) as unknown as SessionDoc;

  it("gives every track with audio the preset, as one edit", () => {
    const r = voiceChainPresetOp(mk(), "Podcast Voice", "all", [], known, nid);
    const ops = (r.op as unknown as { ops: { target: { id: string }; chain: { slots: unknown[] } }[] }).ops;
    expect(ops.map((o) => o.target.id)).toEqual(["t1", "t2"]);
    expect(ops[0]!.chain.slots.length).toBeGreaterThan(0);
    expect(r.message).toBe("Podcast Voice on 2 tracks.");
  });

  it("names one track, ignoring case, and says so when it is missing", () => {
    const r = voiceChainPresetOp(mk(), "Podcast Voice - Host", "host", [], known, nid);
    expect((r.op as unknown as { target: { id: string } }).target.id).toBe("t1");
    expect(() => voiceChainPresetOp(mk(), "Podcast Voice", "Nobody", [], known, nid)).toThrow(/no track called/);
    expect(() => voiceChainPresetOp(mk(), "No Such Preset", "all", [], known, nid)).toThrow(/no Voice Chain preset/);
  });

  it("a saved preset with the same name wins over the built-in one", () => {
    const mine: ChainPreset = { name: "Podcast Voice", group: "Voice chains", chain: { enabled: true, input_gain_db: 0, output_gain_db: 0, mix: 1, slots: [{ effect: [...known][0]!, params: {} }] } };
    const r = voiceChainPresetOp(mk(), "Podcast Voice", "Host", [mine], known, nid);
    expect((r.op as unknown as { chain: { slots: unknown[] } }).chain.slots).toHaveLength(1);
  });

  it("QuickPass removes a long silence from the clip, closing the gap on its track", () => {
    const doc = mk();
    const window = 480;
    const db = [...Array(300).fill(-20), ...Array(100).fill(-80), ...Array(600).fill(-20)] as number[];
    const levels = [{ clipId: "a", start: 0, window, db, peak: db }];
    const s = quickPassSettings("Clean Up Podcast Interview", "track");
    const r = quickPassOp(doc, levels, s, nid);
    expect(r.op).not.toBeNull();
    const after = applyLocal(doc, r.op!);
    const total = after.tracks[0]!.clips.reduce((x, c) => x + (c.source_out - c.source_in), 0);
    expect(total).toBe(480000 - (100 * 480 - 7200));
    expect(r.message).toBe("removed 1 stretch.");
  });

  it("QuickPass with nothing quiet changes nothing", () => {
    const db = Array(1000).fill(-20) as number[];
    const r = quickPassOp(mk(), [{ clipId: "a", start: 0, window: 480, db, peak: db }], quickPassSettings("Clean Up Podcast Interview", "none"), nid);
    expect(r.op).toBeNull();
  });

  it("a preset a chain cannot run is refused with a reason, and so is a bad gap setting", () => {
    expect(() => quickPassSettings("No Such Preset", "none")).toThrow(/no QuickPass preset/);
    expect(() => quickPassSettings("Clean Up Podcast Interview", "sideways")).toThrow(/none, track or all/);
  });
});
