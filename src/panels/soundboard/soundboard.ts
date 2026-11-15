// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Soundboard pads: nine slots, each holding the path of an audio file
 * (a jingle, a stinger, an ad read). Pressing a pad puts its file on the
 * session at the playhead. The pads are saved with the program's
 * preferences, not with a session.
 */

export const PAD_COUNT = 9;

export interface Pad {
  path: string | null;
  label: string;
}

export const emptyPad = (): Pad => ({ path: null, label: "" });

export const emptyPads = (): Pad[] => Array.from({ length: PAD_COUNT }, emptyPad);

/** The file name without its folder or extension. */
export function padLabel(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? path;
  const dot = name.lastIndexOf(".");
  return (dot > 0 ? name.slice(0, dot) : name).slice(0, 40);
}

/** Saved pads checked: always nine, each with a path or empty. */
export function sanitizePads(raw: unknown): Pad[] {
  const list = Array.isArray(raw) ? raw : [];
  return Array.from({ length: PAD_COUNT }, (_, i) => {
    const p = list[i] as { path?: unknown; label?: unknown } | null | undefined;
    if (!p || typeof p.path !== "string" || p.path.length === 0 || p.path.length > 1024) return emptyPad();
    const label = typeof p.label === "string" && p.label.trim() ? p.label.trim().slice(0, 40) : padLabel(p.path);
    return { path: p.path, label };
  });
}

/** The pads with slot `i` holding `path`. */
export function withPad(pads: readonly Pad[], i: number, path: string | null): Pad[] {
  return pads.map((p, k) => (k === i ? (path ? { path, label: padLabel(path) } : emptyPad()) : p));
}

type Listener = (pads: Pad[]) => void;

export class PadsStore {
  private value: Pad[] = emptyPads();
  private readonly listeners = new Set<Listener>();

  get current(): Pad[] {
    return this.value;
  }

  set(next: Pad[]): void {
    this.value = sanitizePads(next);
    for (const fn of this.listeners) fn(this.value);
  }

  subscribe(fn: Listener): () => void {
    this.listeners.add(fn);
    fn(this.value);
    return () => {
      this.listeners.delete(fn);
    };
  }
}
