// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { addPaths, checkTrack, checkYear, filterEntries, overlay, problems, removeSet, sanitizeSets, upsertSet } from "./metadata";

describe("tag sets", () => {
  it("keeps only well-formed sets and drops blank fields and repeated names", () => {
    const sets = sanitizeSets([
      { name: " GCRN Daily ", tags: { artist: "The GCRN", album: " ", year: 2026, genre: "Podcast" } },
      { name: "gcrn daily", tags: { artist: "other" } },
      { name: "", tags: { artist: "x" } },
      { name: "Empty", tags: {} },
      "junk",
      null,
    ]);
    expect(sets).toEqual([{ name: "GCRN Daily", tags: { artist: "The GCRN", genre: "Podcast" } }]);
    expect(sanitizeSets("nope")).toEqual([]);
  });

  it("saving under an existing name replaces it, and the list stays sorted", () => {
    let sets = upsertSet([], "Zeta", { artist: "Z" });
    sets = upsertSet(sets, "alpha", { artist: "A" });
    sets = upsertSet(sets, "ZETA", { artist: "Z2", album: "  " });
    expect(sets.map((s) => s.name)).toEqual(["alpha", "ZETA"]);
    expect(sets[1]?.tags).toEqual({ artist: "Z2" });
    expect(upsertSet(sets, "Blank", { artist: " " })).toBe(sets);
    expect(upsertSet(sets, " ", { artist: "A" })).toBe(sets);
    expect(removeSet(sets, "alpha").map((s) => s.name)).toEqual(["ZETA"]);
  });
});

describe("field checks", () => {
  it("accepts blank, four-digit years, and track numbers with an optional total", () => {
    expect(checkYear("")).toBeNull();
    expect(checkYear("2026")).toBeNull();
    expect(checkYear("26")).not.toBeNull();
    expect(checkYear("0999")).not.toBeNull();
    expect(checkTrack("3")).toBeNull();
    expect(checkTrack("3/12")).toBeNull();
    expect(checkTrack("3 of 12")).not.toBeNull();
    expect(problems({ year: "abcd", track: "1/2/3" })).toHaveLength(2);
    expect(problems({ year: "2026" })).toEqual([]);
  });
});

describe("the file list", () => {
  const entries = [
    { path: "C:\\shows\\ep1.mp3", tags: { title: "Pilot", artist: "GCRN" } },
    { path: "C:\\shows\\ep2.mp3", tags: { title: "Second", album: "Season One" } },
  ];
  it("filters by every word, across name and tags, ignoring case", () => {
    expect(filterEntries(entries, "")).toHaveLength(2);
    expect(filterEntries(entries, "season one").map((e) => e.path)).toEqual(["C:\\shows\\ep2.mp3"]);
    expect(filterEntries(entries, "EP1 gcrn")).toHaveLength(1);
    expect(filterEntries(entries, "nothing")).toHaveLength(0);
  });
  it("does not list the same file twice, whatever the slash or case", () => {
    expect(addPaths(["C:\\a\\b.mp3"], ["c:/a/B.mp3", "C:\\a\\c.mp3", "C:\\a\\c.mp3"])).toEqual(["C:\\a\\b.mp3", "C:\\a\\c.mp3"]);
  });
  it("a set shown over a file keeps the fields the set lacks", () => {
    expect(overlay({ title: "Pilot", artist: "Old" }, { name: "s", tags: { artist: "New" } })).toEqual({ title: "Pilot", artist: "New" });
  });
});
