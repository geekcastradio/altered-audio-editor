// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the bars and beats ruler in TypeScript against the fixture the Rust
// test reads too (shared-fixtures/bar-ruler.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/bar-ruler.json";
import { barTicks, formatBars, gridLines, parseBars } from "./timeline";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const ts = fx.tempos as never[];

describe("bar ruler fixture", () => {
  it("formats and parses bar.beat.tick", () => {
    for (const c of fx.fmt) expect(formatBars(c.s, 48000, ts[c.ti]!)).toBe(c.out);
    for (const c of fx.parse) expect(J(parseBars(c.text, 48000, ts[c.ti]!))).toEqual(c.out);
  });
  it("draws ticks and grid lines", () => {
    for (const c of fx.ticks) expect(J(barTicks({ spp: c.spp, scroll: c.scroll }, 900, 48000, ts[c.ti]!, (c as { gap?: number }).gap ?? 70))).toEqual(c.out);
    for (const c of fx.grid) expect(J(gridLines({ spp: c.spp, scroll: c.scroll }, 900, 48000, ts[c.ti]!, 6))).toEqual(c.out);
  });
});
