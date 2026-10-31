// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { waveRows } from "./wave-rows";

const cur = { path: "C:\\x\\Untitled 2.wav", format: "WAV", untitled: true };
const old = { path: "C:\\x\\Untitled 1.wav", format: "WAV", untitled: true, dirty: true };

describe("waveRows", () => {
  it("lists the open file first, then the ones set aside", () => {
    const r = waveRows(cur, false, [old], []);
    expect(r.map((x) => x.name)).toEqual(["Untitled 2.wav", "Untitled 1.wav"]);
    expect(r[0]!.front).toBe(true);
    expect(r[1]!.front).toBe(false);
    expect(r[1]!.dirty).toBe(true);
  });
  it("leaves out files the list already shows, ignoring case", () => {
    const r = waveRows(cur, false, [old], ["c:\\X\\untitled 1.WAV"]);
    expect(r.map((x) => x.name)).toEqual(["Untitled 2.wav"]);
  });
  it("is empty with nothing open", () => {
    expect(waveRows(null, false, [], [])).toEqual([]);
  });
});
