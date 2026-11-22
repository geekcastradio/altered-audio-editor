// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript clip level and view rules must agree with
// shared-fixtures/clip-gain.json, the same file the Rust tests read
// (src-tauri/tests/clip_gain_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/clip-gain.json";
import { dragGain, gainBox, gainText } from "../panels/editor/clip-gain";
import { editSelection, viewFor, viewRange } from "../panels/time-info/time-info";

describe("shared clip gain fixture", () => {
  it.each(fixture.gainBox)("gainBox: $x0 $x1 $bottom $top", (c) => {
    expect(gainBox(c.x0, c.x1, c.bottom, c.top)).toEqual(c.want);
  });
  it.each(fixture.dragGain)("dragGain: $start $dx $dy $fine", (c) => {
    expect(dragGain(c.start, c.dx, c.dy, c.fine)).toBeCloseTo(c.want, 9);
  });
  it.each(fixture.gainText)("gainText: $db", (c) => {
    expect(gainText(c.db)).toBe(c.want);
  });
  it.each(fixture.viewRange)("viewRange: $v $width", (c) => {
    expect(viewRange(c.v as never, c.width)).toEqual(c.want);
  });
  it.each(fixture.viewFor)("viewFor: $start $end $width", (c) => {
    const got = viewFor({ scroll: 0, samples_per_pixel: 1 } as never, c.start, c.end, c.width);
    expect(got.scroll).toBeCloseTo(c.want.scroll, 9);
    expect(got.samples_per_pixel).toBeCloseTo(c.want.samples_per_pixel, 9);
  });
  it.each(fixture.editSelection)("editSelection: $sel $part $value", (c) => {
    expect(editSelection(c.sel, c.part as "start" | "end" | "duration", c.value)).toEqual(c.want);
  });
});
