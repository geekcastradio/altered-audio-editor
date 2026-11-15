// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * What each process chain action does, with no dialog: the same edits the
 * menu commands make, with their settings read from the chain.
 */

import { closeDeadSpace } from "../panels/quickpass/dead-space";
import { LockedError } from "../panels/editor/clip-edit";
import { describeMix, micTracks, mixOps, type MixAnalysis, type MixOptions } from "./multitrack";
import type { ChainDeps } from "./chains";
import { guessRegion, predictExport, targetById, type LoudnessReport } from "./loudness";
import { applyPreset, BUILTIN_PRESETS as VOICE_PRESETS, chainOf, type ChainPreset } from "../panels/voice-chain/chain";
import { DEFAULTS as QP_DEFAULTS, BUILTIN_PRESETS as QP_PRESETS, duckOp, findAll, removeOp, scanInfo, type QuickPassSettings } from "../panels/quickpass/quickpass";
import type { ClipLevels } from "./core";
import type { Op, SessionDoc } from "../stores/session";

/** The Master fader's range, matching its dial. */
const MASTER_MIN = -60;
const MASTER_MAX = 12;

/**
 * The Master change that lands the mix on a loudness target, or null when it is
 * already close enough, silent, or cannot be raised without passing the target's
 * peak limit. Uses the same rule as the export, so what the chain sets is what
 * an export to that target would apply.
 */
export function loudnessTargetOp(report: LoudnessReport, masterDb: number, targetId: string, toleranceDb: number): { op: Op | null; message: string } {
  const t = targetById(targetId, guessRegion());
  const pred = predictExport(report, t, t.mono);
  if (!Number.isFinite(pred.loudness)) return { op: null, message: "the mix is silent or too short to measure." };
  if (Math.abs(pred.gain) <= Math.max(0, toleranceDb) || Math.abs(pred.gain) < 0.05) {
    return { op: null, message: `already at ${pred.loudness.toFixed(1)} LUFS, close enough to ${t.lufs}.` };
  }
  const next = Math.min(MASTER_MAX, Math.max(MASTER_MIN, Math.round((masterDb + pred.gain) * 10) / 10));
  if (next === masterDb) return { op: null, message: "the Master is already at its limit." };
  const how = `Master ${pred.gain > 0 ? "raised" : "lowered"} ${Math.abs(next - masterDb).toFixed(1)} dB for ${t.lufs} LUFS`;
  const why = pred.limitedByPeak ? `, stopped by the ${t.peak} dBTP peak limit (lands at ${pred.loudness.toFixed(1)} LUFS)` : "";
  return { op: { op: "set_master_volume", volume_db: next } as Op, message: `${how}${why}.` };
}

const num = (v: unknown, fallback: number): number => (typeof v === "number" && Number.isFinite(v) ? v : fallback);

/** A chain's mic mix settings in the names the analysis takes. */
export function micOptions(p: Record<string, unknown>): MixOptions {
  return {
    openDb: num(p.open_db, 8),
    marginDb: num(p.margin_db, 6),
    reduceDb: num(p.reduce_db, 12),
    fadeMs: num(p.fade_ms, 150),
    attackMs: num(p.attack_ms, 15),
    holdMs: num(p.hold_ms, 250),
  };
}

/** What the app gives steps that need more than the session: measurements, saved presets. */
export interface ChainEnv {
  measure?: () => Promise<LoudnessReport>;
  /** Levels of the given clips in 10 ms windows (what a QuickPass scan reads). */
  levels?: (clipIds: string[]) => Promise<ClipLevels[]>;
  /** The user's saved Voice Chain presets and the effects this build has. */
  presets?: () => Promise<{ saved: ChainPreset[]; known: ReadonlySet<string> }>;
}

/** The tracks a step names: every track with audio, or the one with that name. */
function pickTracks(doc: SessionDoc, which: string) {
  const withAudio = doc.tracks.filter((t) => t.clips.length > 0);
  const name = which.trim().toLowerCase();
  if (name === "" || name === "all") return withAudio;
  const hit = doc.tracks.filter((t) => t.name.trim().toLowerCase() === name);
  if (hit.length === 0) throw new Error(`There is no track called "${which}".`);
  return hit;
}

/**
 * Gives each named track the Voice Chain preset, as one edit. A voice chain replaces
 * the track's chain; a single-effect preset adds its effect to the end.
 */
export function voiceChainPresetOp(doc: SessionDoc, presetName: string, which: string, saved: readonly ChainPreset[], known: ReadonlySet<string>, newId: () => string): { op: Op | null; message: string } {
  const p = [...saved, ...VOICE_PRESETS].find((x) => x.name === presetName);
  if (!p) throw new Error(`There is no Voice Chain preset called "${presetName}".`);
  const tracks = pickTracks(doc, which);
  if (tracks.length === 0) return { op: null, message: "no audio on the tracks." };
  const ops: Op[] = [];
  for (const t of tracks) {
    const next = applyPreset(chainOf(t.effects), p, known, newId);
    if (!next) throw new Error(`The Voice Chain on ${t.name} is full.`);
    ops.push({ op: "set_effects", target: { kind: "track", id: t.id }, chain: next } as Op);
  }
  const n = ops.length;
  return { op: n === 1 ? ops[0]! : ({ op: "batch", ops } as Op), message: `${p.name} on ${n} track${n === 1 ? "" : "s"}.` };
}

/** The audio a QuickPass step scans: clips that play, on tracks that play, as the panel does. */
export function quickPassClips(doc: SessionDoc): Map<string, { trackId: string; name: string }> {
  const info = new Map<string, { trackId: string; name: string }>();
  for (const t of doc.tracks) {
    if (t.muted) continue;
    for (const c of t.clips) if (!c.locked && !c.muted) info.set(c.id, { trackId: t.id, name: c.name });
  }
  return info;
}

/** The QuickPass preset a step names, with the step's own gap setting. Throws when a chain cannot run it. */
export function quickPassSettings(presetName: string, ripple: string): QuickPassSettings {
  const p = QP_PRESETS.find((x) => x.name === presetName);
  if (!p) throw new Error(`There is no QuickPass preset called "${presetName}".`);
  const s: QuickPassSettings = { ...QP_DEFAULTS, ...p.settings };
  if (s.scan !== "silence" && s.scan !== "noise" && s.scan !== "audio") {
    throw new Error(`"${p.name}" looks for ${scanInfo(s.scan).label.toLowerCase()}, which needs the QuickPass panel. Chains run the silence, noise and audio scans.`);
  }
  if (s.action !== "remove" && s.action !== "duck") throw new Error(`"${p.name}" ends by ${s.action === "split" ? "splitting" : "marking"} what it finds. Chains run presets that remove or turn down.`);
  if (ripple !== "none" && ripple !== "track" && ripple !== "all") throw new Error(`The gap setting must be none, track or all, not "${ripple}".`);
  return { ...s, ripple };
}

/** What a QuickPass step does with levels already measured. */
export function quickPassOp(doc: SessionDoc, levels: readonly ClipLevels[], s: QuickPassSettings, newId: () => string): { op: Op | null; message: string } {
  const found = findAll(levels, quickPassClips(doc), s, doc.sample_rate);
  if (found.length === 0) return { op: null, message: "found nothing." };
  const op = s.action === "remove" ? removeOp(doc, found, s, newId) : duckOp(doc, found, s.duckDb);
  if (!op) return { op: null, message: `found ${found.length}, but nothing was long enough to change.` };
  return { op, message: `${s.action === "remove" ? "removed" : "turned down"} ${found.length} ${found.length === 1 ? "stretch" : "stretches"}.` };
}

export function chainHandlers(analyze: (ids: string[], options: MixOptions) => Promise<MixAnalysis>, env: ChainEnv = {}): ChainDeps["handlers"] {
  return {
    "multitrack.close-dead-space": {
      title: "Close Dead Space",
      run: (doc, p) => {
        const ids = doc.tracks.filter((t) => t.clips.length > 0 || (t.midi_clips?.length ?? 0) > 0).map((t) => t.id);
        if (ids.length === 0) return { op: null, message: "no audio on the tracks." };
        const rate = doc.sample_rate;
        let r;
        try {
          r = closeDeadSpace(doc, ids, { pause: Math.round(num(p.pause_seconds, 0.5) * rate), minRemove: Math.round(0.1 * rate) });
        } catch (e) {
          if (e instanceof LockedError) throw new Error(e.message);
          throw e;
        }
        if (r === "blocked") throw new Error("A track has MIDI clips, which cannot be moved this way yet.");
        if (!r) return { op: null, message: "no dead space longer than the pause." };
        const n = r.spaces.length;
        return { op: r.op, message: `closed ${n} stretch${n === 1 ? "" : "es"}, ${(r.removed / rate).toFixed(1)} s shorter.` };
      },
    },
    "multitrack.auto-mic-mix": {
      title: "Auto Mic Mix",
      run: async (doc, p) => {
        const ids = micTracks(doc).map((t) => t.id);
        if (ids.length < 2) return { op: null, message: "needs two or more tracks with audio, one per mic." };
        const a = await analyze(ids, micOptions(p));
        const op = mixOps(doc, a);
        return op ? { op, message: describeMix(a) } : { op: null, message: "nothing to write." };
      },
    },
    "multitrack.voice-chain-preset": {
      title: "Voice Chain Preset",
      run: async (doc, p) => {
        if (!env.presets) throw new Error("Presets cannot be read here.");
        const { saved, known } = await env.presets();
        return voiceChainPresetOp(doc, String(p.preset ?? "Podcast Voice"), String(p.tracks ?? "all"), saved, known, () => crypto.randomUUID());
      },
    },
    "multitrack.quickpass": {
      title: "QuickPass",
      run: async (doc, p) => {
        if (!env.levels) throw new Error("QuickPass cannot scan here.");
        const s = quickPassSettings(String(p.preset ?? "Clean Up Podcast Interview"), String(p.ripple ?? "none"));
        const ids = [...quickPassClips(doc).keys()];
        if (ids.length === 0) return { op: null, message: "no clips to scan." };
        return quickPassOp(doc, await env.levels(ids), s, () => crypto.randomUUID());
      },
    },
    "multitrack.loudness-target": {
      title: "Loudness Target",
      run: async (doc, p) => {
        if (!env.measure) throw new Error("Loudness cannot be measured here.");
        const report = await env.measure();
        if (report.cancelled) throw new Error("The measurement was cancelled.");
        const tol = typeof p.tolerance_db === "number" ? p.tolerance_db : 0.5;
        return loudnessTargetOp(report, doc.master?.volume_db ?? 0, typeof p.target === "string" ? p.target : "podcast-stereo", tol);
      },
    },
  };
}
