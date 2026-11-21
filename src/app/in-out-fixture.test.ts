// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript In and Out rules must agree with shared-fixtures/in-out.json,
// the same file the Rust tests read (src-tauri/tests/in_out_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/in-out.json";
import { clearIn, clearOut, markIn, markOut, type Span } from "./in-out";

type Pair = [number, number] | null;
const sel = (p: Pair): Span => (p ? { start: p[0], end: p[1] } : null);
const out = (s: Span): Pair => (s ? [s.start, s.end] : null);

describe("shared in and out fixture", () => {
  it.each(fixture.markIn)("markIn: $name", (c) => {
    expect(out(markIn(sel(c.sel as Pair), c.at, c.contentEnd))).toEqual(c.want);
  });
  it.each(fixture.markOut)("markOut: $name", (c) => {
    expect(out(markOut(sel(c.sel as Pair), c.at))).toEqual(c.want);
  });
  it.each(fixture.clearIn)("clearIn: $name", (c) => {
    expect(out(clearIn(sel(c.sel as Pair), c.contentEnd))).toEqual(c.want);
  });
  it.each(fixture.clearOut)("clearOut: $name", (c) => {
    expect(out(clearOut(sel(c.sel as Pair), c.contentEnd))).toEqual(c.want);
  });
});
