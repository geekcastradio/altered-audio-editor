// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { ClipDoc } from "../stores/session";
import { bleedSummary, bleedWork, replaceOps, type CleanedFile } from "./bleed";

const clip = (id: string, extra: Partial<ClipDoc> = {}): ClipDoc =>
  ({ id, name: id, media_id: "m", start: 1000, source_in: 500, source_out: 9000, gain_db: -2, muted: false, locked: false, color: null, ...extra }) as unknown as ClipDoc;

describe("reduce bleed", () => {
  it("leaves locked, looped and stretched clips alone and says how many", () => {
    const w = bleedWork([clip("a"), clip("b", { locked: true }), clip("c", { looped: { length: 10 } as never }), clip("d", { stretch: { ratio: 1.1 } as never })]);
    expect(w.work.map((c) => c.id)).toEqual(["a"]);
    expect(w.locked).toBe(1);
    expect(w.unsuitable).toBe(2);
  });

  it("swaps each clip for one on its cleaned copy and keeps everything else about it", () => {
    const files: CleanedFile[] = [
      { clip: clip("a"), media: { id: "n1", path: "a.wav", sampleRate: 48000, channels: 1, length: 8500 } },
      { clip: clip("b"), media: { id: "n2", path: "b.wav", sampleRate: 48000, channels: 1, length: 4000 } },
    ];
    const ops = replaceOps("t1", files, new Set(["m"]));
    expect(ops.map((o) => o.op)).toEqual(["add_media", "add_media", "remove_clips", "restore_clips"]);
    const restore = ops[3] as { clips: [string, ClipDoc][] };
    expect(restore.clips[0]![0]).toBe("t1");
    expect(restore.clips[0]![1]).toMatchObject({ id: "a", media_id: "n1", source_in: 0, source_out: 8500, start: 1000, gain_db: -2 });
  });

  it("does not add media the session already has", () => {
    const files: CleanedFile[] = [{ clip: clip("a"), media: { id: "n1", path: "a.wav", sampleRate: 48000, channels: 1, length: 10 } }];
    expect(replaceOps("t1", files, new Set(["n1"])).map((o) => o.op)).toEqual(["remove_clips", "restore_clips"]);
  });

  it("the notice says what changed and what was left", () => {
    expect(bleedSummary([2.1, 6.04], { locked: 1, unsuitable: 0 }, 2)).toBe("Cleaned 2 clips, up to 6.0 dB quieter overall. One undo puts the originals back. Listen before you keep it. 1 locked clip was left alone.");
    expect(bleedSummary([0.05], { locked: 0, unsuitable: 0 }, 0)).toBe("That mic picked up almost none of the other one, so nothing was changed.");
  });
});
