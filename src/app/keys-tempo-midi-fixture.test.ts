// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

// Checks the TypeScript key chords, library list, tempo map and MIDI edits against the same
// fixture the Rust test reads (shared-fixtures/keys-tempo-midi.json).
import { describe, expect, it } from "vitest";
import fx from "../../shared-fixtures/keys-tempo-midi.json";
import { chordFromEvent, parseChord } from "../keymap/chord";
import { lastSegment, MAX_KNOWN, packLibraryList, readLibraryList, sameDir, sanitizeKnown } from "../stores/library-list";
import * as M from "../panels/editor/midi";
import * as T from "../stores/tempo";
import type { MidiClipDoc, NoteDoc } from "../stores/session";

const J = (v: unknown) => JSON.parse(JSON.stringify(v));
const tempo = (v: unknown) => v as T.TempoDoc;
const set = (v: string[]) => new Set(v);
const idx = (v: number[]) => new Set(v);
const clips = (v: unknown) => v as MidiClipDoc[];
const notes = (v: unknown) => v as NoteDoc[];
const counter = (prefix: string) => {
  let n = 0;
  return () => `${prefix}${n++}`;
};

describe("keys, tempo and MIDI fixture", () => {
  it("chords", () => {
    for (const c of fx.chords) expect(parseChord(c.text)).toBe(c.out);
    for (const c of fx.events) expect(chordFromEvent(c.ev as never)).toBe(c.out);
  });

  it("library lists", () => {
    for (const c of fx.libSan) expect(J(sanitizeKnown((c.raw as { __undefined?: boolean } | null)?.__undefined ? undefined : c.raw))).toEqual(c.out);
    const big = Array.from({ length: 520 }, (_, i) => ({ dir: `/lib/${i}`, name: `L${i}` }));
    expect(sanitizeKnown(big).length).toBe(MAX_KNOWN);
    for (const c of fx.packs) expect(packLibraryList(c.known)).toBe(c.text);
    for (const c of fx.reads) expect(J(readLibraryList(c.text))).toEqual(c.out);
    for (const c of fx.segs) expect(lastSegment(c.dir)).toBe(c.out);
    for (const c of fx.sames) expect(sameDir(c.a, c.b)).toBe(c.out);
  });

  it("tempo map", () => {
    for (const c of fx.tempo) {
      const d = tempo(c.doc);
      expect(J(T.segments(d))).toEqual(c.segments);
      for (const p of c.beats) {
        expect(T.beatsToSeconds(d, p.b)).toBeCloseTo(p.seconds, 9);
        const at = T.barAt(d, p.b);
        expect([at.bar, at.beat, at.seg.bar]).toEqual([p.bar, p.beat, p.segBar]);
        expect(T.nearestBar(d, p.b)).toBe(p.nearest);
        expect(J(T.segmentAtBeat(d, p.b))).toEqual(p.segAt);
      }
      for (const p of c.secs) {
        expect(T.secondsToBeats(d, p.s)).toBeCloseTo(p.beats, 9);
        expect(J(T.segmentAtSeconds(d, p.s))).toEqual(p.segAt);
      }
      for (const p of c.bars) {
        expect(T.barStart(d, p.b)).toBeCloseTo(p.start, 9);
        expect(J(T.segmentAtBar(d, p.b))).toEqual(p.segAt);
      }
    }
    for (const c of fx.tempoEdits) {
      let d = J(c.start) as T.TempoDoc;
      for (const step of c.steps as { kind: string; id?: string | null; bar?: number; values?: Record<string, number>; out: unknown; outId?: string }[]) {
        if (step.kind === "edit") {
          const seg = T.segments(d).find((s) => s.id === step.id)!;
          d = T.editSegment(d, seg, step.values!);
        } else if (step.kind === "add") {
          const r = T.addChange(d, step.bar!, step.id!);
          expect(r.id).toBe(step.outId);
          d = r.tempo;
        } else if (step.kind === "move") d = T.moveChange(d, step.id!, step.bar!);
        else d = T.removeChange(d, step.id!);
        expect(J(d)).toEqual(step.out);
      }
    }
    for (const c of fx.bpmTexts) expect(T.bpmText(c.b)).toBe(c.out);
    for (const c of fx.meters) expect(T.meterText((c as { unit?: number }).unit === undefined ? { beats_per_bar: c.bpb } : { beats_per_bar: c.bpb, beat_unit: (c as { unit: number }).unit })).toBe(c.out);
    for (const c of fx.meterTexts) expect(J(T.parseMeter(c.text))).toEqual(c.out);
  });

  it("MIDI math", () => {
    fx.grids.forEach((g, i) => {
      expect(M.GRIDS[i]!.label).toBe(g.label);
      expect(M.GRIDS[i]!.beats).toBeCloseTo(g.beats, 12);
    });
    for (const c of fx.conv) {
      const d = tempo(c.doc);
      const samples = M.beatsToSamples(c.beats, d, c.rate);
      expect(samples).toBe(c.samples);
      expect(M.samplesToBeats(samples, d, c.rate)).toBeCloseTo(c.back, 9);
      expect(M.samplesToBeats(Math.round(c.beats * 1000), d, c.rate)).toBeCloseTo(c.fromSamples, 9);
    }
    for (const c of fx.snaps) {
      expect(M.snap(c.b, c.g)).toBeCloseTo(c.snap, 9);
      expect(M.snapDown(c.b, c.g)).toBeCloseTo(c.down, 9);
    }
    for (const c of fx.gridCases) expect(M.gridBeats(c.g, tempo(c.doc), c.at)).toBeCloseTo(c.out, 12);
    for (const c of fx.names) {
      expect(M.noteName(c.p)).toBe(c.name);
      expect(M.isBlackKey(c.p)).toBe(c.black);
    }
  });

  it("MIDI clips", () => {
    for (const c of fx.clipCases) {
      const d = tempo(c.doc);
      const cl = clips(c.clips);
      const ids = set(c.ids);
      expect(J(M.newClip("nc", c.at, d, c.name ?? undefined))).toEqual(c.newClip);
      expect(M.clipAtBeat(cl, c.beat)?.id ?? null).toBe(c.clipAt);
      expect(J(M.sorted(cl))).toEqual(c.sorted);
      expect(J(M.moveClips(cl, ids, c.delta))).toEqual(c.moved);
      expect(J(M.transferClips(cl, clips(c.to), ids, c.delta, c.copy, counter("new")))).toEqual(c.transferred);
      expect(J(M.resizeClip(cl, c.rid, c.len))).toEqual(c.resized);
      expect(J(M.resizeClip(cl, c.rid, c.len, 0.5))).toEqual(c.resizedMin);
      expect(J(M.removeClips(cl, ids))).toEqual(c.removed);
      expect(J(M.duplicateClips(cl, ids, counter("dup")))).toEqual(c.duplicated);
      if (c.rep) expect(J(M.replaceClip(cl, c.rep as MidiClipDoc))).toEqual(c.replaced);
    }
  });

  it("MIDI notes", () => {
    for (const c of fx.noteCases) {
      const ns = notes(c.notes);
      const ix = idx(c.idx);
      expect(J(M.sortNotes(ns))).toEqual(c.sorted);
      expect(J(M.addNote(ns, c.add))).toEqual(c.added);
      expect(J(M.moveNotes(ns, ix, c.beats, c.semis))).toEqual(c.moved);
      expect(J(M.resizeNotes(ns, ix, c.beats))).toEqual(c.resized);
      expect(J(M.removeNotes(ns, ix))).toEqual(c.removed);
      expect(J(M.setVelocity(ns, ix, c.vel))).toEqual(c.velocity);
      expect(M.pitchRange(ns)).toEqual(c.range);
      expect(J(M.resizeNotesStart(ns, ix, c.beats))).toEqual(c.startResized);
      const copied = M.copyNotes(ns, ix);
      expect(J(copied)).toEqual(c.copied);
      expect(M.spanOf(copied)).toBe(c.span);
      const p = M.pasteNotes(ns, copied, c.at);
      expect({ notes: J(p.notes), pasted: [...p.pasted].sort((a, b) => a - b) }).toEqual(c.pasted);
    }
  });
});
