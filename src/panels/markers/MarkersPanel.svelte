<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Markers panel: every marker in time order. Click a row to jump there,
  double-click the name to rename, change the type from the list, and
  delete with the row's button. Right-click a row for the marker menu.
  Drag the edge of a column heading to resize it; the search box filters
  by name, description or type. Every change is undoable. Follows the
  active editor: the Waveform editor's file markers when it is in front,
  the session's otherwise.
-->
<script lang="ts">
  import type { Readable } from "svelte/store";
  import type { ActionRegistry } from "../../app/actions";
  import type { CommandRunner } from "../../stores/media-pool";
  import { editor, type EditorState } from "../../stores/editor";
  import type { MarkerChange, MarkerDoc, MarkerKind, SessionDoc, SessionStore } from "../../stores/session";
  import type { TransportStore } from "../../stores/transport";
  import ContextMenu, { type ContextItem } from "../../app/ContextMenu.svelte";
  import { formatIn } from "../editor/timeline";
  import { markerMenuItems } from "./marker-menu";
  import { filterMarkers, KIND_LABELS, kindChange, markerCss, markerEnd, markersBetween } from "./markers";
  import { chapterUrl, isChapterUrl, titleTips } from "./chapters";

  interface Props {
    session: SessionStore;
    transport: TransportStore;
    runner: CommandRunner;
    actions: ActionRegistry;
    onError?: (e: unknown) => void;
    /** The Waveform editor's store, history and transport, when present. */
    wave?: SessionStore;
    waveRunner?: CommandRunner;
    waveTransport?: TransportStore;
    /** The open Waveform editor file; null when it is empty. */
    waveFile?: Readable<unknown>;
  }

  const { session, transport, runner, actions, onError = console.error, wave, waveRunner, waveTransport, waveFile }: Props = $props();

  let sessionDoc = $state<SessionDoc | null>(null);
  $effect(() =>
    session.subscribe((d) => {
      sessionDoc = d;
    }),
  );
  let waveDoc = $state<SessionDoc | null>(null);
  $effect(() =>
    wave?.subscribe((d) => {
      waveDoc = d;
    }),
  );
  let waveHasFile = $state(false);
  $effect(() =>
    waveFile?.subscribe((f) => {
      waveHasFile = f !== null && f !== undefined;
    }),
  );
  let ed = $state<EditorState>(editor.current());
  $effect(() =>
    editor.subscribe((s) => {
      ed = s;
    }),
  );

  // The Waveform editor wins while it is in front with a file open.
  const onWave = $derived(ed.mode === "waveform" && !!wave && !!waveDoc && waveHasFile);
  const doc = $derived(onWave ? waveDoc : sessionDoc);
  const store = $derived(onWave && wave ? wave : session);
  const run = $derived(onWave && waveRunner ? waveRunner : runner);
  const tp = $derived(onWave && waveTransport ? waveTransport : transport);

  let renaming = $state<string | null>(null);
  const rate = $derived(doc?.sample_rate ?? 48000);
  const t = (samples: number) => formatIn(samples, rate, ed.timeFormat);
  // Kinds that can be chosen now. The others arrive with their phases.
  const KINDS: MarkerKind[] = ["cue", "subclip", "chapter", "ad_slot", "social_clip", "cd_track", "cart_timer"];

  function edit(label: string, m: MarkerDoc, change: Omit<MarkerChange, "id">) {
    void store.edit(label, { op: "set_markers", changes: [{ id: m.id, ...change }] }, run).catch(onError);
  }

  function commitRename(m: MarkerDoc, value: string) {
    renaming = null;
    const v = value.trim();
    if (v && v !== m.name) edit("Rename Marker", m, { name: v });
  }

  /** How many of the picked markers are still in the list. */
  const pickedCount = $derived(doc ? doc.markers.filter((m) => ed.selectedMarkers.has(m.id)).length : 0);
  /** The row a Shift+click run starts from. */
  let anchor: string | null = null;

  /**
   * A click on a row. Plain click picks that marker and jumps there; Ctrl (or Cmd) adds or removes it
   * from the pick; Shift picks every marker from the last plain click to this one.
   */
  function pick(e: MouseEvent, m: MarkerDoc) {
    if (!doc) return;
    if (e.ctrlKey || e.metaKey) {
      const next = new Set(ed.selectedMarkers);
      if (next.has(m.id)) next.delete(m.id);
      else next.add(m.id);
      anchor = m.id;
      editor.selectMarkers(next);
      return;
    }
    if (e.shiftKey && anchor) {
      editor.selectMarkers(markersBetween(doc.markers, anchor, m.id));
      return;
    }
    anchor = m.id;
    editor.selectMarkers([m.id]);
    void tp.seek(m.position);
  }
  /** The columns after the color swatch: name, start, end, duration, type and description. */
  const COLS = [
    { label: "Name", min: 70, def: 170 },
    { label: "Start", min: 60, def: 84 },
    { label: "End", min: 60, def: 84 },
    { label: "Duration", min: 52, def: 76 },
    { label: "Type", min: 64, def: 96 },
    { label: "Description", min: 80, def: 240 },
  ] as const;
  const KEY_COLS = "markers.columns";
  function loadWidths(): number[] {
    try {
      const raw: unknown = JSON.parse(localStorage.getItem(KEY_COLS) ?? "null");
      if (Array.isArray(raw) && raw.length === COLS.length && raw.every((n) => typeof n === "number" && Number.isFinite(n))) {
        return COLS.map((c, i) => Math.min(900, Math.max(c.min, raw[i] as number)));
      }
    } catch {
      // Storage can be missing or blocked; the standard widths do.
    }
    return COLS.map((c) => c.def);
  }
  let widths = $state<number[]>(loadWidths());
  const template = $derived(`10px ${widths.map((w) => `${w}px`).join(" ")} 18px`);

  /** Drag the right edge of heading `i` to resize that column. */
  function resize(e: PointerEvent, i: number) {
    e.preventDefault();
    e.stopPropagation();
    const startX = e.clientX;
    const start = widths[i] ?? COLS[i]!.def;
    const move = (ev: PointerEvent) => {
      widths[i] = Math.min(900, Math.max(COLS[i]!.min, Math.round(start + ev.clientX - startX)));
    };
    const up = () => {
      window.removeEventListener("pointermove", move, true);
      window.removeEventListener("pointerup", up, true);
      try {
        localStorage.setItem(KEY_COLS, JSON.stringify(widths));
      } catch {
        // Not remembered this time.
      }
    };
    window.addEventListener("pointermove", move, true);
    window.addEventListener("pointerup", up, true);
  }

  let query = $state("");
  const shown = $derived(doc ? filterMarkers(doc.markers, query) : []);

  // A marker picked somewhere else (the ruler, say) scrolls into view here.
  let tableEl = $state<HTMLDivElement | null>(null);
  $effect(() => {
    const picked = ed.selectedMarkers;
    if (picked.size !== 1 || !tableEl) return;
    const [id] = [...picked];
    const row = [...tableEl.querySelectorAll<HTMLElement>("[data-marker]")].find((r) => r.dataset.marker === id);
    row?.scrollIntoView({ block: "nearest" });
  });

  let menu = $state<{ x: number; y: number; items: ContextItem[] } | null>(null);
  function openMenu(e: MouseEvent, m: MarkerDoc) {
    e.preventDefault();
    e.stopPropagation();
    if (!doc) return;
    if (!ed.selectedMarkers.has(m.id)) editor.selectMarkers([m.id]);
    const items = markerMenuItems(m, {
      rate,
      selection: doc.view.selection,
      playlist: doc.playlist,
      goTo: (x) => void tp.seek(x.position),
      rename: (x) => (renaming = x.id),
      change: (label, x, change) => edit(label, x, change),
      setPlaylist: (ids) => void store.edit("Edit Playlist", { op: "set_playlist", ids }, run).catch(onError),
      remove: (x) => void store.edit("Delete Marker", { op: "remove_markers", ids: [x.id] }, run).catch(onError),
    });
    menu = { x: e.clientX, y: e.clientY, items };
  }
</script>

<section class="markers" aria-label="Markers">
  <div class="toolbar">
    <button type="button" onclick={() => void actions.run("edit.marker.cue")}>+ Cue</button>
    <button type="button" disabled={!doc?.view.selection} title="Needs a time selection" onclick={() => void actions.run("edit.marker.range")}>+ Range</button>
    <button type="button" disabled={pickedCount === 0} title="Delete the picked markers. Click a row to pick one; Ctrl+click adds, Shift+click picks a run." onclick={() => void actions.run("edit.marker.delete")}>Delete</button>
    <button type="button" disabled={!doc || doc.markers.length === 0} title="Delete every marker. Undo brings them back." onclick={() => void actions.run("edit.marker.delete-all")}>Delete All</button>
    <input class="search" type="search" placeholder="Search" aria-label="Search markers" bind:value={query} onkeydown={(e) => e.stopPropagation()} />
  </div>
  {#if !doc || doc.markers.length === 0}
    <p class="empty">No markers yet. Add one at the playhead with Edit &gt; Markers, or right-click the ruler.</p>
  {:else}
    <div class="table" role="table" bind:this={tableEl} style:--cols={template}>
      <div class="row head" role="row">
        <span></span>
        {#each COLS as c, i (c.label)}
          <span role="columnheader" class="th" class:num-h={i === 1 || i === 2 || i === 3}>
            {c.label}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span class="grip" title="Drag to resize" onpointerdown={(e) => resize(e, i)}></span>
          </span>
        {/each}
        <span></span>
      </div>
      {#each shown as m (m.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div class="row" class:picked={ed.selectedMarkers.has(m.id)} role="row" tabindex="-1" data-marker={m.id} onclick={(e) => pick(e, m)} oncontextmenu={(e) => openMenu(e, m)}>
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
          <label class="swatch" title={m.color ? "Change color (right-click to go back to the type's color)" : "Pick a color for this marker"} style:background={markerCss(m)} onclick={(e) => e.stopPropagation()} oncontextmenu={(e) => { e.preventDefault(); if (m.color) edit("Marker Color", m, { color: "" }); }}>
            <input type="color" value={m.color ?? "#c8b040"} aria-label="Marker color" onchange={(e) => edit("Marker Color", m, { color: e.currentTarget.value })} />
          </label>
          {#if renaming === m.id}
            <!-- svelte-ignore a11y_autofocus -->
            <input
              type="text"
              value={m.name}
              autofocus
              onclick={(e) => e.stopPropagation()}
              onblur={(e) => commitRename(m, e.currentTarget.value)}
              onkeydown={(e) => {
                e.stopPropagation();
                if (e.key === "Enter") commitRename(m, e.currentTarget.value);
                if (e.key === "Escape") renaming = null;
              }}
            />
          {:else}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <span class="name" title="Double-click to rename" ondblclick={() => (renaming = m.id)}>
              {m.name}
            </span>
          {/if}
          <span class="num">{t(m.position)}</span>
          <span class="num">{m.duration > 0 ? t(markerEnd(m)) : "-"}</span>
          <span class="num dur" class:off={m.kind === "ad_slot" && m.contracted_duration !== null && Math.abs(m.duration - m.contracted_duration) > 5 * rate} title={m.kind === "ad_slot" && m.contracted_duration !== null ? `Contracted ${t(m.contracted_duration)}. Off by more than 5 seconds shows red.` : ""}>{m.duration > 0 ? t(m.duration) : "-"}</span>
          <select
            value={m.kind}
            onclick={(e) => e.stopPropagation()}
            onchange={(e) => {
              const kind = e.currentTarget.value as MarkerKind;
              edit("Marker Type", m, kindChange(m, kind, rate));
            }}
          >
            {#each KINDS.includes(m.kind) ? KINDS : [...KINDS, m.kind] as k (k)}
              <option value={k}>{KIND_LABELS[k]}</option>
            {/each}
          </select>
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div class="desc" onclick={(e) => e.stopPropagation()}>
            <input
              type="text"
              placeholder="Description"
              aria-label="Marker description"
              value={m.description}
              onkeydown={(e) => {
                e.stopPropagation();
                if (e.key === "Enter") e.currentTarget.blur();
              }}
              onblur={(e) => {
                const v = e.currentTarget.value;
                if (v !== m.description) edit("Marker Description", m, { description: v });
              }}
            />
          </div>
          <button
            type="button"
            class="del"
            title="Delete marker"
            aria-label="Delete marker"
            onclick={(e) => {
              e.stopPropagation();
              void store.edit("Delete Marker", { op: "remove_markers", ids: [m.id] }, run).catch(onError);
            }}>×</button
          >
          {#if m.kind === "chapter"}
            <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
            <div class="notes" onclick={(e) => e.stopPropagation()}>
              <input
                type="text"
                placeholder="Link (optional; MP3 files carry it, chapters.json always does)"
                aria-label="Chapter link"
                value={chapterUrl(m)}
                onkeydown={(e) => {
                  e.stopPropagation();
                  if (e.key === "Enter") e.currentTarget.blur();
                }}
                onblur={(e) => {
                  const v = e.currentTarget.value.trim();
                  if (v === chapterUrl(m)) return;
                  if (v !== "" && !isChapterUrl(v)) {
                    onError?.(new Error("A chapter link starts with http:// or https:// and has no spaces."));
                    e.currentTarget.value = chapterUrl(m);
                    return;
                  }
                  edit("Chapter Link", m, { url: v });
                }}
              />
            </div>
            {#each titleTips(m.name) as tip (tip)}
              <div class="notes tip">{tip}</div>
            {/each}
          {/if}
        </div>
      {/each}
      {#if shown.length === 0}
        <p class="empty">No markers match.</p>
      {/if}
    </div>
  {/if}
  {#if menu}
    <ContextMenu x={menu.x} y={menu.y} items={menu.items} onClose={() => (menu = null)} />
  {/if}
</section>

<style>
  .markers {
    container-type: inline-size;
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    height: 100%;
    background: var(--surface-1);
  }

  .toolbar {
    display: flex;
    gap: 4px;
    padding: 4px 6px;
    border-bottom: 1px solid var(--border);
  }

  .toolbar button {
    height: 20px;
    padding: 0 8px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
    font-size: var(--font-size-small);
  }

  .toolbar button:disabled {
    color: var(--text-faint);
  }

  .empty {
    margin: 0;
    padding: 16px;
    color: var(--text-faint);
    text-align: center;
  }

  .table {
    overflow: auto;
  }

  .search {
    flex: 1 1 80px;
    min-width: 60px;
    max-width: 220px;
    margin-left: auto;
    height: 20px;
    padding: 0 6px;
  }

  .row {
    display: grid;
    grid-template-columns: var(--cols);
    width: max-content;
    min-width: 100%;
    gap: 2px 6px;
    align-items: center;
    min-height: 22px;
    padding: 0 6px;
    border-bottom: 1px solid var(--surface-0);
    font-size: var(--font-size-small);
    cursor: pointer;
  }

  .row:hover:not(.head) {
    background: var(--surface-2);
  }

  .row.picked,
  .row.picked:hover:not(.head) {
    background: var(--accent-dim);
  }

  .head {
    position: sticky;
    top: 0;
    background: var(--surface-2);
    color: var(--text-dim);
    cursor: default;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .swatch {
    position: relative;
    width: 10px;
    height: 10px;
    border-radius: 2px;
    cursor: pointer;
  }

  .swatch input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
  }

  .dur.off {
    color: var(--danger);
  }

  .th {
    position: relative;
    overflow: hidden;
    white-space: nowrap;
  }

  .num-h {
    text-align: right;
  }

  .grip {
    position: absolute;
    top: 0;
    right: -3px;
    bottom: 0;
    width: 7px;
    cursor: col-resize;
  }

  .grip:hover {
    background: var(--border-strong);
  }

  .desc input {
    width: 100%;
    padding: 0 4px;
  }

  .notes {
    grid-column: 2 / -1;
    padding-bottom: 3px;
  }

  .tip {
    padding: 0 4px 3px;
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  .notes input {
    width: 100%;
    height: 18px;
    padding: 0 4px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-dim);
    font: inherit;
  }

  .notes input:hover,
  .notes input:focus {
    border-color: var(--border);
    background: var(--surface-0);
    color: var(--text);
  }

  .num {
    color: var(--value);
    font-family: var(--font-mono);
    text-align: right;
  }

  input,
  select {
    height: 18px;
    min-width: 0;
    border: 1px solid var(--border-strong);
    background: var(--surface-0);
    color: var(--text);
    font: inherit;
  }

  .del {
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--text-dim);
    font-size: 14px;
  }

  .del:hover {
    color: var(--danger);
  }
</style>
