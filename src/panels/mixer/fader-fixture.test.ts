// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript fader rules must agree with shared-fixtures/fader.json,
// the same file the Rust tests read (src-tauri/tests/fader_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../../shared-fixtures/fader.json";
import { dbToFader, faderToDb, formatDb, formatPan } from "./fader";

describe("shared fader fixture", () => {
  it.each(fixture.dbToFader)("dbToFader: $db", (c) => {
    expect(dbToFader(c.db)).toBeCloseTo(c.want, 9);
  });
  it.each(fixture.faderToDb)("faderToDb: $pos", (c) => {
    expect(faderToDb(c.pos)).toBeCloseTo(c.want, 9);
  });
  it.each(fixture.formatDb)("formatDb: $db", (c) => {
    expect(formatDb(c.db)).toBe(c.want);
  });
  it.each(fixture.formatPan)("formatPan: $p", (c) => {
    expect(formatPan(c.p)).toBe(c.want);
  });
});
