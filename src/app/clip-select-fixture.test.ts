// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript clip selection rules must agree with
// shared-fixtures/clip-select.json, the same file the Rust tests read
// (src-tauri/tests/clip_select_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/clip-select.json";
import { anchorStart, clipsFrom, clipsUpTo, nextClipId } from "./clip-select";

const sets = fixture.sets as Record<string, { id: string; start: number }[]>;
const clipsOf = (c: { set: string }) => sets[c.set]!;

describe("shared clip selection fixture", () => {
  it("anchorStart", () => {
    for (const c of fixture.anchorStart) {
      expect(
        anchorStart(clipsOf(c), new Set(c.chosen), c.playhead),
        JSON.stringify(c),
      ).toBeCloseTo(c.want, 9);
    }
  });
  it("clipsFrom and clipsUpTo", () => {
    for (const c of fixture.clipsFrom)
      expect(clipsFrom(clipsOf(c), c.at), JSON.stringify(c)).toEqual(c.want);
    for (const c of fixture.clipsUpTo)
      expect(clipsUpTo(clipsOf(c), c.at), JSON.stringify(c)).toEqual(c.want);
  });
  it("nextClipId", () => {
    for (const c of fixture.nextClipId) {
      expect(
        nextClipId(clipsOf(c), new Set(c.chosen), c.playhead),
        JSON.stringify(c),
      ).toBe(c.want);
    }
  });
});
