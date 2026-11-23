// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Reads the same fixture as src-tauri/tests/key_dispatch_fixture.rs.
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { decide } from "./dispatch";
import { ownsArrowKeys } from "./owns-arrows";

const fx = JSON.parse(readFileSync(new URL("../../shared-fixtures/key-dispatch.json", import.meta.url), "utf8"));

describe("key dispatch fixture", () => {
  it("decides every key press the same as Rust", () => {
    const keymap = { actionFor: (c: string) => fx.map[c] } as never;
    for (const c of fx.cases) {
      const out = decide({ ...c.ev, repeat: c.repeat, defaultPrevented: c.dp }, keymap, { inTextField: c.inTextField, modalOpen: c.modalOpen, release: c.release });
      expect({ action: out.action ?? null, prevent: out.prevent }).toEqual(c.out);
    }
  });
  it("knows which controls own the arrow keys", () => {
    for (const c of fx.owns) {
      const el = c.tag === null ? null : { tagName: c.tag, type: c.type ?? undefined, getAttribute: (n: string) => (n === "role" ? c.role : null) };
      expect(ownsArrowKeys(el)).toBe(c.out);
    }
  });
});
