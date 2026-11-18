// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** Roles whose widgets use the arrow and Page keys themselves. */
const ARROW_ROLES = new Set(["listbox", "option", "slider", "spinbutton", "tab", "tablist", "menu", "menuitem", "menubar", "tree", "treeitem", "grid", "gridcell", "radio", "radiogroup"]);

interface ElementLike {
  tagName?: string;
  type?: string;
  getAttribute?: (name: string) => string | null;
  closest?: (selector: string) => ElementLike | null;
}

/**
 * True when the focused control uses the bare arrow and Page keys itself (a slider, a drop-down, a list, a tab row),
 * so the playhead keys must leave them alone.
 */
export function ownsArrowKeys(target: unknown): boolean {
  const el = target as ElementLike | null;
  if (!el || typeof el.tagName !== "string") return false;
  const tag = el.tagName.toUpperCase();
  if (tag === "SELECT") return true;
  if (tag === "INPUT" && (el.type === "range" || el.type === "radio")) return true;
  const role = el.getAttribute?.("role");
  if (role && ARROW_ROLES.has(role)) return true;
  return false;
}
