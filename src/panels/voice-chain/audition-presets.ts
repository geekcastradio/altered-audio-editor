// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Reads the effect chains (racks) saved in an Audition `EffectPresets.xml` and turns them into
 * Voice Chain presets. A rack lists its effects by name and keeps each effect's settings as
 * numbers in Audition's own scale, or as opaque binary. We can only read the settings we have
 * worked out, so the chain comes across with the same effects in the same order and our own
 * starting values, except where the scale is known (the limiter's ceiling). The Presets
 * menu says so after an import.
 */

import type { ChainPreset } from "./chain";
import { BLOCKS } from "./chain";

export interface AuditionSlot {
  name: string;
  /** The effect's identifier in the file, which is what we match on. */
  id: string;
  on: boolean;
  /** The settings as plain numbers, or null when Audition kept them as binary. */
  values: number[] | null;
}

export interface AuditionRack {
  name: string;
  slots: AuditionSlot[];
}

interface Node {
  tag: string;
  text: string;
  kids: Node[];
}

/** A small reader for the prop.map XML Audition writes: tags, text, no attributes needed. */
function parse(xml: string): Node {
  const root: Node = { tag: "#root", text: "", kids: [] };
  const stack: Node[] = [root];
  const re = /<(\/?)([A-Za-z0-9_.]+)([^>]*?)(\/?)>|([^<]+)/g;
  for (let m = re.exec(xml); m; m = re.exec(xml)) {
    const top = stack[stack.length - 1]!;
    if (m[5] !== undefined) {
      top.text += m[5];
    } else if (m[1]) {
      if (stack.length > 1) stack.pop();
    } else {
      const n: Node = { tag: m[2]!, text: "", kids: [] };
      top.kids.push(n);
      if (!m[4]) stack.push(n);
    }
  }
  return root;
}

const decode = (s: string) => s.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&apos;/g, "'").replace(/&amp;/g, "&").trim();

/** A prop.list as key to value node. */
function pairs(list: Node | undefined): Map<string, Node> {
  const out = new Map<string, Node>();
  for (const p of list?.kids ?? []) {
    if (p.tag !== "prop.pair") continue;
    const key = p.kids.find((k) => k.tag === "key");
    const val = p.kids.find((k) => k.tag !== "key");
    if (key && val) out.set(decode(key.text), val);
  }
  return out;
}

/** The racks in an EffectPresets.xml, without Audition's own "(Default)" and "(Last Applied Rack)". */
export function readRacks(xml: string): AuditionRack[] {
  const doc = parse(xml);
  const map = doc.kids.find((k) => k.tag === "prop.map");
  const top = pairs(map?.kids.find((k) => k.tag === "prop.list"));
  const racks = pairs(pairs(top.get("EffectsRackPresetID")).get("Presets"));
  const out: AuditionRack[] = [];
  for (const [name, preset] of racks) {
    if (name === "(Default)" || name === "(Last Applied Rack)") continue;
    const slots: { n: number; slot: AuditionSlot }[] = [];
    const walk = (node: Node) => {
      for (const [key, val] of pairs(node)) {
        const m = /^SlotNumber_(\d+)$/.exec(key);
        if (m && val.tag === "prop.list") {
          const f = pairs(val);
          const dn = f.get("DisplayName");
          if (!dn) continue;
          const eid = decode(f.get("EffectID")?.text ?? "").toLowerCase();
          const arr = pairs(f.get("EffectSettings")).get("ParamList");
          const values = arr ? arr.kids.filter((k) => k.tag === "float" || k.tag === "double").map((k) => Number(k.text)) : null;
          slots.push({
            n: Number(m[1]),
            slot: { name: decode(dn.text), id: eid, on: f.get("EffectPower")?.tag !== "false", values: values && values.every(Number.isFinite) ? values : null },
          });
        } else if (val.tag === "prop.list") {
          walk(val);
        }
      }
    };
    walk(preset);
    if (slots.length) out.push({ name, slots: slots.sort((a, b) => a.n - b.n).map((s) => s.slot) });
  }
  return out;
}

type Made = { effect: string; params: Record<string, number> };
const plain =
  (effect: string, params: Record<string, number> = {}) =>
  (): Made => ({ effect: `altaudio.${effect}`, params });

function limiterFrom(v: number[] | null): Made {
  const db = v && v.length > 0 ? Math.round((v[0]! - 1) * 1000) / 10 : -0.1;
  return { effect: "altaudio.limiter", params: { ceiling: Math.max(-24, Math.min(0, db)), lookahead: 5, release: 100 } };
}

/**
 * Audition's effects we have an equivalent for, by the identifier the file stores (never by
 * name, so no one else's feature names live in this source). Built-in effects carry a GUID;
 * the Phaser carries a VST-style id and is matched by its display name.
 */
const MAP: Record<string, (v: number[] | null) => Made> = {
  // The limiter keeps its ceiling as 1 + dB/100 (0.999 is -0.1 dB, 0.97 is -3 dB, 0.94 is -6 dB).
  "57c2006b-d675-40bc-a1aa-f7764ef79959": limiterFrom,
  // The same effect under its older identifier, as saved by earlier versions.
  "e0b23f05-f1a7-4ef7-9b50-7ec3e3002058": limiterFrom,
  "3034b5c2-23a7-43b6-a1ef-9c4047f4368d": plain("speech_leveler"),
  "c6762838-e451-4d22-9c90-b0eaa734605a": () => BLOCKS.cleanNoise(10),
  "f76dad9c-77da-477a-9939-eefefc5c23bd": plain("dynamics"),
  "3981e750-c8ae-40a2-889f-41a01c7efa03": plain("multiband"),
  "8edcd326-33de-4a4d-985b-f1bf48ed1427": plain("parametric_eq"),
  "4f327230-f04c-4c34-9ea5-a998b4459221": plain("curve_eq"),
  "ee38cdc5-5914-4a75-b7dd-73a265e55c31": plain("notch"),
  "ffbe710f-cd69-4139-ad26-65603616d9d4": plain("deesser"),
  "f14fee7d-fe5f-4202-b790-f53209e56411": plain("gain"),
  "f205d7b7-5e0b-421b-bc6d-5f27096ba192": plain("echo"),
  "5e85f068-b39d-4c63-9912-2030f6f0041e": plain("echo"),
  "c4ad21d8-9748-4d79-b4a3-d88ab7e4be34": plain("echo"),
  "31d33f85-3288-489e-8390-d03c60d9ce6c": plain("chorus"),
  "d30b6731-3e00-4a11-9650-e816f58a9d12": plain("flanger"),
  "9b66ab13-4d1b-469a-88c6-b807fcf1df16": plain("distortion"),
  "164b4e8a-7105-406b-b23b-1ef2cc4d8957": plain("pitch_shift"),
  "433322b0-d25a-4cfa-a4f1-4a1ab2d7615b": plain("click_repair"),
  "0a1016a3-74e6-4c93-8b6c-e058a1419bf1": plain("center_remover"),
  "7354704a-d87c-11e0-b05b-763c4824019b": plain("phase_curve"),
};
const BY_NAME: Record<string, (v: number[] | null) => Made> = { Phaser: plain("phaser") };

export interface AuditionImport {
  presets: ChainPreset[];
  /** Audition effect names with no match here, with how many chains used them. */
  missing: { name: string; count: number }[];
  /** Chains in the file that came across. */
  chains: number;
}

/** Voice Chain presets from the racks in an EffectPresets.xml, named "<rack> (Audition)". */
export function importAuditionRacks(xml: string): AuditionImport {
  const racks = readRacks(xml);
  const missing = new Map<string, number>();
  const presets: ChainPreset[] = [];
  for (const rack of racks) {
    const slots: ChainPreset["chain"]["slots"] = [];
    for (const s of rack.slots) {
      const build = MAP[s.id] ?? BY_NAME[s.name];
      if (!build) {
        missing.set(s.name, (missing.get(s.name) ?? 0) + 1);
        continue;
      }
      // An effect switched off in the rack stays out, so the chain sounds as it did there.
      if (s.on) slots.push(build(s.values));
    }
    if (slots.length === 0) continue;
    presets.push({ name: `${rack.name} (Audition)`, group: "Voice chains", chain: { enabled: true, input_gain_db: 0, output_gain_db: 0, mix: 1, slots } });
  }
  return { presets, missing: [...missing].map(([name, count]) => ({ name, count })).sort((a, b) => b.count - a.count), chains: presets.length };
}
