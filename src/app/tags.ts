// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** The tag fields the File Tags dialog edits, in the order it shows them. */
import type { FileTags } from "./core";

export const TAG_FIELDS: { key: keyof FileTags; label: string }[] = [
  { key: "title", label: "Title" },
  { key: "artist", label: "Artist" },
  { key: "album", label: "Album" },
  { key: "year", label: "Year" },
  { key: "genre", label: "Genre" },
  { key: "track", label: "Track" },
  { key: "comment", label: "Comment" },
];

/** Formats that take tags when saved. */
export const TAGGABLE = /\.(wav|wave|flac|mp3|m4a|mp4|m4b)$/i;

/** The tags worth writing: trimmed, with blank fields left out so they don't touch what the file has. */
export function cleanTags(draft: Record<string, string>): FileTags {
  const out: FileTags = {};
  for (const f of TAG_FIELDS) {
    const v = (draft[f.key] ?? "").trim();
    if (v) out[f.key] = v;
  }
  return out;
}
