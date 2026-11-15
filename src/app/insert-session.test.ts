// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { insertNotice, mixName } from "./insert-session";

describe("mixName", () => {
  it("uses the file name without its ending", () => {
    expect(mixName("C:\\Shows\\Ep 12 Intro.altaudio")).toBe("Ep 12 Intro Mix");
    expect(mixName("/shows/theme.altaudio")).toBe("theme Mix");
  });
});

describe("insertNotice", () => {
  it("says what was inserted and that it is a copy", () => {
    expect(insertNotice("Intro Mix", { missing: 0, peak: 0.5 })).toBe(
      'Inserted "Intro Mix" as one clip. It is a copy of the session as it was saved; insert it again to pick up later changes.',
    );
  });
  it("adds the missing-files and over-zero warnings", () => {
    const t = insertNotice("A Mix", { missing: 2, peak: 1.4 });
    expect(t).toContain("2 audio files it uses could not be found");
    expect(t).toContain("above 0 dBFS");
    expect(insertNotice("A Mix", { missing: 1, peak: 0 })).toContain("1 audio file it uses");
  });
});
