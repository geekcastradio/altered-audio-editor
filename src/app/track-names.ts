// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * A track name from a file name: the extension goes, and so does the
 * number call recorders put in front of each speaker ("1-name.flac"
 * becomes "name"). Underscores read as spaces.
 */
export function trackNameFromFile(fileName: string): string {
  const base = fileName.replace(/^.*[\\/]/, "");
  const s = base
    .replace(/\.[^.]+$/, "")
    .replace(/^\d{1,3}[-_ ]+(?=\S)/, "")
    .replace(/_+/g, " ")
    .trim();
  return s || base;
}
