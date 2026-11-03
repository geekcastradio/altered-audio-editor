// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/** Effects the user favorites, kept as a short list of effect ids at the top of every effect menu. */

import { prefs } from "../../stores/prefs";

export const KEY_FAVORITES = "effects.favorites";
export const MAX_FAVORITES = 40;

/** A saved list with anything unreadable dropped, each id once. */
export function sanitizeFavorites(raw: unknown): string[] {
  if (!Array.isArray(raw)) return [];
  const out: string[] = [];
  for (const v of raw) if (typeof v === "string" && v.length > 0 && v.length < 200 && !out.includes(v)) out.push(v);
  return out.slice(0, MAX_FAVORITES);
}

/** The list with `id` added when it was missing and removed when it was there. */
export function toggleFavorite(list: readonly string[], id: string): string[] {
  return list.includes(id) ? list.filter((x) => x !== id) : [...list, id].slice(-MAX_FAVORITES);
}

/**
 * What a fresh install starts with: the effects a podcast edit reaches for most. Used only
 * until the user stars or unstars something, after which their own list (even an empty one) wins.
 */
export const STARTER_FAVORITES: readonly string[] = [
  "altaudio.hum_cut",
  "altaudio.noise_reducer",
  "altaudio.dc_offset",
  "altaudio.deesser",
  "altaudio.dynamics",
  "altaudio.speech_leveler",
  "altaudio.parametric_eq",
  "altaudio.limiter",
  "altaudio.pitch_shift",
];

/** The saved favorites right now, or the starter set when none were ever saved. */
export const favoriteIds = (): string[] => {
  const raw = prefs.raw(KEY_FAVORITES);
  return raw === undefined || raw === null ? [...STARTER_FAVORITES] : sanitizeFavorites(raw);
};
