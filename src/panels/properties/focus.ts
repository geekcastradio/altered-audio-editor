// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * A way for a menu command to open a section of the Properties panel.
 * Clip > Time Stretch and Clip > Fit Music to Length show the panel and
 * open their section there instead of raising a dialog. The request
 * carries the time it was made, so a panel that mounts a moment later
 * (it was closed) still answers it, and an old request never opens a
 * section the next time the panel is shown.
 */

import { writable, type Readable } from "svelte/store";

export type PropertiesSection = "stretch" | "remix" | "channels";

export interface SectionRequest {
  section: PropertiesSection;
  at: number;
}

/** How long a request stays valid, in milliseconds. */
export const FRESH_MS = 2000;

const store = writable<SectionRequest | null>(null);

export const sectionRequests: Readable<SectionRequest | null> = { subscribe: store.subscribe };

export function requestSection(section: PropertiesSection, now: number = Date.now()): void {
  store.set({ section, at: now });
}

export function isFresh(r: SectionRequest | null, now: number = Date.now()): r is SectionRequest {
  return r !== null && now - r.at >= 0 && now - r.at < FRESH_MS;
}
