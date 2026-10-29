// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { plainNotes, updateDue, WEEK_MS } from "./update";

describe("updateDue", () => {
  it("never runs while the setting is off", () => {
    expect(updateDue(false, 0, 1e12)).toBe(false);
  });

  it("runs the first time it is on", () => {
    expect(updateDue(true, 0, 1e12)).toBe(true);
  });

  it("waits a week between checks", () => {
    const last = 1e12;
    expect(updateDue(true, last, last + WEEK_MS - 1)).toBe(false);
    expect(updateDue(true, last, last + WEEK_MS)).toBe(true);
  });

  it("runs when the clock went backwards", () => {
    expect(updateDue(true, 2e12, 1e12)).toBe(true);
  });
});

describe("plainNotes", () => {
  it("turns Markdown release notes into plain text", () => {
    const md = "# Altered Audio Editor 0.7.0\n\n## Added\n- **Duck Music.** Turns the music `down`.\n- See [the guide](https://x.y/z).\n\n\n\n## Fixed\n";
    expect(plainNotes(md)).toBe("Altered Audio Editor 0.7.0\n\nAdded\n\u2022 Duck Music. Turns the music down.\n\u2022 See the guide.\n\nFixed");
  });
});
