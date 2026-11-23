// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the panel rules in TypeScript against the fixture the Rust test
// reads too (shared-fixtures/panel-rules.json).
import { describe, expect, it } from "vitest";
import fx from "../../../shared-fixtures/panel-rules.json";
import { groupMin, MIN_SIZE, workspaceId } from "../../app/docking/registry";
import { allMidiClipIds, deleteClipsOps } from "../editor/midi";
import * as E from "./eq-graph";

const J = (v: unknown) => JSON.parse(JSON.stringify(v ?? null));

describe("panel rules fixture", () => {
  it("EQ nodes", () => {
    const nodes = E.eqNodes(fx.info as never);
    expect(J(nodes)).toEqual(fx.nodes);
    expect(J(E.eqNodes({ id: "x", name: "x", params: [] } as never))).toEqual(fx.empty);
    for (const c of fx.on) expect(E.nodeOn(fx.slots[c.si] as never, nodes[c.ni]!)).toBe(c.out);
  });
  it("clips and workspaces", () => {
    expect(J(allMidiClipIds(fx.doc as never))).toEqual(fx.midiIds);
    fx.sets.forEach((ids, i) => expect(J(deleteClipsOps(fx.doc as never, new Set(ids)))).toEqual(fx.del[i]));
    for (const c of fx.ws) expect(workspaceId(fx.names[c.ni]!, new Set(fx.taken[c.ti]))).toBe(c.out);
  });
  it("panel sizes", () => {
    expect(J(MIN_SIZE)).toEqual(fx.mins);
    for (const c of fx.groups) expect(groupMin(c.ids as never)).toEqual(c.out);
  });
});
