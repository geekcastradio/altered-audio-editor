// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript Reduce Bleed notice rules must agree with
// shared-fixtures/bleed-notice.json, the same file the Rust tests read
// (src-tauri/tests/bleed_notice_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/bleed-notice.json";
import { bleedSummary, bleedWork } from "./bleed";
import { mixdownBytes } from "./session-properties";
import type { ClipDoc } from "../stores/session";

describe("shared bleed notice fixture", () => {
  it.each(fixture.work)("bleedWork: $clips", (c) => {
    const clips = c.clips.map((f, i) => ({ id: `c${i}`, locked: f.locked, looped: f.looped, stretch: f.stretched ? { ratio: 1.1 } : undefined }) as unknown as ClipDoc);
    const w = bleedWork(clips);
    expect(w.work.map((x) => Number(x.id.slice(1)))).toEqual(c.want.work);
    expect(w.locked).toBe(c.want.locked);
    expect(w.unsuitable).toBe(c.want.unsuitable);
  });
  it.each(fixture.summary)("bleedSummary: $removed $locked $unsuitable $changed", (c) => {
    expect(bleedSummary(c.removed, { locked: c.locked, unsuitable: c.unsuitable }, c.changed)).toBe(c.want);
  });
  it.each(fixture.mixdown)("mixdownBytes: $samples $bits $channels", (c) => {
    expect(mixdownBytes(c.samples, c.bits, c.channels)).toBeCloseTo(c.want, 9);
  });
});
