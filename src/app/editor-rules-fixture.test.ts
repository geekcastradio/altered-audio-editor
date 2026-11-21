// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript editing rules must agree with shared-fixtures/editor-rules.json,
// the same file the Rust tests read (src-tauri/tests/editor_rules_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/editor-rules.json";
import { followEffect, maxScroll } from "../panels/editor/follow";
import { anchorFor, edgeAt, type Edge } from "../panels/editor/selection-edge";
import { silenceSpan } from "../panels/editor/silence-span";
import type { EnvelopePointDoc } from "../stores/session";

describe("shared editor rules fixture", () => {
  it.each(fixture.follow)("followEffect: $now $change", (c) => {
    expect(followEffect(c.now, c.change)).toBe(c.want);
  });
  it.each(fixture.maxScroll)("maxScroll: $total $liveEnd $following", (c) => {
    const base = { total: c.total, width: c.width, spp: c.spp, liveEnd: c.liveEnd, following: c.following };
    expect(maxScroll("lead" in c ? { ...base, lead: c.lead as number } : base)).toBeCloseTo(c.want, 9);
  });
  it.each(fixture.edge)("edgeAt: $sel $x $view", (c) => {
    expect("grab" in c ? edgeAt(c.sel, c.x, c.view, c.grab as number) : edgeAt(c.sel, c.x, c.view)).toBe(c.want);
  });
  it.each(fixture.anchor)("anchorFor: $sel $edge", (c) => {
    expect(anchorFor(c.sel, c.edge as Edge)).toBe(c.want);
  });
  it.each(fixture.silence)("silenceSpan: $len $from $to $ramp", (c) => {
    const got = silenceSpan(c.points as EnvelopePointDoc[], c.len, c.from, c.to, c.ramp);
    if (c.want === null) {
      expect(got).toBeNull();
      return;
    }
    expect(got).toEqual(c.want);
  });
});
