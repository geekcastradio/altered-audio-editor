// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import appSource from "./App.svelte?raw";

// session.current is a plain getter, not reactive. A dialog block written as
// `{#if session.current && thing()}` never tracks `thing` when the session was
// not loaded at first render, so the dialog never opens. The state that opens
// the dialog has to be read first.
describe("dialogs opened from a clip", () => {
  it("Time Stretch and Fit Music read their own state before session.current", () => {
    expect(appSource).toContain("{#if stretchFor && session.current && stretchClip()}");
    expect(appSource).toContain("{#if fitFor && session.current && fitClip()}");
    expect(appSource).not.toContain("{#if session.current && stretchClip()}");
    expect(appSource).not.toContain("{#if session.current && fitClip()}");
  });
});
