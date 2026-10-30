// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * How the Libraries panel lays items out: filtered by kind, sorted, and grouped into
 * sections (by bin, by type, or not at all). Pure, so it can be tested without a screen.
 */

import type { Library, LibraryCollection, LibraryItem } from "../../app/core";

export type GroupBy = "bin" | "type" | "none";
export type SortBy = "custom" | "name" | "date";
export type KindFilter = "all" | "audio" | "image";
export type ViewMode = "list" | "grid";

export interface ViewState {
  groupBy: GroupBy;
  sortBy: SortBy;
  filter: KindFilter;
  view: ViewMode;
  /** Section ids folded shut. */
  collapsed: string[];
}

export const DEFAULT_VIEW: ViewState = { groupBy: "bin", sortBy: "custom", filter: "all", view: "list", collapsed: [] };

/** A saved view, with anything unreadable put back to its default. */
export function sanitizeView(raw: unknown): ViewState {
  const o = (raw && typeof raw === "object" ? raw : {}) as Record<string, unknown>;
  const pick = <T extends string>(v: unknown, allowed: readonly T[], def: T): T => (allowed.includes(v as T) ? (v as T) : def);
  return {
    groupBy: pick(o.groupBy, ["bin", "type", "none"], DEFAULT_VIEW.groupBy),
    sortBy: pick(o.sortBy, ["custom", "name", "date"], DEFAULT_VIEW.sortBy),
    filter: pick(o.filter, ["all", "audio", "image"], DEFAULT_VIEW.filter),
    view: pick(o.view, ["list", "grid"], DEFAULT_VIEW.view),
    collapsed: Array.isArray(o.collapsed) ? o.collapsed.filter((x): x is string => typeof x === "string").slice(0, 200) : [],
  };
}

export interface Section {
  /** Folding key: a bin id, "type:audio", "unsorted", or "all". */
  id: string;
  name: string;
  color: string | null;
  /** The bin that dropping items on this section files them into, if it is one. */
  bin: string | null;
  items: LibraryItem[];
}

const byName = (a: LibraryItem, b: LibraryItem) => a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });
const byDate = (a: LibraryItem, b: LibraryItem) => (a.added < b.added ? 1 : a.added > b.added ? -1 : byName(a, b));

function sorted(items: LibraryItem[], by: SortBy): LibraryItem[] {
  if (by === "name") return [...items].sort(byName);
  if (by === "date") return [...items].sort(byDate);
  return items;
}

/** The items to show, in sections. Empty sections are dropped, except bins when grouping by bin. */
export function arrange(lib: Pick<Library, "collections">, items: LibraryItem[], v: ViewState): Section[] {
  const kept = v.filter === "all" ? items : items.filter((i) => i.kind === v.filter);
  if (v.groupBy === "none") return [{ id: "all", name: "All", color: null, bin: null, items: sorted(kept, v.sortBy) }];
  if (v.groupBy === "type") {
    const audio = kept.filter((i) => i.kind === "audio");
    const images = kept.filter((i) => i.kind === "image");
    return [
      { id: "type:audio", name: "Audio", color: null, bin: null, items: sorted(audio, v.sortBy) },
      { id: "type:image", name: "Pictures", color: null, bin: null, items: sorted(images, v.sortBy) },
    ].filter((s) => s.items.length > 0);
  }
  const byId = new Map(kept.map((i) => [i.id, i]));
  const binned = new Set<string>();
  const sections: Section[] = lib.collections.map((c: LibraryCollection) => {
    const inBin = c.items.map((id) => byId.get(id)).filter((i): i is LibraryItem => !!i);
    for (const i of inBin) binned.add(i.id);
    return { id: c.id, name: c.name, color: c.color ?? null, bin: c.id, items: sorted(inBin, v.sortBy) };
  });
  const loose = kept.filter((i) => !binned.has(i.id));
  if (loose.length > 0 || sections.length === 0) sections.push({ id: "unsorted", name: sections.length ? "Unsorted" : "All", color: null, bin: null, items: sorted(loose, v.sortBy) });
  return sections;
}

/** Item ids in the order they are drawn, each once, for range selection. */
export function flatIds(sections: readonly Section[], collapsed: readonly string[]): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const s of sections) {
    if (collapsed.includes(s.id)) continue;
    for (const i of s.items) {
      if (!seen.has(i.id)) {
        seen.add(i.id);
        out.push(i.id);
      }
    }
  }
  return out;
}

export function toggleCollapsed(v: ViewState, id: string): ViewState {
  return { ...v, collapsed: v.collapsed.includes(id) ? v.collapsed.filter((x) => x !== id) : [...v.collapsed, id] };
}
