// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript time input rules must agree with shared-fixtures/time-input.json,
// the same file the Rust tests read (src-tauri/tests/time_input_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../../shared-fixtures/time-input.json";
import { editSelection, parseTime } from "./time-info";

type Format = "time" | "samples" | "timecode" | "seconds";

describe("shared time input fixture", () => {
  it.each(fixture.parseTime)("parseTime: $name", (c) => {
    expect(parseTime(c.in, c.rate, c.format as Format, c.fps)).toBe(c.want);
  });
  it.each(fixture.editSelection)("editSelection: $name", (c) => {
    const out = editSelection({ start: c.sel[0]!, end: c.sel[1]! }, c.part as "start" | "end" | "duration", c.value);
    expect([out.start, out.end]).toEqual(c.want);
  });
});
