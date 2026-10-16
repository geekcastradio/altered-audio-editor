// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { addRows, conflicts, destFor, expandTemplate, sanitizeBatch, type BatchRow } from "./batch";

const row = (id: string, source: string): BatchRow => ({ id, source, status: "waiting", progress: 0, error: null });

describe("Batch Convert", () => {
  it("names files from a template", () => {
    const base = { template: "{name}", folder: null, kind: "wav" as const };
    expect(destFor("C:\\Calls\\1-host.flac", base)).toBe("C:\\Calls\\1-host.wav");
    expect(destFor("/home/a/ep.v2.mp3", { ...base, template: "Show_{name} (clean)", kind: "flac" })).toBe("/home/a/Show_ep.v2 (clean).flac");
    expect(destFor("C:\\in\\a.wav", { ...base, folder: "D:\\out\\", kind: "mp3" })).toBe("D:\\out\\a.mp3");
    expect(destFor("/x/a.wav", { ...base, template: "{name}a/b:c" })).toBe("/x/aa_b_c.wav");
  });

  it("fills number, date and format tokens", () => {
    const ctx = { number: 3, count: 12, date: new Date(2026, 9, 5) };
    expect(expandTemplate("Ep{number}-{name}", "intro", "mp3", ctx)).toBe("Ep03-intro");
    expect(expandTemplate("{date}_{name}.{format}", "intro", "flac", { ...ctx, number: 1 })).toBe("2026-10-05_intro.flac");
    expect(expandTemplate("{NUMBER}", "x", "wav", { number: 7, count: 120, date: ctx.date })).toBe("007");
    expect(expandTemplate("   ", "keep", "wav", ctx)).toBe("keep");
    expect(expandTemplate("{unknown}", "keep", "wav", ctx)).toBe("{unknown}");
  });

  it("finds rows that would collide or replace their source", () => {
    const rows = [row("1", "/a/ep.flac"), row("2", "/a/ep.wav"), row("3", "/a/other.wav")];
    const dests = new Map(rows.map((r) => [r.id, destFor(r.source, { template: "{name}", folder: null, kind: "wav" })]));
    const c = conflicts(rows, dests, false);
    expect(c.get("1")).toBeUndefined();
    expect(c.get("2")).toMatch(/same file as ep\.flac/);
    expect(c.get("3")).toMatch(/replace the original/);
    expect(conflicts(rows, dests, true).has("3")).toBe(false);
  });

  it("adds each file once and keeps saved settings sane", () => {
    let n = 0;
    const { rows, added } = addRows([row("x", "C:\\a\\One.wav")], ["c:/a/one.wav", "C:\\a\\Two.wav", "C:\\a\\Two.wav"], () => `n${n++}`);
    expect(added).toBe(1);
    expect(rows.map((r) => r.source)).toEqual(["C:\\a\\One.wav", "C:\\a\\Two.wav"]);
    expect(sanitizeBatch({ prefix: 5, folder: "", overwrite: "yes", format: { kind: "mp3" } })).toMatchObject({
      template: "{name}",
      folder: null,
      overwrite: false,
      format: { kind: "mp3" },
    });
    // Settings saved when the panel had Before and After boxes still work.
    expect(sanitizeBatch({ prefix: "Show_", suffix: "_v2" }).template).toBe("Show_{name}_v2");
  });
});
