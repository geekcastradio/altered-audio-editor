// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { lastSegment, packLibraryList, readLibraryList, sameDir, sanitizeKnown } from "./library-list";

describe("library list", () => {
  it("round-trips names, colors and folders", () => {
    const list = [
      { dir: "C:\\Libs\\Beds", name: "Beds", color: "--clip-3" },
      { dir: "D:\\Shows\\Promos", name: "Promos", color: null },
    ];
    expect(readLibraryList(packLibraryList(list))).toEqual(list);
  });

  it("refuses files that are not a list or are from a newer version", () => {
    expect(readLibraryList("nope")).toEqual({ error: "That file is not a library list." });
    expect(readLibraryList('{"format":"other","version":1}')).toEqual({ error: "That file is not an Altered Audio library list." });
    const newer = readLibraryList('{"format":"altaudio.libraries","version":9,"libraries":[]}');
    expect(newer).toEqual({ error: "That list was made by a newer version of Altered Audio. Update the app to read it." });
  });

  it("drops junk and duplicates and names an entry after its folder when it has no name", () => {
    const out = sanitizeKnown([
      { dir: "C:\\A\\One", name: "One" },
      { dir: "c:/a/one/", name: "Again" },
      { dir: "", name: "Blank" },
      { dir: "C:\\B\\Two" },
      "text",
      null,
      { dir: "C:\\C\\Three", name: "Three", color: "" },
    ]);
    expect(out).toEqual([
      { dir: "C:\\A\\One", name: "One", color: null },
      { dir: "C:\\B\\Two", name: "Two", color: null },
      { dir: "C:\\C\\Three", name: "Three", color: null },
    ]);
    expect(sanitizeKnown("x")).toEqual([]);
  });

  it("compares folders in either slash style and ignores case", () => {
    expect(sameDir("C:\\Libs\\Beds", "c:/libs/beds/")).toBe(true);
    expect(sameDir("C:\\Libs\\Beds", "C:\\Libs\\Bed")).toBe(false);
    expect(lastSegment("C:\\Libs\\Beds\\")).toBe("Beds");
  });
});
