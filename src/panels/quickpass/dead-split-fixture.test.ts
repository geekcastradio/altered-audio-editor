// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks Close Dead Space and Split Channels in TypeScript against the fixture
// the Rust test reads too (shared-fixtures/dead-split.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/dead-split.json";
import { LockedError, findClips } from "../editor/clip-edit";
import { splitChannelsOp, splittable } from "../properties/split-channels";
import { closeDeadSpace, deadSpaces } from "./dead-space";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));
const doc = fx.doc as never;

describe("dead space and split channels fixture", () => {
  it("dead space", () => {
    for (const c of fx.dead) {
      const o = { pause: c.pause as number, minRemove: c.minRemove as number };
      expect(J(deadSpaces(doc, c.tracks, o))).toEqual(c.spaces);
      let out: unknown;
      try {
        out = J(closeDeadSpace(doc, c.tracks, o));
      } catch (e) {
        out = { locked: e instanceof LockedError };
      }
      expect(out).toEqual(c.closed);
    }
  });
  it("split channels", () => {
    for (const c of fx.split) {
      const found = findClips(doc, c.ids);
      expect(splittable(doc, found).map((f) => f.clip.id)).toEqual(c.splittable);
      let n = 0;
      expect(J(splitChannelsOp(doc, found, () => `u${++n}`))).toEqual(c.out);
    }
  });
});
