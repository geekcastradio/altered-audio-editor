<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Multitrack > Find Isolated Short Clips: lists stray bumps, coughs and breaths that
  survived QuickPass as tiny clips on their own. Nothing is removed until you check
  it and say so.
-->
<script lang="ts">
  import type { IsolatedClip } from "./isolated-clips";

  interface Props {
    rate: number;
    find: (maxSeconds: number, gapSeconds: number) => IsolatedClip[];
    onPlay: (c: IsolatedClip) => void;
    onRemove: (ids: string[]) => Promise<void>;
    onClose: () => void;
  }

  const { rate, find, onPlay, onRemove, onClose }: Props = $props();

  let maxSeconds = $state(0.8);
  let gapSeconds = $state(2);
  let items = $state<IsolatedClip[]>([]);
  let checked = $state<Set<string>>(new Set());
  let searched = $state(false);
  let busy = $state(false);

  function run() {
    items = find(maxSeconds, gapSeconds);
    checked = new Set();
    searched = true;
  }

  function toggle(id: string) {
    const next = new Set(checked);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    checked = next;
  }

  async function remove() {
    busy = true;
    try {
      await onRemove([...checked]);
      run();
    } finally {
      busy = false;
    }
  }

  function time(samples: number): string {
    const s = samples / rate;
    const m = Math.floor(s / 60);
    return `${m}:${(s - m * 60).toFixed(1).padStart(4, "0")}`;
  }

  const length = (c: IsolatedClip) => `${((c.end - c.start) / rate).toFixed(2)} s`;
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={() => !busy && onClose()}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="iso-title" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <h1 id="iso-title">Find Isolated Short Clips</h1>
    <p class="note">Finds very short clips with nothing else near them on their own track, the stray bumps and breaths QuickPass can leave behind. Play each one, check the ones to remove, then remove them. Nothing is checked for you.</p>
    <label class="row">
      <span>Shorter than (s)</span>
      <input type="number" min="0.1" max="3" step="0.1" bind:value={maxSeconds} disabled={busy} />
    </label>
    <label class="row">
      <span>Empty around it (s)</span>
      <input type="number" min="0.5" max="30" step="0.5" bind:value={gapSeconds} disabled={busy} />
    </label>
    {#if searched && items.length === 0}
      <p class="advice">No isolated short clips found. Nothing to clean up.</p>
    {/if}
    {#if items.length}
      <ul class="list">
        {#each items as c (c.id)}
          <li>
            <label>
              <input type="checkbox" checked={checked.has(c.id)} onchange={() => toggle(c.id)} disabled={busy} />
              <span class="where">{c.trackName} at {time(c.start)}</span>
              <span class="len">{length(c)}</span>
              {#if c.overlapsOthers}<span class="tag" title="Another track has audio at the same time, so this may be bleed or a quick reply.">under another track</span>{/if}
            </label>
            <button type="button" onclick={() => onPlay(c)} disabled={busy}>Play</button>
          </li>
        {/each}
      </ul>
    {/if}
    <footer>
      <button type="button" onclick={run} disabled={busy}>{searched ? "Search Again" : "Search"}</button>
      <button type="button" onclick={() => void remove()} disabled={busy || checked.size === 0}>Remove Checked ({checked.size})</button>
      <button type="button" class="primary" onclick={onClose} disabled={busy}>Close</button>
    </footer>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 5000;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.5);
  }

  .dialog {
    display: grid;
    gap: 10px;
    width: min(520px, 92vw);
    padding: 14px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  }

  h1 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }

  .row {
    display: grid;
    grid-template-columns: 140px 80px;
    gap: 8px;
    align-items: center;
    color: var(--text-dim);
  }

  input[type="number"] {
    height: 24px;
    padding: 0 6px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-1);
    color: var(--text);
    font: inherit;
  }

  .note {
    margin: 0;
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  .advice {
    margin: 0;
    padding: 6px 8px;
    border-left: 2px solid var(--accent);
    background: var(--surface-1);
    color: var(--text);
    font-size: var(--font-size-small);
  }

  .list {
    display: grid;
    gap: 2px;
    max-height: 260px;
    margin: 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
    border: 1px solid var(--border-strong);
    background: var(--surface-1);
  }

  li {
    display: flex;
    gap: 8px;
    align-items: center;
    justify-content: space-between;
    padding: 3px 6px;
  }

  li label {
    display: flex;
    flex: 1;
    gap: 8px;
    align-items: center;
    min-width: 0;
  }

  .where {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .len {
    color: var(--value);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
  }

  .tag {
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  footer {
    display: flex;
    gap: 6px;
    justify-content: flex-end;
    align-items: center;
  }

  button {
    min-width: 72px;
    height: 22px;
    padding: 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
  }

  button:hover:not(:disabled) {
    border-color: var(--accent);
  }

  button:disabled {
    opacity: 0.5;
  }

  button.primary {
    border-color: var(--accent);
  }
</style>
