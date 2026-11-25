// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { clipEnd, mediaOf } from "../panels/editor/clip-edit";
import type { SessionDoc } from "../stores/session";

/**
 * The audio tracks, top to bottom, with nothing playing between `start` and `start + length`
 * (session samples). Imported audio goes on the first of these, so a new session fills Track 1
 * before it makes a Track 7.
 */
export function tracksWithRoom(doc: SessionDoc, start: number, length: number): string[] {
  const media = mediaOf(doc);
  const end = start + Math.max(1, length);
  return doc.tracks
    .filter((t) => t.kind !== "midi" && !t.clips.some((c) => c.start < end && clipEnd(c, media, doc.sample_rate) > start))
    .map((t) => t.id);
}

/**
 * Side by side: where each of several files starting at `start` goes. The first takes the track
 * under the pointer (`first`), or else the first track with room. Every later file takes the next
 * audio track below the one before it that has room for it, so it never lands on top of audio
 * already there. null means a new track is needed. `lengths` are in session samples.
 */
export function sideBySideTargets(doc: SessionDoc, first: string | null, start: number, lengths: number[]): (string | null)[] {
  const audio = doc.tracks.filter((t) => t.kind !== "midi").map((t) => t.id);
  const out: (string | null)[] = [];
  let after = -1;
  lengths.forEach((len, k) => {
    const room = new Set(tracksWithRoom(doc, start, len));
    const used = new Set(out.filter((x): x is string => x !== null));
    let pick: string | null = null;
    if (k === 0 && first && audio.includes(first)) pick = first;
    else {
      for (let i = after + 1; i < audio.length; i++) {
        const id = audio[i]!;
        if (room.has(id) && !used.has(id)) {
          pick = id;
          break;
        }
      }
    }
    out.push(pick);
    if (pick !== null) after = audio.indexOf(pick);
  });
  return out;
}
