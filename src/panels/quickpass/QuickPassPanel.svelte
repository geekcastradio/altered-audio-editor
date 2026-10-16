<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  QuickPass: scans the selected clips (or every clip on a track that
  plays when none are selected) for silences. The Scan view lists what
  was found; click a row to select it and jump to it, then fix the
  selected rows or all of them. The Settings view holds the two levels,
  what fixing does, and saved presets. Nothing changes until you press
  Fix, and every change can be undone.
-->
<script lang="ts">
  import { quickpassLevels, type ClipLevels } from "../../app/core";
  import { editor } from "../../stores/editor";
  import type { CommandRunner } from "../../stores/media-pool";
  import type { Preferences } from "../../stores/preferences";
  import type { SessionDoc, SessionStore } from "../../stores/session";
  import type { TransportStore } from "../../stores/transport";
  import { formatIn } from "../editor/timeline";
  import {
    BUILTIN_PRESETS,
    clipsOn,
    cutRange,
    DEFAULTS,
    findSilences,
    markersFor,
    removeOp,
    sanitizeSettings,
    shiftFindings,
    splitOp,
    suggestLevels,
    type Finding,
    type Preset,
    type QuickPassSettings,
  } from "./quickpass";

  interface Props {
    session: SessionStore;
    transport: TransportStore;
    runner: CommandRunner;
    prefs: Preferences;
    ask: (title: string, label: string, value: string) => Promise<string | null>;
    onNotice: (text: string) => void;
    onError: (e: unknown) => void;
  }

  const { session, transport, runner, prefs, ask, onNotice, onError }: Props = $props();

  const KEY = "quickpass.settings";
  const PRESETS_KEY = "quickpass.presets";
  const WINDOW_MS = 10;
  const VERB = { remove: "Remove", split: "Split", mark: "Mark" } as const;

  let s = $state<QuickPassSettings>({ ...DEFAULTS });
  let mine = $state<Preset[]>([]);
  let preset = $state("Default");
  let loaded = $state(false);
  // svelte-ignore state_referenced_locally
  void prefs.load().then(() => {
    s = sanitizeSettings(prefs.raw(KEY));
    const raw = prefs.raw(PRESETS_KEY);
    mine = Array.isArray(raw)
      ? raw.flatMap((p: unknown) => {
          const r = p as { name?: unknown; settings?: unknown };
          return typeof r?.name === "string" && r.name.trim() ? [{ name: r.name, settings: sanitizeSettings(r.settings) }] : [];
        })
      : [];
    const saved = prefs.raw("quickpass.preset");
    if (typeof saved === "string") preset = saved;
    loaded = true;
  });
  $effect(() => {
    const snap = $state.snapshot(s);
    const list = $state.snapshot(mine);
    const name = preset;
    if (!loaded) return;
    prefs.set(KEY, snap);
    prefs.set(PRESETS_KEY, list);
    prefs.set("quickpass.preset", name);
  });

  const allPresets = $derived([...BUILTIN_PRESETS, ...mine]);
  const isBuiltin = $derived(BUILTIN_PRESETS.some((p) => p.name === preset));
  const known = $derived(allPresets.some((p) => p.name === preset));

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

  type Done = "removed" | "split" | "marked";
  let view = $state<"scan" | "settings">("scan");
  let found = $state<Finding[]>([]);
  let done = $state<Record<string, Done>>({});
  let picked = $state<ReadonlySet<string>>(new Set());
  let scanned = $state(false);
  let busy = $state(false);
  const rate = $derived(doc?.sample_rate ?? 48000);
  const waiting = $derived(found.filter((f) => !done[f.id]));
  const target = $derived(picked.size > 0 ? waiting.filter((f) => picked.has(f.id)) : []);

  function choosePreset(name: string) {
    const p = allPresets.find((x) => x.name === name);
    if (!p) return;
    preset = name;
    s = { ...p.settings };
  }

  async function savePreset() {
    const name = (await ask("Save Preset", "Preset name", known && !isBuiltin ? preset : ""))?.trim();
    if (!name) return;
    if (BUILTIN_PRESETS.some((p) => p.name === name)) return onNotice("That name is used by a built-in preset. Pick another.");
    mine = [...mine.filter((p) => p.name !== name), { name, settings: { ...s } }];
    preset = name;
  }

  function deletePreset() {
    if (isBuiltin || !known) return;
    mine = mine.filter((p) => p.name !== preset);
    preset = "Default";
    s = { ...DEFAULTS };
  }

  function targets(): { ids: string[]; info: Map<string, { trackId: string; name: string }> } {
    const info = new Map<string, { trackId: string; name: string }>();
    for (const t of doc?.tracks ?? []) {
      for (const c of t.clips) {
        if (chosen.size > 0 ? chosen.has(c.id) : !t.muted && !c.muted) info.set(c.id, { trackId: t.id, name: c.name });
      }
    }
    return { ids: [...info.keys()], info };
  }
  const scope = $derived(chosen.size > 0 ? `${chosen.size} selected ${chosen.size === 1 ? "clip" : "clips"}` : "all clips on tracks that play");

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
    if (ids.length === 0) return onNotice("There are no clips to scan.");
    try {
      found = findSilences(await measure(ids), info, s, rate);
      done = {};
      picked = new Set();
      scanned = true;
      view = "scan";
    } catch (e) {
      onError(e);
    }
  }

  async function findLevels() {
    const { ids } = targets();
    if (ids.length === 0) return onNotice("There are no clips to measure.");
    try {
      const r = suggestLevels((await measure(ids)).flatMap((l) => l.db));
      if (!r) return onNotice("There is not enough sound to suggest levels. Set them by hand.");
      s = { ...s, ...r };
    } catch (e) {
      onError(e);
    }
  }

  const uuid = () => crypto.randomUUID();
  const t = (samples: number) => formatIn(samples, rate, "time");

  async function fix(list: Finding[]) {
    if (!doc || list.length === 0) return;
    try {
      if (s.action === "remove") {
        const op = removeOp(doc, list, s, uuid);
        if (!op) return onNotice("Nothing to remove there. The silences are shorter than the pause you keep.");
        await session.edit(list.length === 1 ? "Remove Silence" : "Remove Silences", op, runner);
        let rest = found.filter((f) => !list.includes(f));
        for (const g of [...list].sort((a, b) => b.start - a.start)) rest = shiftFindings(rest, g, s.ripple, s.keepMs, rate);
        const next: Record<string, Done> = {};
        for (const f of list) next[f.id] = "removed";
        found = [...rest, ...list].sort((a, b) => a.start - b.start);
        done = { ...done, ...next };
      } else if (s.action === "split") {
        const op = splitOp(doc, list, uuid);
        if (!op) return onNotice("Nothing to split there. Locked clips are left alone.");
        await session.edit("Split at Silences", op, runner);
        if (session.current) editor.selectClips(clipsOn(session.current, list));
        done = { ...done, ...Object.fromEntries(list.map((f) => [f.id, "split" as const])) };
      } else {
        await session.edit(list.length === 1 ? "Mark Silence" : "Mark Silences", { op: "add_markers", markers: markersFor(doc, list, uuid) }, runner);
        done = { ...done, ...Object.fromEntries(list.map((f) => [f.id, "marked" as const])) };
      }
      picked = new Set();
    } catch (e) {
      onError(e);
    }
  }

  function clearFixed() {
    found = found.filter((f) => !done[f.id]);
    done = {};
  }

  function pick(f: Finding, e: MouseEvent) {
    const next = new Set(e.ctrlKey || e.metaKey ? picked : []);
    if (e.ctrlKey || e.metaKey) {
      if (next.has(f.id)) next.delete(f.id);
      else next.add(f.id);
    } else next.add(f.id);
    picked = next;
    void transport.seek(f.start);
  }

  const field = (e: Event) => Number((e.currentTarget as HTMLInputElement).value);
  const set = (patch: Partial<QuickPassSettings>) => (s = sanitizeSettings({ ...s, ...patch }));
  const doneLabel = { removed: "Removed", split: "Split", marked: "Marked" } as const;
</script>

<section class="qp" aria-label="QuickPass">
  <div class="top">
    <label class="line">Scan for
      <select aria-label="What to scan for">
        <option>Silence</option>
        <option disabled>Clicks and pops (not built yet)</option>
        <option disabled>Clipping (not built yet)</option>
      </select>
    </label>
    <div class="line">
      <label class="grow" for="qp-preset">Preset</label>
      <select id="qp-preset" value={known ? preset : ""} onchange={(e) => choosePreset(e.currentTarget.value)}>
        {#if !known}<option value="" disabled>Custom</option>{/if}
        {#each allPresets as p (p.name)}<option value={p.name}>{p.name}</option>{/each}
      </select>
      <button type="button" class="icon" title="Save the current settings as a preset" aria-label="Save preset" onclick={() => void savePreset()}>
        <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 2.5h8l2.5 2.5v8.5h-10.5zM5.5 2.5v3.5h4v-3.5M5.5 13.5v-4h5v4" fill="none" stroke="currentColor" stroke-width="1.2" /></svg>
      </button>
      <button type="button" class="icon" disabled={isBuiltin || !known} title={isBuiltin ? "Built-in presets cannot be deleted" : "Delete this preset"} aria-label="Delete preset" onclick={deletePreset}>
        <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 4.5h9M6 4.5v-2h4v2M4.5 4.5l.6 9h5.8l.6-9M7 7v4M9 7v4" fill="none" stroke="currentColor" stroke-width="1.2" /></svg>
      </button>
    </div>
    <div class="tabs" role="tablist" aria-label="QuickPass view">
      <button type="button" role="tab" aria-selected={view === "scan"} class:on={view === "scan"} onclick={() => (view = "scan")}>Scan</button>
      <button type="button" role="tab" aria-selected={view === "settings"} class:on={view === "settings"} onclick={() => (view = "settings")}>Settings</button>
    </div>
  </div>

  {#if view === "settings"}
    <div class="body settings">
      <fieldset>
        <legend>Define silence as</legend>
        <label>Level below <input type="number" step="1" min="-100" max="-1" value={s.quietDb} onchange={(e) => set({ quietDb: field(e) })} /> dB</label>
        <label>For more than <input type="number" step="10" min="5" max="60000" value={Math.round(s.minQuietSec * 1000)} onchange={(e) => set({ minQuietSec: field(e) / 1000 })} /> ms</label>
      </fieldset>
      <fieldset>
        <legend>Define sound as</legend>
        <label>Level above <input type="number" step="1" min="-100" max="0" value={s.contentDb} onchange={(e) => set({ contentDb: field(e) })} /> dB</label>
        <label>For more than <input type="number" step="10" min="10" max="10000" value={Math.round(s.minContentSec * 1000)} onchange={(e) => set({ minContentSec: field(e) / 1000 })} /> ms</label>
      </fieldset>
      <button type="button" class="wide" disabled={busy} onclick={findLevels} title="Measure the clips and set both levels to suit them.">Find Levels</button>
      <fieldset>
        <legend>Fix by</legend>
        <label>Action
          <select value={s.action} onchange={(e) => set({ action: e.currentTarget.value as QuickPassSettings["action"] })}>
            <option value="remove">Removing the silence</option>
            <option value="split">Splitting at the silence</option>
            <option value="mark">Marking the silence</option>
          </select>
        </label>
        {#if s.action === "remove"}
          <label title="Leave this much of each silence so speech does not run together.">Keep a pause of <input type="number" step="10" min="0" max="5000" value={s.keepMs} onchange={(e) => set({ keepMs: field(e) })} /> ms</label>
          <label title="Where the gap closes. This track only leaves other tracks where they are.">Close the gap on
            <select value={s.ripple} onchange={(e) => set({ ripple: e.currentTarget.value === "all" ? "all" : "track" })}>
              <option value="track">this track only</option>
              <option value="all">all tracks</option>
            </select>
          </label>
        {/if}
      </fieldset>
    </div>
  {:else}
    <div class="bar">
      <button type="button" class="go" disabled={busy} onclick={scan}>{busy ? "Scanning" : "Scan"}</button>
      <span class="grow"></span>
      <button type="button" disabled={target.length === 0} onclick={() => void fix(target)}>{VERB[s.action]}</button>
      <button type="button" disabled={waiting.length === 0} onclick={() => void fix(waiting)}>{VERB[s.action]} All</button>
      <button type="button" disabled={Object.keys(done).length === 0} onclick={clearFixed}>Clear Fixed</button>
    </div>
    <div class="body table" role="table" aria-label="Silences found">
      <div class="head" role="row"><span>Fixed</span><span>Start</span><span>Length</span><span>Clip</span></div>
      {#if !scanned}
        <p class="empty">Scans {scope}. Press Scan to list the silences. Nothing changes until you fix them.</p>
      {:else if found.length === 0}
        <p class="empty">No silences found. Try a higher silence level or a shorter time in Settings.</p>
      {:else}
        {#each found as f (f.id)}
          <button type="button" role="row" class="row" class:picked={picked.has(f.id)} class:fixed={!!done[f.id]} disabled={!!done[f.id]} onclick={(e) => pick(f, e)}>
            <span>{done[f.id] ? doneLabel[done[f.id]!] : ""}</span>
            <span class="num">{t(f.start)}</span>
            <span class="num">{t(f.end - f.start)}</span>
            <span class="nm">{f.clipName}</span>
          </button>
        {/each}
      {/if}
    </div>
    <p class="status" aria-live="polite">
      {#if !scanned}Ready.{:else}{waiting.length} {waiting.length === 1 ? "silence" : "silences"} to fix{#if Object.keys(done).length > 0}, {Object.keys(done).length} fixed{/if}.{/if}
    </p>
  {/if}
</section>

<style>
  .qp {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: var(--surface-1);
    font-size: 12px;
  }
  .top {
    display: grid;
    gap: 5px;
    padding: 6px 8px;
    border-bottom: 1px solid var(--border);
  }
  .line {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .line select,
  .grow {
    flex: 1;
    min-width: 0;
  }
  .line > label.grow {
    flex: 0 0 auto;
  }
  select,
  input[type="number"] {
    background: var(--surface-0);
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    color: var(--text);
    font: inherit;
    padding: 1px 4px;
  }
  input[type="number"] {
    width: 58px;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 22px;
    height: 20px;
    padding: 0;
    border: 1px solid transparent;
    background: none;
    color: var(--text-dim);
  }
  .icon svg {
    width: 14px;
    height: 14px;
  }
  .icon:hover:not(:disabled) {
    border-color: var(--border-strong);
    color: var(--text);
  }
  .icon:disabled {
    opacity: 0.4;
  }
  .tabs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    overflow: hidden;
  }
  .tabs button {
    padding: 3px 0;
    border: none;
    background: var(--surface-0);
    color: var(--text-dim);
  }
  .tabs button.on {
    background: var(--accent);
    color: var(--on-accent, #fff);
  }
  .body {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .settings {
    display: grid;
    align-content: start;
    gap: 8px;
    padding: 8px;
  }
  fieldset {
    display: grid;
    gap: 5px;
    margin: 0;
    padding: 6px 8px 8px;
    border: 1px solid var(--border);
    border-radius: 3px;
  }
  legend {
    padding: 0 4px;
    color: var(--text-dim);
  }
  fieldset label {
    display: flex;
    gap: 6px;
    align-items: center;
    justify-content: space-between;
  }
  .wide {
    justify-self: start;
  }
  .bar {
    display: flex;
    gap: 4px;
    align-items: center;
    padding: 4px 8px;
    border-bottom: 1px solid var(--border);
  }
  .go {
    border-color: var(--accent);
  }
  .table {
    background: var(--surface-0);
  }
  .head,
  .row {
    display: grid;
    grid-template-columns: 62px 78px 78px 1fr;
    gap: 6px;
    padding: 2px 8px;
    text-align: left;
  }
  .head {
    position: sticky;
    top: 0;
    background: var(--surface-1);
    color: var(--text-dim);
    border-bottom: 1px solid var(--border);
  }
  .row {
    width: 100%;
    border: none;
    background: none;
    color: var(--text);
    font: inherit;
  }
  .row:hover:not(:disabled) {
    background: var(--surface-1);
  }
  .row.picked {
    background: var(--accent);
    color: var(--on-accent, #fff);
  }
  .row.fixed {
    color: var(--text-faint);
  }
  .num {
    font-variant-numeric: tabular-nums;
  }
  .nm {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty {
    margin: 0;
    padding: 10px 8px;
    color: var(--text-dim);
  }
  .status {
    margin: 0;
    padding: 3px 8px;
    border-top: 1px solid var(--border);
    color: var(--text-dim);
  }
</style>
