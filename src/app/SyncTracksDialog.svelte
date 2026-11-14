<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Multitrack > Sync Tracks: measures how far one track trails another recording of the
  same conversation, and how fast their clocks drift, then moves the track into line.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
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

<Dialog title="Sync Tracks" canClose={!busy} labelWidth={120} {onClose}>
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
  {#snippet footer()}
    {#if busy}
      <span class="note">Listening to both tracks...</span>
    {:else}
      <button type="button" onclick={onClose}>Close</button>
      <button type="button" class:primary={!found} onclick={() => void run()} disabled={same || !first || !second}>{found ? "Measure Again" : "Measure"}</button>
      {#if found && Math.round(found.offset) !== 0}
        <button type="button" class="primary" onclick={() => void apply()}>Move Track</button>
      {/if}
    {/if}
  {/snippet}
</Dialog>

<style>
  .advice {
    margin: 0;
    padding: 6px 8px;
    border-left: 2px solid var(--accent);
    background: var(--surface-1);
    color: var(--text);
    font-size: var(--font-size-small);
  }
</style>
