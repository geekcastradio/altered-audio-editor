// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Multitrack > Reduce Bleed: which clips of a mic's track can be cleaned,
 * how cleaned copies replace them, and the notice afterwards. The cleaning
 * is done in the core; this decides what to ask for and builds one edit.
 */

import type { ClipDoc, Op } from "../stores/session";

/** Clips below this much change are left as they are. */
export const MIN_REMOVED_DB = 0.2;

export interface BleedWork {
  /** Clips that can be cleaned. */
  work: ClipDoc[];
  locked: number;
  /** Looped or stretched clips, which the core cannot copy as they play. */
  unsuitable: number;
}

/** Splits a track's clips into the ones that can be cleaned and the ones left alone. */
export function bleedWork(clips: readonly ClipDoc[]): BleedWork {
  const open = clips.filter((c) => !c.locked);
  const work = open.filter((c) => !c.looped && !c.stretch);
  return { work, locked: clips.length - open.length, unsuitable: open.length - work.length };
}

export interface CleanedFile {
  /** The clip it replaces. */
  clip: ClipDoc;
  /** The cleaned copy, as the media pool read it back. */
  media: { id: string; path: string; sampleRate: number; channels: number; length: number };
}

/** One edit: add any media the session does not know yet, then swap each clip for one on its copy. */
export function replaceOps(trackId: string, files: readonly CleanedFile[], known: ReadonlySet<string>): Op[] {
  const have = new Set(known);
  const ops: Op[] = [];
  const restored: [string, ClipDoc][] = [];
  for (const f of files) {
    if (!have.has(f.media.id)) {
      have.add(f.media.id);
      ops.push({
        op: "add_media",
        media: { id: f.media.id, path: f.media.path, relative_path: null, sample_rate: f.media.sampleRate, channels: f.media.channels, length: f.media.length },
      });
    }
    restored.push([trackId, { ...f.clip, media_id: f.media.id, source_in: 0, source_out: f.media.length }]);
  }
  ops.push({ op: "remove_clips", ids: files.map((f) => f.clip.id) }, { op: "restore_clips", clips: restored });
  return ops;
}

/** The notice after a run. `removed` holds each cleaned clip's change in dB. */
export function bleedSummary(removed: readonly number[], w: Pick<BleedWork, "locked" | "unsuitable">, changed: number): string {
  const notes: string[] = [];
  if (w.locked > 0) notes.push(`${w.locked} locked ${w.locked === 1 ? "clip was" : "clips were"} left alone`);
  if (w.unsuitable > 0) notes.push(`${w.unsuitable} looped or stretched ${w.unsuitable === 1 ? "clip was" : "clips were"} left alone`);
  const tail = notes.length ? ` ${notes.join(", ")}.` : "";
  if (changed === 0) {
    return `That mic picked up almost none of the other one, so nothing was changed.${tail}`;
  }
  const most = Math.max(...removed);
  return `Cleaned ${changed} ${changed === 1 ? "clip" : "clips"}, up to ${most.toFixed(1)} dB quieter overall. One undo puts the originals back. Listen before you keep it.${tail}`;
}
