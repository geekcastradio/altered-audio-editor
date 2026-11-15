// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { emptyPads, PAD_COUNT, PadsStore, padLabel, sanitizePads, withPad } from "./soundboard";

describe("soundboard pads", () => {
  it("names a pad after its file", () => {
    expect(padLabel("C:\\Show\\Intro Sting.wav")).toBe("Intro Sting");
    expect(padLabel("/a/b/laugh.track.mp3")).toBe("laugh.track");
    expect(padLabel("noext")).toBe("noext");
  });

  it("keeps nine pads and drops anything that is not a path", () => {
    expect(sanitizePads(null)).toEqual(emptyPads());
    const p = sanitizePads([{ path: "a/b.wav", label: "  Open  " }, { path: 5 }, { path: "" }, null, { path: "c.wav" }]);
    expect(p).toHaveLength(PAD_COUNT);
    expect(p[0]).toEqual({ path: "a/b.wav", label: "Open" });
    expect(p[1]!.path).toBeNull();
    expect(p[4]).toEqual({ path: "c.wav", label: "c" });
    expect(sanitizePads(new Array(20).fill({ path: "x.wav" }))).toHaveLength(PAD_COUNT);
  });

  it("assigns and clears one pad", () => {
    const a = withPad(emptyPads(), 2, "x/y/stinger.wav");
    expect(a[2]).toEqual({ path: "x/y/stinger.wav", label: "stinger" });
    expect(a[1]!.path).toBeNull();
    expect(withPad(a, 2, null)[2]!.path).toBeNull();
  });

  it("tells listeners when the pads change", () => {
    const s = new PadsStore();
    let seen = 0;
    const off = s.subscribe(() => seen++);
    s.set(withPad(s.current, 0, "a.wav"));
    off();
    s.set(emptyPads());
    expect(seen).toBe(2);
    expect(s.current[0]!.path).toBeNull();
  });
});
