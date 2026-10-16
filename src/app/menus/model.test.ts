// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { MENUS, commandList, menuActionIds, type MenuItem } from "./model";

function allItems(items: readonly MenuItem[]): MenuItem[] {
  return items.flatMap((it) => (it.kind === "submenu" ? [it, ...allItems(it.items)] : [it]));
}

describe("menu model", () => {
  it("has a mnemonic letter in every menu label, all different", () => {
    const letters = MENUS.map((m) => m.mnemonic.toLowerCase());
    expect(new Set(letters).size).toBe(letters.length);
    for (const m of MENUS) expect(m.label.toLowerCase()).toContain(m.mnemonic.toLowerCase());
  });

  it("never starts, ends or doubles a separator", () => {
    const check = (items: readonly MenuItem[]) => {
      items.forEach((it, i) => {
        if (it.kind === "sep") {
          expect(i).toBeGreaterThan(0);
          expect(i).toBeLessThan(items.length - 1);
          expect(items[i + 1]?.kind).not.toBe("sep");
        }
        if (it.kind === "submenu") check(it.items);
      });
    };
    for (const m of MENUS) check(m.items);
  });

  // The list of names to keep out lives in private/banned-terms.txt (one
  // regular expression per line), which is never committed. Without it,
  // for example in CI, the check is skipped.
  const found = import.meta.glob<string>("../../../private/banned-terms.txt", { query: "?raw", import: "default", eager: true });
  const bannedText = Object.values(found)[0];
  it.skipIf(bannedText === undefined)("does not use other products' feature names", () => {
    const terms = (bannedText ?? "")
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter(Boolean);
    const banned = new RegExp(terms.join("|"), "i");
    for (const m of MENUS) {
      for (const it of allItems(m.items)) {
        if (it.kind !== "sep") expect(it.label).not.toMatch(banned);
      }
    }
  });

  it("gives every unbuilt item a reason", () => {
    const built = new Set([
      "edit.undo",
      "edit.redo",
      "file.exit",
      "window.reset-layout",
      "window.minimize",
      "edit.keyboard-shortcuts",
      "help.shortcuts",
      "help.website",
      "help.about",
      "file.import-files",
      "file.import-folder",
      "file.reveal",
      "edit.delete",
      "edit.select-all",
      "edit.deselect",
      "edit.find",
      "multitrack.add.mono",
      "multitrack.add.stereo",
      "multitrack.duplicate-track",
      "multitrack.delete-track",
      "multitrack.delete-empty",
      "multitrack.insert-files",
      "multitrack.insert-tracks",
      "multitrack.insert-selected-tracks",
      "view.editor.multitrack",
      "view.zoom-in",
      "view.zoom-out",
      "view.zoom-session",
      "view.time.decimal",
      "view.time.samples",
      "clip.reveal",
      "file.close-unused",
      "edit.select-track-clips",
      "file.new-session",
      "file.open",
      "file.save",
      "file.save-as",
      "file.save-all",
      "file.close",
      "file.close-all",
      "file.import-sesx",
      "multitrack.play-pause",
      "multitrack.stop",
      "multitrack.properties",
      "clip.split",
      "clip.split-all",
      "clip.gain",
      "clip.lock",
      "clip.mute",
      "clip.fade-in",
      "clip.fade-out",
      "clip.trim.start",
      "clip.trim.end",
      "view.time.timecode",
      "file.export.stems",
      "clip.group",
      "help.book",
      "help.donate",
      "clip.merge",
      "clip.unique",
      "clip.loop",
      "view.editor.waveform",
      "clip.edit-source",
      "fx.silence",
      "fx.amplify",
      "fx.normalize",
      "fx.spectrum.reduce-12",
      "fx.spectrum.reduce-24",
      "fx.spectrum.remove",
      "fx.spectrum.repair",
      "view.spectrum",
      "view.pitch",
      "multitrack.record",
      "tool.box",
      "tool.lasso",
      "tool.brush",
      "tool.spot",
      "fx.fade-in",
      "fx.fade-out",
      "edit.crop",
      "edit.paste-new",
      "file.new-audio",
      "file.save-template",
      "edit.mix-paste",
      "tool.move",
      "tool.razor",
      "tool.slip",
      "tool.time",
      "file.check-loudness",
      "view.env.volume",
      "view.env.pan",
      "clip.ungroup",
      "edit.marker.cue",
      "edit.marker.range",
      "edit.snap.markers",
      "edit.snap.ruler",
      "clip.auto-crossfade",
      "clip.color",
      "file.export.mixdown",
      "file.export.selection",
      "multitrack.mixdown.all",
      "multitrack.mixdown.selection",
      "multitrack.mixdown.clips",
      "multitrack.bounce.selection",
      "multitrack.bounce.clips",
      "edit.cut",
      "edit.copy",
      "edit.paste",
      "edit.ripple.clips",
      "edit.ripple.time-track",
      "edit.ripple.time-all",
      "edit.ripple.gap",
      "edit.select-to-end",
      "edit.select-next-clip",
      "edit.select-clips-track-end",
      "edit.select-clips-session-start",
      "edit.select-clips-session-end",
      "edit.select-view-time",
      "edit.select-all-time",
      "edit.select-time-around-clips",
      "edit.clear-time-selection",
      "edit.snap.on",
      "edit.snap.clips",
      "edit.snap.playhead",
      "clip.trim.selection",
      "view.zoom-selection",
    ]);
    for (const m of MENUS) {
      for (const it of allItems(m.items)) {
        if (it.kind !== "item" || built.has(it.id) || it.id.startsWith("window.panel.") || it.id.startsWith("file.recent.") || it.id.startsWith("window.workspace.")) continue;
        expect(it.pending, it.id).toBeTruthy();
      }
    }
  });

  it("keeps the Window menu with every panel", () => {
    const win = MENUS.find((m) => m.label === "Window");
    expect(win).toBeDefined();
    const ids = win!.items.flatMap((it) => (it.kind === "item" ? [it.id] : []));
    expect(ids).toContain("window.panel.history");
    expect(ids).toContain("window.reset-layout");
  });

  it("lists bindable commands once each, without placeholders", () => {
    const list = commandList();
    const ids = list.map((c) => c.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).not.toContain("fx.dynamics");
    expect(list.find((c) => c.id === "edit.select-all")?.path).toBe("Edit > Select > Select All");
  });

  it("lists action ids", () => {
    const ids = menuActionIds();
    expect(ids).toContain("edit.undo");
    expect(ids).toContain("window.panel.libraries");
  });
});
