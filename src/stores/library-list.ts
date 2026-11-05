// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The list of libraries the app knows about, and the small file that carries
 * it between computers. The file holds names, colors and folder paths only.
 * The libraries themselves (library.json and their audio) are folders you
 * move yourself; importing the list on another computer adds each entry, and
 * any whose folder is not there yet shows as missing with a Locate button.
 */

export interface KnownLibrary {
  /** The library folder. */
  dir: string;
  name: string;
  /** Palette token or #rrggbb, or null for the automatic color. */
  color: string | null;
}

export const LIST_FORMAT = "altaudio.libraries";
export const LIST_VERSION = 1;

/** The most libraries a list holds, so a damaged file cannot flood the app. */
export const MAX_KNOWN = 500;

/** Last folder name in a path, used as a name when none is known. */
export function lastSegment(dir: string): string {
  const parts = dir.split(/[\\/]+/).filter((p) => p.length > 0);
  return parts[parts.length - 1] ?? dir;
}

/** The same folder written two ways (slash style, letter case, trailing slash). */
export function sameDir(a: string, b: string): boolean {
  const clean = (p: string) => p.replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase();
  return clean(a) === clean(b);
}

/** Reads a stored or imported list: keeps good entries, drops duplicates and junk. */
export function sanitizeKnown(raw: unknown): KnownLibrary[] {
  if (!Array.isArray(raw)) return [];
  const out: KnownLibrary[] = [];
  for (const r of raw) {
    if (!r || typeof r !== "object") continue;
    const e = r as { dir?: unknown; name?: unknown; color?: unknown };
    if (typeof e.dir !== "string" || e.dir.trim() === "") continue;
    if (out.some((k) => sameDir(k.dir, e.dir as string))) continue;
    const name = typeof e.name === "string" && e.name.trim() !== "" ? e.name.trim().slice(0, 120) : lastSegment(e.dir);
    const color = typeof e.color === "string" && e.color !== "" ? e.color : null;
    out.push({ dir: e.dir, name, color });
    if (out.length >= MAX_KNOWN) break;
  }
  return out;
}

/** The list as text: pretty printed so it reads and diffs well. */
export function packLibraryList(known: readonly KnownLibrary[]): string {
  return JSON.stringify({ format: LIST_FORMAT, version: LIST_VERSION, libraries: known }, null, 2) + "\n";
}

/** A list read from text, or the reason it cannot be used. */
export function readLibraryList(text: string): KnownLibrary[] | { error: string } {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    return { error: "That file is not a library list." };
  }
  if (!raw || typeof raw !== "object" || (raw as { format?: unknown }).format !== LIST_FORMAT) {
    return { error: "That file is not an Altered Audio library list." };
  }
  const v = (raw as { version?: unknown }).version;
  if (typeof v !== "number" || v > LIST_VERSION) {
    return { error: "That list was made by a newer version of Altered Audio. Update the app to read it." };
  }
  return sanitizeKnown((raw as { libraries?: unknown }).libraries);
}
