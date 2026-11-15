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

export function chainHandlers(analyze: (ids: string[], options: MixOptions) => Promise<MixAnalysis>): ChainDeps["handlers"] {
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
  };
}
