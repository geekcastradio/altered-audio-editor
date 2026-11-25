// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Favorites: the commands a person pins for quick access. The list holds action ids, in the
 * order they were pinned. It starts empty and nothing is pinned for them. The Quick Actions bar
 * and the Favorites menu both show this one list.
 */

import type { Menu, MenuItem } from "./menus/model";

/** Most commands one person can pin. Enough for a busy bar, and the Favorites menu has a slot for each. */
export const MAX_FAVORITES = 24;

/** Prefix of the Favorites menu's own rows, which are never pinned themselves. */
export const FAVORITE_SLOT_PREFIX = "favorites.slot.";

/** Whether a menu command can be pinned. The Favorites rows, Open Recent rows and empty slots cannot. */
export function isPinnable(id: string): boolean {
  return id !== "" && !id.startsWith("favorites.") && !id.startsWith("file.recent.");
}

/** Cleans a saved list: text ids only, no repeats, nothing unpinnable, at most MAX_FAVORITES. */
export function sanitizeFavorites(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  const out: string[] = [];
  for (const v of value) {
    if (typeof v === "string" && isPinnable(v) && !out.includes(v)) out.push(v);
    if (out.length >= MAX_FAVORITES) break;
  }
  return out;
}

/** The list with `id` added at the end, or removed when it is already there. A full list stays as it is. */
export function toggleFavorite(list: readonly string[], id: string): string[] {
  if (list.includes(id)) return list.filter((x) => x !== id);
  if (!isPinnable(id) || list.length >= MAX_FAVORITES) return [...list];
  return [...list, id];
}

/** The text a button shows: the menu label without the trailing dots that mean "asks first". */
export function shortLabel(label: string): string {
  return label.replace(/\s*(\.{3}|…)\s*$/, "").trim();
}

/** Menu label by action id, from every menu and submenu. The first label found for an id wins. */
export function menuLabels(menus: readonly Menu[]): Map<string, string> {
  const out = new Map<string, string>();
  const walk = (items: readonly MenuItem[]) => {
    for (const it of items) {
      if (it.kind === "item" && !out.has(it.id)) out.set(it.id, it.label);
      else if (it.kind === "submenu") walk(it.items);
    }
  };
  for (const m of menus) walk(m.items);
  return out;
}
