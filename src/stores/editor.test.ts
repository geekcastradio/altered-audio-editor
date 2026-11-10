// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { editor, parseSettings, restoreSettings, settingsOf } from "./editor";

describe("editor settings", () => {
  it("keeps defaults for missing or bad values", () => {
    const s = parseSettings({ snap: { on: false, ruler: "yes" }, autoCrossfade: 1, timeFormat: "samples" });
    expect(s.snap).toEqual({ on: false, clips: true, playhead: true, markers: true, ruler: true });
    expect(s.autoCrossfade).toBe(true);
    expect(s.timeFormat).toBe("samples");
    expect(parseSettings(null).snap.on).toBe(true);
  });

  it("restores saved settings into the editor", () => {
    restoreSettings({ snap: { clips: false }, autoCrossfade: false });
    const s = settingsOf(editor.current());
    expect(s.snap.clips).toBe(false);
    expect(s.autoCrossfade).toBe(false);
  });

  it("keeps the HUD on unless it was saved off", () => {
    expect(parseSettings(null).showHud).toBe(true);
    expect(parseSettings({ showHud: false }).showHud).toBe(false);
    expect(parseSettings({ showHud: "no" }).showHud).toBe(true);
    editor.setShowHud(false);
    expect(settingsOf(editor.current()).showHud).toBe(false);
    editor.setShowHud(true);
  });
});

describe("gap pick", () => {
  it("picks a gap and its track, and clears when clips or another track are picked", () => {
    editor.selectClips(["a"]);
    editor.selectGap({ trackId: "T1", at: 5_000 });
    expect(editor.current().selectedGap).toEqual({ trackId: "T1", at: 5_000 });
    expect(editor.current().selectedTrack).toBe("T1");
    expect(editor.current().selectedClips.size).toBe(0);
    editor.selectClips(["b"]);
    expect(editor.current().selectedGap).toBeNull();
    editor.selectGap({ trackId: "T1", at: 5_000 });
    editor.selectTrack("T2");
    expect(editor.current().selectedGap).toBeNull();
    editor.selectGap({ trackId: "T1", at: 1 });
    editor.selectGap(null);
    expect(editor.current().selectedGap).toBeNull();
    expect(editor.current().selectedTrack).toBe("T1");
  });
});

describe("marker pick", () => {
  it("replaces the pick, clears with an empty list, and leaves the clip pick alone", () => {
    editor.selectClips(["a"]);
    editor.selectMarkers(["m1", "m2"]);
    expect([...editor.current().selectedMarkers]).toEqual(["m1", "m2"]);
    expect(editor.current().selectedClips.has("a")).toBe(true);
    editor.selectMarkers(["m3"]);
    expect([...editor.current().selectedMarkers]).toEqual(["m3"]);
    editor.selectMarkers([]);
    expect(editor.current().selectedMarkers.size).toBe(0);
  });
});

describe("play mode", () => {
  it("toggles between loop, skip and normal, and survives a restart", () => {
    editor.togglePlayMode("loop");
    expect(editor.current().playMode).toBe("loop");
    editor.togglePlayMode("skip");
    expect(editor.current().playMode).toBe("skip");
    editor.togglePlayMode("skip");
    expect(editor.current().playMode).toBe("normal");
    expect(parseSettings({ playMode: "loop" }).playMode).toBe("loop");
    expect(parseSettings({ playMode: "sideways" }).playMode).toBe("normal");
  });

  it("snap to zero crossings is off by default, saved, and ignores junk", () => {
    expect(parseSettings(null).snapZero).toBe(false);
    expect(parseSettings({ snapZero: true }).snapZero).toBe(true);
    expect(parseSettings({ snapZero: "yes" }).snapZero).toBe(false);
  });
});
