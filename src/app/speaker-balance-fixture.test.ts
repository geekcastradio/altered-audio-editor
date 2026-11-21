// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// The TypeScript Speaker Balance arithmetic must agree with
// shared-fixtures/speaker-balance.json, the same file the Rust tests read
// (src-tauri/tests/speaker_balance_fixture.rs).

import { describe, expect, it } from "vitest";
import fixture from "../../shared-fixtures/speaker-balance.json";
import type { LoudnessReport } from "./loudness";
import { balance, describeBalance } from "./speaker-balance";
import type { SessionDoc } from "../stores/session";

const number = (v: unknown): number => (v === "NaN" ? NaN : v === "Infinity" ? Infinity : v === "-Infinity" ? -Infinity : (v as number));

describe("shared speaker balance fixture", () => {
  it.each(fixture.balance)("balance: $name", (c) => {
    const items = c.labels.map((label, i) => ({ key: `t${i}`, label, clipIds: [`c${i}`] }));
    const reports = c.reports.map((r) =>
      r ? ({ integrated: number(r.integrated), integratedMono: number(r.integrated), range: 0, truePeak: number(r.truePeak), samplePeak: 0, seconds: 1, cancelled: false } as LoudnessReport) : undefined,
    ) as LoudnessReport[];
    const b = balance({ tracks: [] } as unknown as SessionDoc, items, reports);
    b.rows.forEach((r, i) => {
      const w = c.want.rows[i]!;
      for (const [got, want] of [[r.measured, w.measured], [r.truePeak, w.truePeak], [r.change, w.change], [r.peakAfter, w.peakAfter]] as const) {
        const x = number(want);
        if (Number.isNaN(x)) expect(got).toBeNaN();
        else if (!Number.isFinite(x)) expect(got).toBe(x);
        else expect(got).toBeCloseTo(x, 9);
      }
      expect(r.measurable).toBe(w.measurable);
    });
    expect(b.reference ? b.rows.indexOf(b.reference) : null).toBe(c.want.reference);
    expect(describeBalance(b)).toBe(c.want.text);
  });
});
