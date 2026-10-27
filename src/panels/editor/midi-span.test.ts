// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { midiSpan } from "./clip-edit";

describe("midiSpan", () => {
  const clip = { start: 4, length: 8 };

  it("uses the default 120 bpm when the session has no tempo", () => {
    // beat 4 is 2 s, beat 12 is 6 s
    expect(midiSpan({ sample_rate: 48_000 }, clip)).toEqual([96_000, 288_000]);
  });

  it("follows the session tempo", () => {
    // at 60 bpm a beat is a second: 4 s to 12 s
    const tempo = { bpm: 60, beats_per_bar: 4 };
    expect(midiSpan({ sample_rate: 48_000, tempo }, clip)).toEqual([192_000, 576_000]);
  });
});
