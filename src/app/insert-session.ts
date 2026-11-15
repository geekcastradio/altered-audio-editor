// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** Insert Session as Clip: names and the notice after it. */

/** The clip's name: the session's file name without its ending, plus "Mix". */
export function mixName(path: string): string {
  const file = path.split(/[\\/]/).pop() ?? path;
  const stem = file.replace(/\.[^.]+$/, "");
  return `${stem || "Session"} Mix`;
}

/** What to tell the person once the session is in as one clip. */
export function insertNotice(name: string, r: { missing: number; peak: number }): string {
  const parts = [`Inserted "${name}" as one clip. It is a copy of the session as it was saved; insert it again to pick up later changes.`];
  if (r.missing > 0) parts.push(`${r.missing} audio file${r.missing === 1 ? "" : "s"} it uses could not be found, so that sound is missing.`);
  if (r.peak > 1) parts.push("The mix goes above 0 dBFS; it is saved as float, so nothing is lost, but turn the clip down before export.");
  return parts.join(" ");
}
