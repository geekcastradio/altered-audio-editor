// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it, vi } from "vitest";
import { editor } from "../../stores/editor";
import type { SessionStore } from "../../stores/session";
import { showTime } from "./show-time";

function fake(scroll: number) {
  const setView = vi.fn();
  const session = { current: { view: { scroll, samples_per_pixel: 100, playhead: 0, selection: null } }, setView } as unknown as SessionStore;
  return { session, setView };
}

describe("showTime", () => {
  it("scrolls when the position is out of view", () => {
    editor.setWidth(1000);
    const { session, setView } = fake(0);
    showTime(session, 500_000);
    expect(setView).toHaveBeenCalledTimes(1);
    expect(setView.mock.calls[0]![0].scroll).toBe(475_000);
  });
  it("leaves the view alone when the position is already on screen", () => {
    editor.setWidth(1000);
    const { session, setView } = fake(0);
    showTime(session, 50_000);
    expect(setView).not.toHaveBeenCalled();
  });
});
