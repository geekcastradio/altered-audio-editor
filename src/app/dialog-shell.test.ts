// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";

// Every dialog is built on Dialog.svelte so they all look and behave alike: same
// panel, same buttons, Cancel first and the main action last, Escape to close, the
// first field focused. Dialogs not moved over yet are listed here. Moving one over
// means deleting its line from this list; the tests then hold it to the shell.
const LEGACY = new Set([
  "AnalysisDialog.svelte",
  "ChoiceDialog.svelte",
  "ControllersDialog.svelte",
  "KeyboardShortcutsDialog.svelte",
  "PluginManagerDialog.svelte",
]);

const files = import.meta.glob("./*Dialog.svelte", { eager: true, query: "?raw", import: "default" }) as Record<string, string>;
const dialogs = Object.entries(files)
  .map(([path, source]) => ({ name: path.replace("./", ""), source }))
  .filter((d) => d.name !== "Dialog.svelte");

describe("dialogs use the shared shell", () => {
  it("finds the dialogs", () => {
    expect(dialogs.length).toBeGreaterThan(20);
  });

  it("every legacy entry is a real dialog that still has its own backdrop", () => {
    for (const name of LEGACY) {
      const d = dialogs.find((x) => x.name === name);
      expect(d, `${name} is listed as legacy but does not exist`).toBeDefined();
      expect(d!.source.includes("<Dialog"), `${name} uses Dialog already: remove it from LEGACY`).toBe(false);
    }
  });

  it("every other dialog uses Dialog.svelte and has no backdrop of its own", () => {
    for (const d of dialogs.filter((x) => !LEGACY.has(x.name))) {
      expect(d.source, `${d.name} should import the shell`).toContain('import Dialog from "./Dialog.svelte"');
      expect(d.source, `${d.name} should use <Dialog>`).toContain("<Dialog");
      expect(d.source, `${d.name} should not define its own backdrop`).not.toMatch(/class="backdrop/);
    }
  });

  it("Cancel comes before the main action in every footer", () => {
    for (const d of dialogs.filter((x) => !LEGACY.has(x.name))) {
      const footer = /\{#snippet footer\(\)\}([\s\S]*?)\{\/snippet\}/.exec(d.source)?.[1] ?? "";
      const cancel = footer.search(/>(Cancel|Close)<\/button>/);
      const primary = footer.search(/class="primary"/);
      if (cancel >= 0 && primary >= 0) expect(cancel, `${d.name}: Cancel should come before the main action`).toBeLessThan(primary);
    }
  });
});
