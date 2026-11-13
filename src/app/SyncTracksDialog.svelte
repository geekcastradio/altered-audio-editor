<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Multitrack > Sync Tracks: measures how far one track trails another recording of the
  same conversation, and how fast their clocks drift, then moves the track into line.
-->
<script lang="ts">
  import { describeSync, trust, type TrackSync } from "./sync-tracks";

  interface Props {
    tracks: { id: string; name: string }[];
    rate: number;
    initialSecond: string | null;
    onMeasure: (firstId: string, secondId: string) => Promise<TrackSync | string>;
    onApply: (secondId: string, r: TrackSync) => Promise<void>;
    onClose: () => void;
  }

  const { tracks, rate, initialSecond, onMeasure, onApply, onClose }: Props = $props();

  // The dialog is mounted fresh each time it opens, so the starting picks are read once on purpose.
  // svelte-ignore state_referenced_locally
  let second = $state(initialSecond ?? tracks[1]?.id ?? "");
  // svelte-ignore state_referenced_locally
  let first = $state(tracks.find((t) => t.id !== (initialSecond ?? tracks[1]?.id))?.id ?? "");
  let busy = $state(false);
  let found = $state<TrackSync | null>(null);
  let message = $state("");

  const same = $derived(first === second);
  const lines = $derived(found ? describeSync(found, rate) : []);
  const grade = $derived(found ? trust(found) : null);

  async function run() {
    busy = true;
    message = "";
    found = null;
    const r = await onMeasure(first, second);
    busy = false;
    if (typeof r === "string") message = r;
    else found = r;
  }

  async function apply() {
    if (!found) return;
    busy = true;
    try {
      await onApply(second, found);
      onClose();
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={() => !busy && onClose()}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="sync-title" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <h1 id="sync-title">Sync Tracks</h1>
    <p class="note">Lines up two recordings of the same conversation, such as a call recording and a local mic. The first track stays put and is never changed. Both tracks must have heard some of the same audio, and each needs at least two minutes of it.</p>
    <label class="row">
      <span>Keep in place</span>
      <select bind:value={first} disabled={busy}>
        {#each tracks as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
      </select>
    </label>
    <label class="row">
      <span>Move this one</span>
      <select bind:value={second} disabled={busy}>
        {#each tracks as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
      </select>
    </label>
    {#if same}<p class="note">Pick two different tracks.</p>{/if}
    {#if message}<p class="advice">{message}</p>{/if}
    {#if found}
      <p class="advice">
        {#each lines as l (l)}{l}<br />{/each}
        {#if grade === "weak"}This match is weak. Listen to both tracks together before you trust it.{:else if grade === "fair"}This match is fair.{:else}This is a strong match.{/if}
      </p>
    {/if}
    <footer>
      {#if busy}
        <span class="note">Listening to both tracks...</span>
      {:else}
        {#if found && Math.round(found.offset) !== 0}
          <button type="button" onclick={() => void apply()}>Move Track</button>
        {/if}
        <button type="button" onclick={() => void run()} disabled={same || !first || !second}>{found ? "Measure Again" : "Measure"}</button>
        <button type="button" class="primary" onclick={onClose}>Close</button>
      {/if}
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
    width: min(460px, 92vw);
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
    grid-template-columns: 120px minmax(0, 1fr);
    gap: 8px;
    align-items: center;
    color: var(--text-dim);
  }

  select {
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
