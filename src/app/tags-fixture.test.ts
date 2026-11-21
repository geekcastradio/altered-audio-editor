// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript tag helpers must agree with shared-fixtures/tags.json, the
// same file the Rust tests read (src-tauri/tests/tags_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/tags.json";
import { TAGGABLE, cleanTags, saveTags } from "./tags";

describe("shared tags fixture", () => {
  it.each(fixture.clean)("cleanTags: $draft", (c) => {
    expect(cleanTags(c.draft)).toEqual(c.want);
  });
  it.each(fixture.save)("saveTags: $draft", (c) => {
    expect(saveTags(c.draft)).toEqual(c.want);
  });
  it.each(fixture.taggable)("TAGGABLE: $path", (c) => {
    expect(TAGGABLE.test(c.path)).toBe(c.want);
  });
});
