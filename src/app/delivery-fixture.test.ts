// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript audiobook delivery check must agree with shared-fixtures/delivery.json,
// the same file the Rust tests read (src-tauri/tests/delivery_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/delivery.json";
import { acxCheck } from "./delivery";
import type { LoudnessReport } from "./loudness";

describe("shared delivery fixture", () => {
  it.each(fixture.acxCheck)("acxCheck: $name", (c) => {
    // A peak that is not measurable is stored as null; the core reports it as minus infinity.
    const peak = c.samplePeak ?? -Infinity;
    const report = { integrated: -20, integratedMono: -23, range: 5, truePeak: peak, samplePeak: peak, seconds: 60, cancelled: false, rms: c.rms, noiseFloor: c.noiseFloor } as LoudnessReport;
    if ("silentSeconds" in c) report.silentSeconds = c.silentSeconds as number;
    expect(acxCheck(report)).toEqual(c.want);
  });
});
