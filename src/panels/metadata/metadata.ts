// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The Metadata panel's logic: saved tag sets, field checks, the file list
 * filter. No DOM, so it can be unit tested.
 */

import type { FileTags } from "../../app/core";
import { TAG_FIELDS } from "../../app/tags";

export type TagKey = "title" | "artist" | "album" | "year" | "genre" | "track" | "comment";

/** A named group of tag values to put on many files, such as a show's artist, album and genre. */
export interface TagSet {
  name: string;
  tags: Partial<Record<TagKey, string>>;
}

export const MAX_SETS = 100;
const KEYS = TAG_FIELDS.map((f) => f.key as TagKey);

/** Saved sets read back from preferences, keeping only well-formed ones. */
export function sanitizeSets(raw: unknown): TagSet[] {
  if (!Array.isArray(raw)) return [];
  const out: TagSet[] = [];
  for (const r of raw) {
    if (out.length >= MAX_SETS) break;
    if (!r || typeof r !== "object") continue;
    const o = r as { name?: unknown; tags?: unknown };
    const name = typeof o.name === "string" ? o.name.trim().slice(0, 80) : "";
    if (!name || out.some((s) => s.name.toLowerCase() === name.toLowerCase())) continue;
    const tags: TagSet["tags"] = {};
    if (o.tags && typeof o.tags === "object") {
      for (const k of KEYS) {
        const v = (o.tags as Record<string, unknown>)[k];
        if (typeof v === "string" && v.trim()) tags[k] = v.trim();
      }
    }
    if (Object.keys(tags).length) out.push({ name, tags });
  }
  return out;
}

/** Adds a set, or replaces the one with the same name (ignoring case). Blank fields are left out. */
export function upsertSet(sets: TagSet[], name: string, values: Partial<Record<TagKey, string | null | undefined>>): TagSet[] {
  const clean = name.trim().slice(0, 80);
  const tags: TagSet["tags"] = {};
  for (const k of KEYS) {
    const v = values[k]?.trim();
    if (v) tags[k] = v;
  }
  if (!clean || !Object.keys(tags).length) return sets;
  const rest = sets.filter((s) => s.name.toLowerCase() !== clean.toLowerCase());
  return [...rest, { name: clean, tags }].sort((a, b) => a.name.localeCompare(b.name)).slice(0, MAX_SETS);
}

export function removeSet(sets: TagSet[], name: string): TagSet[] {
  return sets.filter((s) => s.name !== name);
}

/** The year check: four digits from 1000 to 2999, or blank. Returns a message when wrong. */
export function checkYear(v: string): string | null {
  const t = v.trim();
  if (!t) return null;
  return /^[12]\d{3}$/.test(t) ? null : "Year should be four digits, such as 2026.";
}

/** The track check: a number, or a number and a total such as 3/12, or blank. */
export function checkTrack(v: string): string | null {
  const t = v.trim();
  if (!t) return null;
  return /^\d{1,3}(\/\d{1,3})?$/.test(t) ? null : "Track should be a number, or a number and a total such as 3/12.";
}

/** Every problem with the draft, in field order. */
export function problems(draft: Partial<Record<TagKey, string>>): string[] {
  return [checkYear(draft.year ?? ""), checkTrack(draft.track ?? "")].filter((m): m is string => m !== null);
}

/** One file in the panel's list. */
export interface Entry {
  path: string;
  tags: FileTags;
}

export const baseName = (p: string) => p.split(/[\\/]/).pop() ?? p;

/** The entries whose name or tags contain every word typed (ignoring case). */
export function filterEntries(entries: Entry[], query: string): Entry[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (!words.length) return entries;
  return entries.filter((e) => {
    const hay = [baseName(e.path), e.tags.title, e.tags.artist, e.tags.album, e.tags.genre].filter(Boolean).join("\n").toLowerCase();
    return words.every((w) => hay.includes(w));
  });
}

/** The tags a set would write: just its own fields. */
export function setToTags(s: TagSet): FileTags {
  return { ...s.tags };
}

/** Shows a set's values over a file's tags, keeping fields the set does not have. */
export function overlay(tags: FileTags, s: TagSet): FileTags {
  return { ...tags, ...s.tags };
}

/** Adds paths to the list without repeating any (compared ignoring case and slash direction). */
export function addPaths(have: string[], more: string[]): string[] {
  const key = (p: string) => p.replace(/\\/g, "/").toLowerCase();
  const seen = new Set(have.map(key));
  const out = [...have];
  for (const p of more) {
    if (!seen.has(key(p))) {
      seen.add(key(p));
      out.push(p);
    }
  }
  return out;
}
