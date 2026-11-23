// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the video helpers and the open-file rules in TypeScript against the
// fixture the Rust test reads too (shared-fixtures/video-open.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/video-open.json";
import { colorVars, DEFAULT_SETTINGS, meterPercent, settings, wheelAnchorX, zoomRatios } from "../stores/settings";
import { dropScale, isSessionFile } from "./filedrop";
import * as F from "./session-files";
import * as V from "./video";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const v = fx.video;
const o = fx.open;

describe("video fixture", () => {
  it("describes and decodes", () => {
    for (const c of v.describe) expect(V.describeVideo(c.meta as never, c.st as never)).toBe(c.out);
    for (const c of v.pics) {
      const r = V.decodePicture(c.bytes);
      expect(r ? { width: r.width, height: r.height, time: r.time, rgba: Array.from(r.rgba) } : null).toEqual(c.out);
    }
  });
  it("maps time and sizes", () => {
    for (const c of v.time) expect(J(V.videoTime(c.s ?? Infinity, c.o, c.d))).toEqual(c.out);
    for (const c of v.tc) expect(V.timecode(c.s, c.f)).toBe(c.out);
    for (const c of v.frame) expect(V.needsFrame(c.a as never, c.b as never, c.fps)).toBe(c.out);
    for (const c of v.width) expect(V.askWidth(c.p, c.r, c.w)).toBe(c.out);
    for (const c of v.err) expect(V.errorText(c.e, c.fb)).toBe(c.out);
  });
});

describe("open rules fixture", () => {
  it("plans and names", () => {
    for (const c of o.plans) expect(J(F.planOpen(c.paths))).toEqual(c.out);
    for (const c of o.kinds) expect([F.isAudioFile(c.p), F.isVideoFile(c.p), isSessionFile(c.p)]).toEqual([c.audio, c.video, c.session]);
    for (const c of o.names) expect(F.displayName(c.p, c.d ? ({ name: c.d } as never) : null)).toBe(c.out);
  });
  it("summaries", () => {
    for (const c of o.sesx) expect(F.sesxSummary(c.r as never)).toBe(c.out);
    for (const c of o.rpp) expect(F.rppSummary(c.r as never)).toBe(c.out);
    for (const c of o.otio) expect(F.otioSummary(c.r as never)).toBe(c.out);
  });
  it("screen rules", () => {
    for (const c of o.drop) expect(dropScale(c.p, c.c, c.d)).toBeCloseTo(c.out, 9);
    for (const c of o.wheel) {
      settings.set({ ...DEFAULT_SETTINGS, wheelAnchor: c.a as never });
      expect(wheelAnchorX(c.m, c.w, c.p, c.ex)).toBeCloseTo(c.out, 9);
    }
    for (const c of o.zoom) expect(zoomRatios(c.step)).toEqual(c.out);
    for (const c of o.colors) expect(colorVars(c as never)).toEqual(c.out);
    for (const c of o.meter) expect(meterPercent(c.db, c.floor)).toBeCloseTo(c.out, 9);
  });
});
