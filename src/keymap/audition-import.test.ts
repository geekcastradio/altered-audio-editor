// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { applyAudition, parseAudition } from "./audition-import";
import { DEFAULT_BINDINGS } from "./keymap";

const wrap = (...cmds: string[]) => `<?xml version='1.0'?><keymap app="Audition" version="26.5.0">${cmds.join("")}</keymap>`;
const cmd = (name: string, key: string, mods: string[] = [], extra = "") =>
  `<keycmd><cmd>${name}</cmd>${key}${mods.length ? `<mod>${mods.map((m) => `<${m}/>`).join("")}</mod>` : ""}${extra}</keycmd>`;

describe("Audition shortcut import", () => {
  it("reads characters, key numbers and modifiers, and skips repeats", () => {
    const r = parseAudition(
      wrap(
        cmd("Edit.SelectAll", "<uchar>A</uchar>", ["cmdkey"]),
        cmd("Edit.Redo", "<uchar>z</uchar>", ["cmdkey", "shift"]),
        cmd("Edit.Delete", "<key>35</key>"),
        cmd("View.ZoomInTime", "<uchar>=</uchar>", [], "<hold/>"),
      ),
    );
    expect(r.bindings["edit.select-all"]).toEqual(["Ctrl+A"]);
    expect(r.bindings["edit.redo"]).toEqual(["Ctrl+Shift+Z"]);
    expect(r.bindings["view.zoom-in"]).toBeUndefined();
  });

  it("lists commands with no action here, and keys it cannot read", () => {
    const r = parseAudition(wrap(cmd("Transport.ShuttleLeft", "<uchar>J</uchar>"), cmd("Edit.SelectAll", "<key>999</key>")));
    expect(r.unmapped).toEqual(["Transport.ShuttleLeft"]);
    expect(r.unreadable).toBe(1);
  });

  it("refuses text that is not an export", () => {
    expect(() => parseAudition("<html></html>")).toThrow();
  });

  it("gives imported actions the export's chords and takes a chord from whoever else held it", () => {
    const { overrides, takenFrom } = applyAudition({ "edit.select-all": ["Ctrl+Shift+Q"] }, {}, DEFAULT_BINDINGS);
    expect(overrides["edit.select-all"]).toEqual(["Ctrl+Shift+Q", "Ctrl+A"]);
    expect(takenFrom).toEqual([]);
    const second = applyAudition({ "edit.select-all": ["Ctrl+Z"] }, {}, DEFAULT_BINDINGS);
    expect(second.takenFrom).toEqual([{ action: "edit.undo", chord: "Ctrl+Z" }]);
    expect(second.overrides["edit.undo"]).not.toContain("Ctrl+Z");
  });

  it("keeps our own extra keys that Audition does not use", () => {
    const r = applyAudition({ "file.close": ["Ctrl+W"] }, {}, DEFAULT_BINDINGS);
    expect(r.overrides["file.close"]).toBeUndefined();
  });

  it("leaves no override when the export matches our defaults", () => {
    const r = applyAudition({ "edit.undo": [...(DEFAULT_BINDINGS["edit.undo"] ?? [])] }, { "edit.undo": ["Ctrl+Alt+U"] }, DEFAULT_BINDINGS);
    expect(r.overrides["edit.undo"]).toBeUndefined();
  });
});
