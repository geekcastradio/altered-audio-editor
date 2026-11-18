// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Follow Playhead lets go when you scroll the view by hand while it plays, so you can look
 * at another part of the file. Zooming does not count: a zoom keeps the playhead in view and
 * the following goes on (it pages if the zoom pushed the playhead off the edge).
 */
export type FollowEffect = "release" | "hold" | "none";

/**
 * What a view change made by the person does to the following, given the view as it was.
 * A move of the playhead brings the following back. A zoom changes nothing. A scroll holds it.
 */
export function followEffect(
  now: { spp: number; scroll: number },
  change: { spp?: number; scroll?: number; playhead?: number },
): FollowEffect {
  if (change.playhead !== undefined) return "release";
  if (change.spp !== undefined && change.spp !== now.spp) return "none";
  if (change.scroll !== undefined && change.scroll !== now.scroll) return "hold";
  return "none";
}
