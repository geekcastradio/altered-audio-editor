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
