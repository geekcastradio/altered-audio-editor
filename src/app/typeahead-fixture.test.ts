// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript type-to-jump rules must agree with shared-fixtures/typeahead.json,
// the same file the Rust tests read (src-tauri/tests/typeahead_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/typeahead.json";
import { isTypingKey, typeahead } from "./typeahead";

describe("shared typeahead fixture", () => {
  it.each(fixture.sequences)("sequence: $name", (s) => {
    const t = typeahead();
    for (const st of s.steps) expect(t.next(st.key, s.labels, st.current, st.now)).toBe(st.want);
  });
  it.each(fixture.typingKeys)("isTypingKey: $key $ctrl $alt $meta", (c) => {
    expect(isTypingKey({ key: c.key, ctrlKey: c.ctrl, altKey: c.alt, metaKey: c.meta })).toBe(c.want);
  });
});
