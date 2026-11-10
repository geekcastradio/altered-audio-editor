<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Review panel: the marks dropped while recording (Cut this, Retake, Great
  bit) in time order, with Previous and Next to step through them. Click a
  row to jump there. Follows the active editor like the Markers panel: the
  Waveform editor's file when it is in front, the session otherwise.
-->
<script lang="ts">
  import type { Readable } from "svelte/store";
  import type { ActionRegistry } from "../../app/actions";
  import type { CommandRunner } from "../../stores/media-pool";
  import { editor, type EditorState } from "../../stores/editor";
  import type { MarkerDoc, SessionDoc, SessionStore } from "../../stores/session";
  import type { TransportState, TransportStore } from "../../stores/transport";
  import { formatIn } from "../editor/timeline";
  import { markerCss } from "../markers/markers";
  import { REVIEW_LABELS, reviewCounts, reviewKind, reviewMarks, stepMark, type ReviewFilter } from "./review";

  interface Props {
    session: SessionStore;
    transport: TransportStore;
    runner: CommandRunner;
    actions?: ActionRegistry;
    onError?: (e: unknown) => void;
    wave?: SessionStore;
    waveRunner?: CommandRunner;
    waveTransport?: TransportStore;
    waveFile?: Readable<unknown>;
  }

  const { session, transport, runner, onError = console.error, wave, waveRunner, waveTransport, waveFile }: Props = $props();

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

  const onWave = $derived(ed.mode === "waveform" && !!wave && !!waveDoc && waveHasFile);
  const doc = $derived(onWave ? waveDoc : sessionDoc);
  const store = $derived(onWave && wave ? wave : session);
  const run = $derived(onWave && waveRunner ? waveRunner : runner);
  const tp = $derived(onWave && waveTransport ? waveTransport : transport);

  let tpState = $state<TransportState | null>(null);
  $effect(() =>
    tp.subscribe((s) => {
      tpState = s;
    }),
  );

  let filter = $state<ReviewFilter>("all");
  const rate = $derived(doc?.sample_rate ?? 48000);
  const counts = $derived(doc ? reviewCounts(doc.markers) : { cut: 0, retake: 0, great: 0 });
  const total = $derived(counts.cut + counts.retake + counts.great);
  const marks = $derived(doc ? reviewMarks(doc.markers, filter) : []);
  const playhead = $derived(tpState?.playing ? tpState.position : (doc?.view.playhead ?? 0));
  const prev = $derived(stepMark(marks, playhead, -1));
  const next = $derived(stepMark(marks, playhead, 1));
  const t = (samples: number) => formatIn(samples, rate, ed.timeFormat);

  function go(m: MarkerDoc | null) {
    if (!m) return;
    editor.selectMarkers([m.id]);
    void tp.seek(m.position);
  }

  function remove(m: MarkerDoc) {
    void store.edit("Delete Marker", { op: "remove_markers", ids: [m.id] }, run).catch(onError);
  }
</script>

<section class="review" aria-label="Review">
  <div class="toolbar">
    <button type="button" disabled={!prev} title="Jump to the previous mark" onclick={() => go(prev)}>&lsaquo; Prev</button>
    <button type="button" disabled={!next} title="Jump to the next mark" onclick={() => go(next)}>Next &rsaquo;</button>
    <select bind:value={filter} aria-label="Show">
      <option value="all">All ({total})</option>
      <option value="cut">{REVIEW_LABELS.cut} ({counts.cut})</option>
      <option value="retake">{REVIEW_LABELS.retake} ({counts.retake})</option>
      <option value="great">{REVIEW_LABELS.great} ({counts.great})</option>
    </select>
  </div>
  {#if total === 0}
    <p class="empty">
      No marks yet. While you record, drop them with Edit &gt; Markers: Add Cut This Marker (Shift+F9), Add Retake Marker (F9) or Add Great Bit Marker (Ctrl+F9).
    </p>
  {:else if marks.length === 0}
    <p class="empty">No marks of this kind.</p>
  {:else}
    <div class="list" role="list">
      {#each marks as m (m.id)}
        {@const k = reviewKind(m)}
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
        <div class="row" class:picked={ed.selectedMarkers.has(m.id)} role="listitem" onclick={() => go(m)}>
          <span class="swatch" style:background={markerCss(m)}></span>
          <span class="kind">{k ? REVIEW_LABELS[k] : ""}</span>
          <span class="name" title={m.name}>{m.name}</span>
          <span class="num">{t(m.position)}</span>
          <button
            type="button"
            class="del"
            title="Delete mark"
            aria-label="Delete mark"
            onclick={(e) => {
              e.stopPropagation();
              remove(m);
            }}>×</button
          >
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .review {
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

  .toolbar button,
  .toolbar select {
    height: 20px;
    padding: 0 8px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
    font-size: var(--font-size-small);
  }

  .toolbar select {
    margin-left: auto;
    padding: 0 4px;
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

  .list {
    overflow: auto;
  }

  .row {
    display: grid;
    grid-template-columns: 10px 70px minmax(60px, 1fr) 80px 18px;
    gap: 6px;
    align-items: center;
    min-height: 22px;
    padding: 0 6px;
    border-bottom: 1px solid var(--surface-0);
    font-size: var(--font-size-small);
    cursor: pointer;
  }

  .row:hover {
    background: var(--surface-2);
  }

  .row.picked,
  .row.picked:hover {
    background: var(--accent-dim);
  }

  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }

  .kind {
    color: var(--text-dim);
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .num {
    color: var(--value);
    font-family: var(--font-mono);
    text-align: right;
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
