// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import type { SessionRow } from "../files/session-row";

/** Tabs show once a second session is open; one session needs no tab bar. */
export function showTabs(rows: readonly SessionRow[]): boolean {
  return rows.length >= 2;
}

/** The tab to switch to when the tab at `id` is closed: its right neighbor, else its left, else none. */
export function neighborOf(rows: readonly SessionRow[], id: string): string | null {
  const i = rows.findIndex((r) => r.id === id);
  if (i < 0 || rows.length < 2) return null;
  return (rows[i + 1] ?? rows[i - 1])?.id ?? null;
}

/** What a tab says: the session's name, with a star while it has unsaved changes. */
export function tabLabel(row: SessionRow): string {
  return row.dirty ? `${row.name} *` : row.name;
}
