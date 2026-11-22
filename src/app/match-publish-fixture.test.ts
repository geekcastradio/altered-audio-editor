// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript Match EQ and Publish Package builder against the same fixtures the
// Rust tests read (shared-fixtures/match-eq.json and publish.json).
import { describe, expect, it } from "vitest";
import eqFx from "../../shared-fixtures/match-eq.json";
import pubFx from "../../shared-fixtures/publish.json";
import mk from "../../shared-fixtures/markers.json";
import * as M from "./match-eq";
import * as P from "./publish";
import type { MarkerDoc } from "../stores/session";

const all = mk.markers[0] as unknown as MarkerDoc[];
const sets: Record<string, MarkerDoc[]> = {
  all,
  none: [],
  chaptersOnly: all.filter((m) => m.kind === "chapter"),
  adsOnly: all.filter((m) => m.kind === "ad_slot"),
  clipsOnly: all.filter((m) => m.kind === "social_clip"),
  noChap: all.filter((m) => m.kind !== "chapter"),
};
const any = (c: unknown) => c as Record<string, any>;

describe("match eq fixture", () => {
  it("filter responses", () => {
    for (const c of eqFx.response)
      expect(M.responseDb(c.shape as "peak", c.rate, c.f0, c.q, c.g, c.f)).toBe(
        c.out,
      );
  });
  it("matches", () => {
    for (const c of eqFx.fits)
      expect(M.matchEq(c.src, c.ref, c.opts)).toEqual(c.fit);
  });
  it("voice target", () => {
    for (const raw of eqFx.target) {
      const c = any(raw);
      if ("tgt" in c) expect(M.voiceTarget(c.points)).toEqual(c.tgt);
      else expect(M.voiceTargetEq(c.src)).toEqual(c.fit);
    }
  });
});

describe("publish fixture", () => {
  it("episode checks", () => {
    for (const c of pubFx.problems)
      expect(P.episodeProblems(c.info as P.EpisodeInfo)).toEqual(c.out);
  });
  it("package files", () => {
    for (const c of pubFx.pub)
      expect(
        P.packageFiles(
          c.info as P.EpisodeInfo,
          c.audio,
          c.dur,
          sets[c.set]!,
          48000,
          c.checked,
        ),
      ).toEqual(c.files);
  });
  it("vtt", () => {
    for (const c of pubFx.vtt) expect(P.chaptersVtt(c.chaps, c.d)).toBe(c.out);
    for (const c of pubFx.vttTime) expect(P.vttTime(c.t)).toBe(c.out);
  });
});
