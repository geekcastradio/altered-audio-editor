// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Batch chains: the processing half of Batch Convert saved under a name.
 * A chain is plain data from a closed set of steps (a Voice Chain preset,
 * trim silence, loudness, peak normalize, fades, and optionally an output
 * format), so it can be saved, shared as text and read back safely. File
 * names, tags and the output folder are per job and never part of a chain.
 */

import { sanitizeFormat, type OutputFormat } from "../../app/output-format";
import { DEFAULT_LEVEL, sanitizeLevel, type BatchSettings, type LevelSettings } from "./batch";

export interface BatchChain {
  name: string;
  /** Voice Chain preset run first; null runs none. */
  process: string | null;
  level: LevelSettings;
  /** Output format; null leaves the one already chosen. */
  format: OutputFormat | null;
}

export const MAX_CHAINS = 40;
export const MAX_NAME = 60;

const clean = (name: string) => name.replace(/\s+/g, " ").trim().slice(0, MAX_NAME);

/** Starter chains: the common jobs, set up with sensible numbers. */
export const BUILTIN_CHAINS: readonly BatchChain[] = [
  { name: "Podcast, -16 LUFS", process: null, level: { ...DEFAULT_LEVEL, trim: true, loudness: true, loudnessLufs: -16, ceilingDb: -1, fadeOutMs: 300 }, format: null },
  { name: "Voiceover, trim and level", process: null, level: { ...DEFAULT_LEVEL, trim: true, trimKeepMs: 400, loudness: true, loudnessLufs: -19, ceilingDb: -3 }, format: null },
  { name: "Audiobook, quiet and clean", process: null, level: { ...DEFAULT_LEVEL, trim: true, trimKeepMs: 700, loudness: true, loudnessLufs: -20, ceilingDb: -3 }, format: null },
  { name: "Quick peak match, -1 dB", process: null, level: { ...DEFAULT_LEVEL, normalize: true, normalizeDb: -1 }, format: null },
];

/** Chains read back from saved settings or a shared file; anything odd is dropped. */
export function sanitizeChains(raw: unknown): BatchChain[] {
  if (!Array.isArray(raw)) return [];
  const seen = new Set<string>();
  const out: BatchChain[] = [];
  for (const item of raw) {
    if (!item || typeof item !== "object") continue;
    const v = item as Record<string, unknown>;
    const name = typeof v.name === "string" ? clean(v.name) : "";
    if (!name || seen.has(name.toLowerCase())) continue;
    seen.add(name.toLowerCase());
    out.push({
      name,
      process: typeof v.process === "string" && v.process ? v.process : null,
      level: sanitizeLevel(v.level),
      format: v.format && typeof v.format === "object" ? sanitizeFormat(v.format) : null,
    });
    if (out.length >= MAX_CHAINS) break;
  }
  return out;
}

/** The current Batch settings as a chain. `withFormat` also keeps the output format. */
export function chainFrom(name: string, s: Pick<BatchSettings, "process" | "level" | "format">, withFormat: boolean): BatchChain {
  return { name: clean(name), process: s.process, level: { ...s.level }, format: withFormat ? { ...s.format } : null };
}

/** Saves a chain: one with the same name (any case) is replaced. Null when the name is empty. */
export function saveChain(chains: readonly BatchChain[], chain: BatchChain): BatchChain[] | null {
  const name = clean(chain.name);
  if (!name) return null;
  const key = name.toLowerCase();
  const next = chains.filter((c) => c.name.toLowerCase() !== key);
  next.push({ ...chain, name });
  return next.slice(-MAX_CHAINS);
}

export function removeChain(chains: readonly BatchChain[], name: string): BatchChain[] {
  return chains.filter((c) => c.name !== name);
}

/** The Batch settings with a chain's steps in place; naming, tags and folder stay as they are. */
export function applyChain(s: BatchSettings, chain: BatchChain): BatchSettings {
  return { ...s, process: chain.process, level: { ...chain.level }, format: chain.format ? { ...chain.format } : s.format };
}

/** Plain words for a chain's steps, in the order they run. */
export function describeChain(c: BatchChain): string {
  const l = c.level;
  const steps: string[] = [];
  if (c.process) steps.push(c.process);
  if (l.trim) steps.push(`trim silence below ${l.trimDb} dB`);
  if (l.loudness) steps.push(`${l.loudnessLufs} LUFS, peaks under ${l.ceilingDb} dB`);
  if (l.normalize) steps.push(`peak to ${l.normalizeDb} dB`);
  if (l.fadeInMs > 0) steps.push(`${l.fadeInMs} ms fade in`);
  if (l.fadeOutMs > 0) steps.push(`${l.fadeOutMs} ms fade out`);
  if (c.format) steps.push(`save as ${c.format.kind.toUpperCase()}`);
  return steps.length ? steps.join(", then ") : "no steps";
}
