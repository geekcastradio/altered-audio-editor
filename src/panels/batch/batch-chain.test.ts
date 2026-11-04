// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { sanitizeBatch, shapingFor } from "./batch";
import { applyChain, BUILTIN_CHAINS, chainFrom, describeChain, removeChain, sanitizeChains, saveChain } from "./batch-chain";

describe("Batch chains", () => {
  it("keeps what is valid and drops the rest", () => {
    const got = sanitizeChains([
      { name: "  A  ", process: "Warm", level: { loudness: true, loudnessLufs: -3 } },
      { name: "a" },
      { name: "" },
      null,
      "x",
      { name: "B", format: { kind: "nope" } },
    ]);
    expect(got.map((c) => c.name)).toEqual(["A", "B"]);
    expect(got[0]!.level.loudnessLufs).toBe(-5);
    expect(got[0]!.process).toBe("Warm");
    expect(sanitizeChains("junk")).toEqual([]);
  });

  it("saves over a chain of the same name and removes by name", () => {
    const s = sanitizeBatch(null);
    const one = chainFrom("Mine", s, false);
    const first = saveChain([], one)!;
    const again = saveChain(first, { ...one, name: "MINE", process: "X" })!;
    expect(again).toHaveLength(1);
    expect(again[0]!.process).toBe("X");
    expect(saveChain(first, { ...one, name: "  " })).toBeNull();
    expect(removeChain(again, "MINE")).toEqual([]);
  });

  it("applies steps without touching names, tags or folder", () => {
    const base = { ...sanitizeBatch(null), prefix: "x_", folder: "D:\\out", tags: { title: "T", artist: "", album: "", year: "", genre: "", comment: "" } };
    const out = applyChain(base, BUILTIN_CHAINS[0]!);
    expect(out.prefix).toBe("x_");
    expect(out.folder).toBe("D:\\out");
    expect(out.tags.title).toBe("T");
    expect(out.format).toEqual(base.format);
    expect(out.level.loudness).toBe(true);
  });

  it("turns steps into what the core runs", () => {
    const podcast = BUILTIN_CHAINS[0]!;
    expect(shapingFor(podcast.level)).toMatchObject({ loudnessLufs: -16, ceilingDb: -1, trimDb: -50, fadeOutMs: 300, normalizeDb: null });
    expect(describeChain(podcast)).toBe("trim silence below -50 dB, then -16 LUFS, peaks under -1 dB, then 300 ms fade out");
  });

  it("starter chains have unique names", () => {
    expect(new Set(BUILTIN_CHAINS.map((c) => c.name.toLowerCase())).size).toBe(BUILTIN_CHAINS.length);
  });
});
