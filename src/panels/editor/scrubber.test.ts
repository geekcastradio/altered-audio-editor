// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { Scrubber, type ScrubTransport } from "./scrubber";

function fake(playing = false) {
  const calls: string[] = [];
  const t: ScrubTransport = {
    current: { playing },
    play: async (from) => void calls.push(`play:${from}`),
    pause: async () => void calls.push("pause"),
    seek: async (to) => void calls.push(`seek:${to}`),
  };
  return { t, calls };
}

describe("Scrubber", () => {
  it("plays from the pointer at most once per gap, skipping repeats of the same spot", () => {
    const { t, calls } = fake();
    let now = 0;
    const s = new Scrubber(t, true, () => now, 70);
    s.move(100);
    now = 30;
    s.move(200);
    now = 80;
    s.move(200);
    now = 160;
    s.move(200);
    now = 240;
    s.move(300);
    expect(calls).toEqual(["play:100", "play:200", "play:300"]);
  });

  it("stops on release and puts the playhead at the pointer", async () => {
    const { t, calls } = fake();
    const s = new Scrubber(t, true, () => 1000, 70);
    s.move(500);
    await s.end(900);
    expect(calls).toEqual(["play:500", "pause", "seek:900"]);
  });

  it("only seeks when nothing was played", async () => {
    const { t, calls } = fake();
    const s = new Scrubber(t, true);
    await s.end(42);
    expect(calls).toEqual(["seek:42"]);
  });

  it("does nothing while scrubbing is off or playback was already running", async () => {
    for (const [enabled, playing] of [[false, false], [true, true]] as const) {
      const { t, calls } = fake(playing);
      const s = new Scrubber(t, enabled);
      expect(s.active).toBe(false);
      s.move(100);
      await s.end(200);
      expect(calls).toEqual(["seek:200"]);
    }
  });
});
