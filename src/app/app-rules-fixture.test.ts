// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript pitch edit choices, welcome plan and bug report text against the
// same fixture the Rust tests read (shared-fixtures/app-rules.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/app-rules.json";
import {
  describeReport,
  editFor,
  isNoChange,
  sanitizeChoices,
} from "./pitch-edit";
import { welcomePlan } from "./welcome";
import { bugInfoText, recordError, recordedErrors, scrub } from "./bug-info";

describe("app rules fixture", () => {
  it("pitch choices", () => {
    for (const c of fx.pitch) {
      const got = sanitizeChoices(c.raw);
      expect(got).toEqual(c.c);
      expect(editFor(got)).toEqual(c.edit);
      expect(isNoChange(got)).toBe(c.noChange);
    }
  });
  it("report text", () => {
    for (const r of fx.reports) expect(describeReport(r)).toBe(r.text);
  });
  it("welcome plan", () => {
    for (const w of fx.welcome) {
      const pages = w.revs.map((rev, i) => ({
        rev,
        title: String(i),
        body: [],
      }));
      const p = welcomePlan(w.saved, pages);
      expect(
        p ? { kind: p.kind, idx: p.pages.map((x) => Number(x.title)) } : null,
      ).toEqual(w.expect);
    }
  });
  it("scrub", () => {
    for (const s of fx.scrubs) expect(scrub(s.in)).toBe(s.out);
  });
  it("recorded errors", () => {
    for (const r of fx.records) {
      const e = new Error(r.message);
      e.stack = r.stack;
      recordError(e);
      const last = recordedErrors()[recordedErrors().length - 1]!;
      expect(last.message).toBe(r.msg);
      expect(last.stack).toBe(r.stk);
    }
  });
  it("bug text", () => {
    for (const b of fx.bug) expect(bugInfoText(b.ctx, b.list)).toBe(b.out);
  });
});
