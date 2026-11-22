// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript playhead key rules, update notice rules and insert-session wording against
// the same fixture the Rust test reads (shared-fixtures/playhead-update.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/playhead-update.json";
import { adjustSelection, headTarget, jumpTarget, type EdgeMove, type HeadMove } from "./playhead-keys";
import { highlights, plainNotes, updateDue } from "./update";
import { insertNotice, mixName } from "./insert-session";

describe("playhead, update and insert fixture", () => {
  it("head, jump and selection keys", () => {
    for (const c of fx.head) expect(headTarget(c.m as HeadMove, c.c as never)).toBe(c.out);
    for (const c of fx.jumps) expect(jumpTarget(c.text, c.rate, c.format as never, c.fps, c.end)).toBe(c.out);
    for (const c of fx.adj) expect(adjustSelection(c.sel, c.m as EdgeMove, c.step, c.end)).toEqual(c.out);
  });
  it("update rules", () => {
    for (const c of fx.due) expect(updateDue(c.enabled, c.last ?? NaN, c.now, c.every as never)).toBe(c.out);
    for (const c of fx.plain) expect(plainNotes(c.md)).toBe(c.out);
    for (const c of fx.hl) expect(highlights(c.md, c.max, c.width)).toEqual(c.out);
  });
  it("insert session wording", () => {
    for (const c of fx.mix) expect(mixName(c.p)).toBe(c.out);
    for (const c of fx.notices) expect(insertNotice(c.name, c)).toBe(c.out);
  });
});
