// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { describe, expect, it } from "vitest";
import type { SessionDoc } from "../stores/session";
import { currentProperties, mixdownBytes, propertiesOp } from "./session-properties";

const doc = {
  name: "EP 440",
  sample_rate: 48_000,
  bit_depth: 32,
} as SessionDoc;

describe("propertiesOp", () => {
  it("is null when nothing changed", () => {
    expect(propertiesOp(doc, currentProperties(doc))).toBeNull();
    expect(propertiesOp(doc, { ...currentProperties(doc), name: "  " })).toBeNull();
  });

  it("sends only what changed", () => {
    expect(propertiesOp(doc, { name: " EP 441 ", sampleRate: 48_000, bitDepth: 32 })).toEqual({
      op: "set_session_info",
      name: "EP 441",
    });
    expect(propertiesOp(doc, { name: "EP 440", sampleRate: 44_100, bitDepth: 24 })).toEqual({
      op: "batch",
      ops: [
        { op: "set_session_info", bit_depth: 24 },
        { op: "set_sample_rate", sample_rate: 44_100 },
      ],
    });
  });

  it("changes the pan law, and treats a file without one as balance", () => {
    expect(currentProperties(doc).panLaw).toBe("balance");
    expect(propertiesOp(doc, { ...currentProperties(doc), panLaw: "constant_power" })).toEqual({ op: "set_pan_law", pan_law: "constant_power" });
    expect(propertiesOp({ ...doc, pan_law: "constant_power" }, { ...currentProperties(doc), panLaw: "constant_power" })).toBeNull();
    expect(propertiesOp(doc, { name: "EP 440", sampleRate: 48_000, bitDepth: 32 })).toBeNull();
  });

  it("sets and clears the session's loudness target, and a file without one uses the Settings choice", () => {
    expect(currentProperties(doc).loudnessTarget).toBe("");
    expect(propertiesOp(doc, { ...currentProperties(doc), loudnessTarget: "podcast-stereo" })).toEqual({ op: "set_loudness_target", target: "podcast-stereo" });
    const set = { ...doc, loudness_target: "podcast-stereo" } as SessionDoc;
    expect(propertiesOp(set, currentProperties(set))).toBeNull();
    expect(propertiesOp(set, { ...currentProperties(set), loudnessTarget: "" })).toEqual({ op: "set_loudness_target", target: null });
    expect(propertiesOp(doc, { name: "EP 440", sampleRate: 48_000, bitDepth: 32 })).toBeNull();
  });

  it("sizes a mixdown", () => {
    expect(mixdownBytes(48_000, 24)).toBe(288_000);
  });
});
