// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript track names must agree with shared-fixtures/track-names.json,
// the same file the Rust tests read (src-tauri/tests/track_names_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/track-names.json";
import { newTrackNames, trackNameFromFile } from "./track-names";

describe("shared track names fixture", () => {
  it.each(fixture.trackNameFromFile)("trackNameFromFile: $name", (c) => {
    expect(trackNameFromFile(c.name)).toBe(c.want);
  });
  it.each(fixture.newTrackNames)("newTrackNames: $existing $count $base", (c) => {
    expect(newTrackNames(c.existing, c.count, c.base)).toEqual(c.want);
  });
});
