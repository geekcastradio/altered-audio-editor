// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { editor } from "../../stores/editor";
import type { SessionStore } from "../../stores/session";
import { revealScroll } from "./reveal";

/**
 * Scrolls the Editor so `to` (a sample position) is on screen, when it is not already.
 * Used when a list (Markers, QuickPass) jumps the playhead somewhere else.
 */
export function showTime(session: SessionStore, to: number, at = 0.25): void {
  const v = session.current?.view;
  if (!v) return;
  const scroll = revealScroll(v.scroll, Math.round(editor.current().width * v.samples_per_pixel), to, at);
  if (scroll !== null) session.setView({ ...v, scroll });
}
