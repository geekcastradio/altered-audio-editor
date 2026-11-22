// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript output format rules must agree with
// shared-fixtures/output-format.json, the same file the Rust tests read
// (src-tauri/tests/output_format_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/output-format.json";
import * as O from "./output-format";

const fmt = (raw: unknown): O.OutputFormat => O.sanitizeFormat(raw);
const kinds = ["wav", "flac", "mp3", "m4a", "aac"] as const;

describe("shared output format fixture", () => {
  it("reading and lookup", () => {
    for (const c of fixture.sanitize) expect(O.sanitizeFormat(c.raw)).toEqual(c.want);
    for (const c of fixture.depths) expect(O.depthsFor(c.kind as O.FormatKind)).toEqual(c.want);
    for (const c of fixture.paths) expect(O.kindOfPath(c.path)).toBe(c.want);
    for (const c of fixture.avail) expect(O.formatAvailable(c.kind as O.FormatKind, c.platform)).toBe(c.want);
  });
  it("text and size", () => {
    for (const c of fixture.describe) expect(O.describeFormat(fmt(c.f), c.src)).toBe(c.want);
    for (const c of fixture.sample) expect(O.describeSampleType(fmt(c.f), c.src)).toBe(c.want);
    for (const c of fixture.bytes) expect(O.estimateBytes(fmt(c.f), c.seconds, c.src)).toBeCloseTo(c.want, 4);
    for (const c of fixture.settings) {
      expect(O.describeSettings(fmt(c.f))).toBe(c.want);
      expect(O.ditherNote(fmt(c.f))).toBe(c.note);
    }
  });
  it("memory", () => {
    for (const c of fixture.memory) expect(O.sanitizeMemory(c.raw)).toEqual(c.want);
    for (const c of fixture.switch) {
      const got = O.switchFormat(c.memory as O.FormatMemory, fmt(c.from), c.to as O.FormatKind);
      expect(got).toEqual(c.want);
    }
    expect(kinds.length).toBe(5);
  });
});
