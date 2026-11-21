// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript dial maths must agree with shared-fixtures/knob.json,
// the same file the Rust tests read (src-tauri/tests/knob_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/knob.json";
import { dragValue, keyValue, pointerAngle } from "./knob";

describe("shared knob fixture", () => {
  it.each(fixture.dragValue)("dragValue: $name", (c) => {
    expect(dragValue(c.start, c.dy, { ctrl: c.ctrl, shift: c.shift })).toBeCloseTo(c.want, 9);
  });
  it.each(fixture.keyValue)("keyValue: $name", (c) => {
    const got = keyValue(c.value, c.key, c.home);
    if (c.want === null) expect(got).toBeNull();
    else expect(got).toBeCloseTo(c.want, 9);
  });
  it.each(fixture.pointerAngle)("pointerAngle: $value", (c) => {
    expect(pointerAngle(c.value)).toBeCloseTo(c.want, 9);
  });
});
