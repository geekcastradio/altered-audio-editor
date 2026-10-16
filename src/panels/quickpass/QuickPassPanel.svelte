<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  QuickPass: scans the selected clips (or every clip when none are
  selected) for silences and lists them. Each finding can be removed
  (the gap closes), split out into its own clip, or marked. Nothing
  changes until you press one of those, and every change is undoable.
-->
<script lang="ts">
  import { quickpassLevels, type ClipLevels } from "../../app/core";
  import { editor } from "../../stores/editor";
  import type { CommandRunner } from "../../stores/media-pool";
  import type { Preferences } from "../../stores/preferences";
  import type { SessionDoc, SessionStore } from "../../stores/session";
  import type { TransportStore } from "../../stores/transport";
  import { formatIn } from "../editor/timeline";
  import { clipsOn, DEFAULTS, findSilences, markersFor, removeOp, sanitizeSettings, shiftFindings, splitOp, suggestLevels, type Finding, type QuickPassSettings } from "./quickpass";

  interface Props {
    session: SessionStore;
    transport: TransportStore;
    runner: CommandRunner;
    prefs: Preferences;
    onNotice: (text: string) => void;
    onError: (e: unknown) => void;
  }

  const { session, transport, runner, prefs, onNotice, onError }: Props = $props();

  const KEY = "quickpass.settings";
  const WINDOW_MS = 10;
  let s = $state<QuickPassSettings>({ ...DEFAULTS });
  let loaded = $state(false);
  // svelte-ignore state_referenced_locally
  void prefs.load().then(() => {
    s = sanitizeSettings(prefs.raw(KEY));
    loaded = true;
  });
  $effect(() => {
    const snap = $state.snapshot(s);
    if (loaded) prefs.set(KEY, snap);
  });

  let doc = $state<SessionDoc | null>(null);
  $effect(() =>
    session.subscribe((d) => {
      doc = d;
    }),
  );
  let chosen = $state<ReadonlySet<string>>(new Set());
  $effect(() =>
    editor.subscribe((e) => {
      chosen = e.selectedClips;
    }),
  );

  let found = $state<Finding[]>([]);
  let scanned = $state(false);
  let busy = $state(false);
  const rate = $derived(doc?.sample_rate ?? 48000);

  /** Clips to look at: the selection, else every clip on a track that plays. */
  function targets(): { ids: string[]; info: Map<string, { trackId: string; name: string }> } {
    const info = new Map<string, { trackId: string; name: string }>();
    for (const t of doc?.tracks ?? []) {
      for (const c of t.clips) {
        if (chosen.size > 0 ? chosen.has(c.id) : !t.muted && !c.muted) info.set(c.id, { trackId: t.id, name: c.name });
      }
    }
    return { ids: [...info.keys()], info };
  }
  const scope = $derived(chosen.size > 0 ? `${chosen.size} selected ${chosen.size === 1 ? "clip" : "clips"}` : "every clip on the tracks that play");

  async function measure(ids: string[]): Promise<ClipLevels[]> {
    busy = true;
    try {
      return await quickpassLevels(ids, WINDOW_MS);
    } finally {
      busy = false;
    }
  }

  async function scan() {
    const { ids, info } = targets();
    if (ids.length === 0) {
      onNotice("There are no clips to scan.");
      return;
    }
    try {
      const levels = await measure(ids);
      found = findSilences(levels, info, s, rate);
      scanned = true;
    } catch (e) {
      onError(e);
    }
  }

  async function suggest() {
    const { ids } = targets();
    if (ids.length === 0) {
      onNotice("There are no clips to measure.");
      return;
    }
    try {
      const levels = await measure(ids);
      const r = suggestLevels(levels.flatMap((l) => l.db));
      if (!r) {
        onNotice("There is not enough sound to suggest levels. Set them by hand.");
        return;
      }
      s = { ...s, ...r };
    } catch (e) {
      onError(e);
    }
  }

  const uuid = () => crypto.randomUUID();
  const t = (samples: number) => formatIn(samples, rate, "time");
  const total = $derived(found.reduce((n, f) => n + (f.end - f.start), 0));

  async function remove(list: Finding[]) {
    if (!doc || list.length === 0) return;
    try {
      const op = removeOp(doc, list, s, uuid);
      if (!op) return onNotice("Nothing to remove there.");
      await session.edit(list.length === 1 ? "Remove Silence" : "Remove Silences", op, runner);
      let rest = found.filter((f) => !list.includes(f));
      for (const g of [...list].sort((a, b) => b.start - a.start)) rest = shiftFindings(rest, g, s.ripple);
      found = rest;
    } catch (e) {
      onError(e);
    }
  }

  async function split(list: Finding[]) {
    if (!doc || list.length === 0) return;
    try {
      const op = splitOp(doc, list, uuid);
      if (!op) return onNotice("Nothing to split there. Locked clips are left alone.");
      await session.edit("Split at Silences", op, runner);
      const after = session.current;
      if (after) editor.selectClips(clipsOn(after, list));
    } catch (e) {
      onError(e);
    }
  }

  async function mark(list: Finding[]) {
    if (!doc || list.length === 0) return;
    try {
      await session.edit(list.length === 1 ? "Mark Silence" : "Mark Silences", { op: "add_markers", markers: markersFor(doc, list, uuid) }, runner);
    } catch (e) {
      onError(e);
    }
  }

  const field = (e: Event) => Number((e.currentTarget as HTMLInputElement).value);
  const set = (patch: Partial<QuickPassSettings>) => (s = sanitizeSettings({ ...s, ...patch }));
</script>

<section class="qp" aria-label="QuickPass">
  <div class="settings">
    <p class="scope">Scans {scope}.</p>
    <div class="grid">
      <label title="Audio below this level counts as quiet.">Quiet below
        <input type="number" step="1" min="-100" max="-1" value={s.quietDb} onchange={(e) => set({ quietDb: field(e) })} /> dB</label>
      <label title="Quiet must last at least this long to count as a silence.">for at least
        <input type="number" step="0.1" min="0.05" max="60" value={s.minQuietSec} onchange={(e) => set({ minQuietSec: field(e) })} /> s</label>
      <label title="Audio at or above this level counts as speech.">Speech from
        <input type="number" step="1" min="-100" max="0" value={s.contentDb} onchange={(e) => set({ contentDb: field(e) })} /> dB</label>
      <label title="Sound shorter than this does not break up a silence.">for at least
        <input type="number" step="0.05" min="0.01" max="10" value={s.minContentSec} onchange={(e) => set({ minContentSec: field(e) })} /> s</label>
    </div>
    <label class="ripple" title="Where Remove closes the gap. This track only leaves other tracks where they are.">Remove closes the gap on
      <select value={s.ripple} onchange={(e) => set({ ripple: e.currentTarget.value === "all" ? "all" : "track" })}>
        <option value="track">this track only</option>
        <option value="all">all tracks (keeps them in sync)</option>
      </select>
    </label>
    <div class="row">
      <button type="button" disabled={busy} onclick={suggest} title="Measure the clips and set the two levels to suit them.">Suggest Levels</button>
      <button type="button" class="primary" disabled={busy} onclick={scan}>{busy ? "Scanning…" : "Scan"}</button>
    </div>
  </div>

  <div class="results" aria-live="polite">
    {#if !scanned}
      <p class="empty">Scan to list the silences. Nothing changes until you remove, split or mark.</p>
    {:else if found.length === 0}
      <p class="empty">No silences found. Try a higher quiet level or a shorter time.</p>
    {:else}
      <p class="sum">{found.length} {found.length === 1 ? "silence" : "silences"}, {t(total)} in all</p>
      <ul>
        {#each found as f (f.id)}
          <li>
            <button type="button" class="where" title="Move the playhead here" onclick={() => void transport.seek(f.start)}>
              <span class="at">{t(f.start)}</span>
              <span class="len">{t(f.end - f.start)}</span>
              <span class="nm">{f.clipName}</span>
            </button>
            <button type="button" onclick={() => void remove([f])} title="Cut this silence out and close the gap">Remove</button>
            <button type="button" onclick={() => void split([f])} title="Cut the clip at both ends of this silence and select it">Split</button>
            <button type="button" onclick={() => void mark([f])} title="Add a range marker over this silence">Mark</button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <div class="all">
    <button type="button" disabled={found.length === 0} onclick={() => void remove(found)}>Remove All</button>
    <button type="button" disabled={found.length === 0} onclick={() => void split(found)}>Split All</button>
    <button type="button" disabled={found.length === 0} onclick={() => void mark(found)}>Mark All</button>
  </div>
</section>

<style>
  .qp {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr) auto;
    height: 100%;
    min-height: 0;
    background: var(--surface-1);
    font-size: var(--text-sm, 12px);
  }
  .settings {
    display: grid;
    gap: 6px;
    padding: 6px 8px;
    border-bottom: 1px solid var(--border);
  }
  .scope,
  .empty,
  .sum {
    margin: 0;
    color: var(--text-dim);
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px 10px;
  }
  label {
    display: flex;
    gap: 4px;
    align-items: center;
    color: var(--text);
  }
  input[type="number"] {
    width: 54px;
  }
  .row,
  .all {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
    padding: 4px 8px;
  }
  .all {
    border-top: 1px solid var(--border);
  }
  .results {
    min-height: 0;
    overflow: auto;
    padding: 6px 8px;
    background: var(--surface-0);
  }
  ul {
    margin: 4px 0 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: flex;
    gap: 4px;
    align-items: center;
    padding: 1px 0;
  }
  .where {
    display: flex;
    flex: 1;
    gap: 8px;
    min-width: 0;
    border: none;
    background: none;
    color: var(--text);
    text-align: left;
  }
  .where:hover {
    background: var(--surface-2, var(--surface-1));
  }
  .at,
  .len {
    font-variant-numeric: tabular-nums;
  }
  .len {
    color: var(--text-dim);
  }
  .nm {
    overflow: hidden;
    color: var(--text-dim);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .primary {
    border-color: var(--accent);
  }
</style>
