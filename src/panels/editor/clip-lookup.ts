// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import type { ClipDoc, SessionDoc } from "../../stores/session";

/** The clip with this id and the track it is on, or null when the id is empty or no longer in the session. */
export function clipById(doc: SessionDoc | null | undefined, id: string | null): { clip: ClipDoc; trackId: string } | null {
  if (!doc || !id) return null;
  for (const t of doc.tracks) for (const c of t.clips) if (c.id === id) return { clip: c, trackId: t.id };
  return null;
}
