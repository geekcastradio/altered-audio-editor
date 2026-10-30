// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Brings shortcuts in from an Audition export (Edit > Keyboard Shortcuts > Save As, an XML
 * file). Each Audition command that has a matching action here takes the chords the export
 * holds for it. Commands with no matching action, and keys this reader can't decode, are
 * listed so nothing is lost silently.
 */

import map from "./audition-map.json";
import { parseChord } from "./chord";
import type { Bindings } from "./keymap";

/** Audition's numbers for keys that have no character. */
const KEY_NUMBERS: Record<number, string> = {
  1: "Space",
  2: "Backspace",
  3: "Tab",
  35: "Delete",
  36: "Home",
  37: "End",
  38: "PageUp",
  39: "PageDown",
  42: "ArrowLeft",
  43: "ArrowRight",
  44: "ArrowUp",
  45: "ArrowDown",
};

const MODS: Record<string, string> = { cmdkey: "Ctrl", alt: "Alt", shift: "Shift" };

export interface AuditionImport {
  /** Our action id to the chords Audition gives its command. */
  bindings: Bindings;
  /** Audition commands with a shortcut and no matching action here. */
  unmapped: string[];
  /** Entries whose key could not be read. */
  unreadable: number;
}

const ACTIONS = map as Record<string, string>;

/** Reads an Audition shortcut export. Throws when the text is not one. */
export function parseAudition(xml: string): AuditionImport {
  if (!/<keymap[\s>]/.test(xml)) throw new Error("That isn't an Audition shortcut export.");
  const bindings: Bindings = {};
  const unmapped = new Set<string>();
  let unreadable = 0;
  for (const block of xml.matchAll(/<keycmd>([\s\S]*?)<\/keycmd>/g)) {
    const body = block[1] ?? "";
    // Hold and extended-keypad repeats are the same shortcut again.
    if (/<hold\s*\/>|<ext\s*\/>/.test(body)) continue;
    const cmd = /<cmd>([^<]+)<\/cmd>/.exec(body)?.[1]?.trim();
    if (!cmd) continue;
    const mods = [...(/<mod>([\s\S]*?)<\/mod>/.exec(body)?.[1] ?? "").matchAll(/<(\w+)\s*\/>/g)].map((m) => MODS[m[1] ?? ""]).filter(Boolean);
    const ch = /<uchar>([^<]+)<\/uchar>/.exec(body)?.[1];
    const num = /<key>(\d+)<\/key>/.exec(body)?.[1];
    const key = ch !== undefined ? unescapeXml(ch) : num !== undefined ? KEY_NUMBERS[Number(num)] : undefined;
    const chord = key ? parseChord([...mods, key].join("+")) : null;
    if (!chord) {
      unreadable++;
      continue;
    }
    const action = ACTIONS[cmd];
    if (!action) {
      unmapped.add(cmd);
      continue;
    }
    const list = (bindings[action] ??= []);
    if (!list.includes(chord)) list.push(chord);
  }
  return { bindings, unmapped: [...unmapped].sort(), unreadable };
}

function unescapeXml(s: string): string {
  return s.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&apos;/g, "'").replace(/&amp;/g, "&");
}

/** The same chords, whatever their order. */
const same = (a: readonly string[], b: readonly string[]) => a.length === b.length && a.every((x) => b.includes(x));

export interface Applied {
  overrides: Bindings;
  /** Actions that gave up a chord because Audition uses it for another command. */
  takenFrom: { action: string; chord: string }[];
}

/**
 * The user's changes after bringing the export in. Every imported action gets Audition's chords
 * first, then keeps any of our own default chords that no imported command wants (so our extra
 * Close key still works). Any other action holding an imported chord loses it, so no chord ever
 * runs two things.
 */
export function applyAudition(imported: Bindings, current: Bindings, defaults: Bindings): Applied {
  const overrides: Bindings = { ...current };
  const owner = new Map<string, string>();
  for (const [id, chords] of Object.entries(imported)) for (const c of chords) owner.set(c, id);
  for (const [id, chords] of Object.entries(imported)) {
    const merged = [...chords, ...(defaults[id] ?? []).filter((c) => !chords.includes(c) && !owner.has(c))];
    if (same(merged, defaults[id] ?? [])) delete overrides[id];
    else overrides[id] = merged;
  }
  const takenFrom: Applied["takenFrom"] = [];
  const ids = new Set([...Object.keys(defaults), ...Object.keys(overrides)]);
  for (const id of ids) {
    if (id in imported) continue;
    const chords = overrides[id] ?? defaults[id] ?? [];
    const kept = chords.filter((c) => !owner.has(c));
    if (kept.length === chords.length) continue;
    for (const c of chords) if (owner.has(c)) takenFrom.push({ action: id, chord: c });
    if (same(kept, defaults[id] ?? [])) delete overrides[id];
    else overrides[id] = kept;
  }
  return { overrides, takenFrom };
}
