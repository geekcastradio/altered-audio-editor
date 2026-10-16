// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Routes key presses to actions through the keymap.
 */

import type { ActionRegistry } from "../app/actions";
import { chordFromEvent } from "./chord";
import type { Keymap } from "./keymap";

/** Actions that keep firing while the key is held. Everything else fires once per press. */
const REPEATABLE = new Set(["edit.undo", "edit.redo", "view.zoom-in", "view.zoom-out"]);

/**
 * Webview keys that would reload, print, or search the page. Blocked in
 * release builds so a stray F5 never throws away unsaved work.
 */
const BROWSER_KEYS = new Set(["F5", "Ctrl+R", "Ctrl+Shift+R", "Ctrl+P", "Ctrl+G", "Ctrl+Shift+G", "Ctrl+U", "F3", "F7"]);

/** True when typing in a text field, where the field's own keys should win. */
export function isTextEntry(target: EventTarget | null): boolean {
  if (typeof HTMLElement === "undefined" || !(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (target instanceof HTMLTextAreaElement) return true;
  if (target instanceof HTMLInputElement) {
    return !["button", "checkbox", "radio", "range", "color", "file"].includes(target.type);
  }
  return false;
}

export interface DispatchDecision {
  /** Action to run, if any. */
  action?: string;
  /** Stop the webview's own handling of this key. */
  prevent: boolean;
}

/** Decides what a key press does. Pure, so it can be tested without a DOM. */
export function decide(
  e: Parameters<typeof chordFromEvent>[0] & { repeat: boolean; defaultPrevented: boolean },
  keymap: Keymap,
  opts: { inTextField: boolean; modalOpen: boolean; release: boolean },
): DispatchDecision {
  if (e.defaultPrevented) return { prevent: false };
  const chord = chordFromEvent(e);
  if (!chord) return { prevent: false };

  const browserKey = opts.release && BROWSER_KEYS.has(chord);
  if (opts.inTextField) return { prevent: browserKey };
  // Outside text fields Ctrl+A never selects the page's text, even under a dialog.
  if (opts.modalOpen) return { prevent: browserKey || chord === "Ctrl+A" };

  const action = keymap.actionFor(chord);
  if (!action) return { prevent: browserKey };
  if (e.repeat && !REPEATABLE.has(action)) return { prevent: true };
  return { action, prevent: true };
}

/** Listens for key presses on the window. Returns a function that stops listening. */
export function bindKeymap(keymap: Keymap, actions: ActionRegistry, onError: (e: unknown) => void): () => void {
  const handler = (e: KeyboardEvent) => {
    const result = decide(e, keymap, {
      inTextField: isTextEntry(e.target),
      modalOpen: document.querySelector('[aria-modal="true"]') !== null,
      release: !import.meta.env.DEV,
    });
    if (result.prevent) e.preventDefault();
    if (result.action) actions.run(result.action).catch(onError);
  };
  window.addEventListener("keydown", handler);
  return () => window.removeEventListener("keydown", handler);
}
