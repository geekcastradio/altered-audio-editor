// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import { importAuditionRacks, readRacks } from "./audition-presets";

const slot = (n: number, name: string, id: string, power: boolean, floats: number[] | null) => `
<prop.pair><key>SlotNumber_${n}</key><prop.list>
<prop.pair><key>DisplayName</key><ustring>${name}</ustring></prop.pair>
<prop.pair><key>EffectID</key><ustring>${id}</ustring></prop.pair>
<prop.pair><key>EffectPower</key><${power}/></prop.pair>
<prop.pair><key>EffectSettings</key><prop.list>${
  floats ? `<prop.pair><key>ParamList</key><array><array.type><float/></array.type>${floats.map((f) => `<float>${f}</float>`).join("")}</array></prop.pair>` : `<prop.pair><key>ChunkData</key><string>AAAA</string></prop.pair>`
}</prop.list></prop.pair>
</prop.list></prop.pair>`;

const rack = (name: string, slots: string) => `<prop.pair><key>${name}</key><prop.list><prop.pair><key>CustomData</key><prop.list>${slots}</prop.list></prop.pair></prop.list></prop.pair>`;

const FILE = `<?xml version='1.0'?><prop.map version='4'><prop.list>
<prop.pair><key>EffectsRackPresetID</key><prop.list><prop.pair><key>Presets</key><prop.list>
${rack("(Default)", "")}
${rack("(Last Applied Rack)", slot(1, "Limiter A", "57c2006b-d675-40bc-a1aa-f7764ef79959", true, [0.94]))}
${rack(
  "My Voice",
  slot(2, "Limiter A", "57c2006b-d675-40bc-a1aa-f7764ef79959", true, [0.97, 0.7, 0.13, 0.375, 1, 1, 0]) +
    slot(1, "Noise A", "c6762838-e451-4d22-9c90-b0eaa734605a", true, null) +
    slot(3, "Comp A", "f76dad9c-77da-477a-9939-eefefc5c23bd", false, [0.5]) +
    slot(4, "Room A", "7ea99d31-ca8f-454f-acd7-81a258efb336", true, null),
)}
${rack("Only Reverb", slot(1, "Room A", "7ea99d31-ca8f-454f-acd7-81a258efb336", true, null))}
</prop.list></prop.pair></prop.list></prop.pair>
</prop.list></prop.map>`;

describe("Audition rack import", () => {
  it("lists racks by name and skips Audition's own two", () => {
    expect(readRacks(FILE).map((r) => r.name)).toEqual(["My Voice", "Only Reverb"]);
  });

  it("keeps the slot order Audition numbered them in", () => {
    expect(readRacks(FILE)[0]!.slots.map((s) => s.name)).toEqual(["Noise A", "Limiter A", "Comp A", "Room A"]);
  });

  it("reads the limiter ceiling from Audition's scale", () => {
    const p = importAuditionRacks(FILE).presets[0]!;
    expect(p.name).toBe("My Voice (Audition)");
    expect(p.chain.slots.map((s) => s.effect)).toEqual(["altaudio.noise_reducer", "altaudio.limiter"]);
    expect(p.chain.slots[1]!.params.ceiling).toBeCloseTo(-3);
  });

  it("leaves out effects that were off or have no match, and says which", () => {
    const r = importAuditionRacks(FILE);
    expect(r.chains).toBe(1);
    expect(r.missing).toEqual([{ name: "Room A", count: 2 }]);
  });

  it("reads nothing from a file that is not a preset file", () => {
    expect(importAuditionRacks("<html></html>").chains).toBe(0);
    expect(importAuditionRacks("").presets).toEqual([]);
  });
});
