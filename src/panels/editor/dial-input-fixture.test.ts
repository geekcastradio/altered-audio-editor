// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript dial input rules must agree with shared-fixtures/dial-input.json,
// the same file the Rust tests read (src-tauri/tests/dial_input_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../../shared-fixtures/dial-input.json";
import { parseNumber, parsePan } from "./dial-input";

describe("shared dial input fixture", () => {
  it.each(fixture.parseNumber)("parseNumber: $in", (c) => {
    expect(parseNumber(c.in)).toBe(c.want);
  });
  it.each(fixture.parsePan)("parsePan: $in", (c) => {
    expect(parsePan(c.in)).toBe(c.want);
  });
});
