<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Markers panel: every marker in time order. Click a row to jump there,
  double-click the name to rename, change the type from the list, and
  delete with the row's button. Every change is undoable.
-->
<script lang="ts">
  import type { ActionRegistry } from "../../app/actions";
  import type { CommandRunner } from "../../stores/media-pool";
  import { editor, type EditorState } from "../../stores/editor";
  import type { MarkerDoc, MarkerKind, SessionDoc, SessionStore } from "../../stores/session";
  import type { TransportStore } from "../../stores/transport";
  import { formatIn } from "../editor/timeline";
  import { KIND_COLORS, KIND_LABELS } from "./markers";

  interface Props {
    session: SessionStore;
    transport: TransportStore;
    runner: CommandRunner;
    actions: ActionRegistry;
    onError?: (e: unknown) => void;
  }

  const { session, transport, runner, actions, onError = console.error }: Props = $props();

  let doc = $state<SessionDoc | null>(null);
  $effect(() =>
    session.subscribe((d) => {
      doc = d;
    }),
  );
  let ed = $state<EditorState>(editor.current());
  $effect(() =>
    editor.subscribe((s) => {
      ed = s;
    }),
  );

  let renaming = $state<string | null>(null);
  const rate = $derived(doc?.sample_rate ?? 48000);
  const t = (samples: number) => formatIn(samples, rate, ed.timeFormat);
  // Kinds that can be chosen now. The others arrive with their phases.
  const KINDS: MarkerKind[] = ["cue", "subclip"];

  function edit(label: string, m: MarkerDoc, change: Partial<MarkerDoc>) {
    void session.edit(label, { op: "set_markers", changes: [{ id: m.id, ...change }] }, runner).catch(onError);
  }

  function commitRename(m: MarkerDoc, value: string) {
    renaming = null;
    const v = value.trim();
    if (v && v !== m.name) edit("Rename Marker", m, { name: v });
  }
</script>

<section class="markers" aria-label="Markers">
  <div class="toolbar">
    <button type="button" onclick={() => void actions.run("edit.marker.cue")}>+ Cue</button>
    <button type="button" disabled={!doc?.view.selection} title="Needs a time selection" onclick={() => void actions.run("edit.marker.range")}>+ Range</button>
  </div>
  {#if !doc || doc.markers.length === 0}
    <p class="empty">No markers yet. Add one at the playhead with Edit &gt; Markers, or right-click the ruler.</p>
  {:else}
    <div class="table" role="table">
      <div class="row head" role="row">
        <span role="columnheader">Name</span>
        <span role="columnheader">Start</span>
        <span role="columnheader" class="dur">Duration</span>
        <span role="columnheader">Type</span>
        <span></span>
      </div>
      {#each doc.markers as m (m.id)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div class="row" role="row" tabindex="-1" onclick={() => void transport.seek(m.position)}>
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
              <i style:background="var({KIND_COLORS[m.kind]})"></i>{m.name}
            </span>
          {/if}
          <span class="num">{t(m.position)}</span>
          <span class="num dur">{m.duration > 0 ? t(m.duration) : "-"}</span>
          <select
            value={m.kind}
            onclick={(e) => e.stopPropagation()}
            onchange={(e) => edit("Marker Type", m, { kind: e.currentTarget.value as MarkerKind })}
          >
            {#each KINDS.includes(m.kind) ? KINDS : [...KINDS, m.kind] as k (k)}
              <option value={k}>{KIND_LABELS[k]}</option>
            {/each}
          </select>
          <button
            type="button"
            class="del"
            title="Delete marker"
            aria-label="Delete marker"
            onclick={(e) => {
              e.stopPropagation();
              void session.edit("Delete Marker", { op: "remove_markers", ids: [m.id] }, runner).catch(onError);
            }}>×</button
          >
        </div>
      {/each}
    </div>
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

  .row {
    display: grid;
    grid-template-columns: minmax(70px, 1fr) 66px 58px 76px 18px;
    gap: 6px;
    align-items: center;
    height: 22px;
    padding: 0 6px;
    border-bottom: 1px solid var(--surface-0);
    font-size: var(--font-size-small);
    cursor: pointer;
  }

  @container (max-width: 320px) {
    .row {
      grid-template-columns: minmax(60px, 1fr) 62px 70px 18px;
    }

    .dur {
      display: none;
    }
  }

  .row:hover:not(.head) {
    background: var(--surface-2);
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

  .name i {
    display: inline-block;
    width: 8px;
    height: 8px;
    margin-right: 6px;
    border-radius: 1px;
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
