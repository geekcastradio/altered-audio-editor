<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Multitrack > Reduce Bleed: takes one mic's sound out of another mic's track, for recordings
  where each microphone also hears the other speakers.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";

  interface Props {
    tracks: { id: string; name: string }[];
    initialTarget: string | null;
    onRun: (targetId: string, referenceId: string) => Promise<string>;
    onClose: () => void;
  }

  const { tracks, initialTarget, onRun, onClose }: Props = $props();

  // The dialog is mounted fresh each time it opens, so the starting picks are read once on purpose.
  // svelte-ignore state_referenced_locally
  let target = $state(tracks.find((t) => t.id === initialTarget)?.id ?? tracks[0]?.id ?? "");
  // svelte-ignore state_referenced_locally
  let reference = $state(tracks.find((t) => t.id !== (tracks.find((x) => x.id === initialTarget)?.id ?? tracks[0]?.id))?.id ?? "");
  let busy = $state(false);
  let message = $state("");

  const same = $derived(target === reference);

  async function run() {
    busy = true;
    message = "";
    try {
      message = await onRun(target, reference);
    } finally {
      busy = false;
    }
  }
</script>

<Dialog title="Reduce Bleed" width={500} labelWidth={170} canClose={!busy} {onClose}>
  <p class="note">For recordings where each mic also hears the other speakers. It learns how the other mic's sound reaches this one and takes that out, keeping the voice on this mic. Run it before Auto Mic Mix. Each clip is cleaned into a copy; your original files are not changed, and one undo puts them back. The two tracks should be in step: use Sync Tracks first if they are not.</p>
  <label class="row">
    <span>Clean this mic</span>
    <select bind:value={target} disabled={busy}>
      {#each tracks as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
    </select>
  </label>
  <label class="row">
    <span>Taking out the sound of</span>
    <select bind:value={reference} disabled={busy}>
      {#each tracks as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
    </select>
  </label>
  {#if same}<p class="note">Pick two different tracks.</p>{/if}
  {#if message}<p class="advice">{message}</p>{/if}
  {#snippet footer()}
    {#if busy}
      <span class="note">Cleaning...</span>
    {:else}
      <button type="button" onclick={onClose}>Close</button>
      <button type="button" class="primary" onclick={() => void run()} disabled={same || !target || !reference}>Reduce Bleed</button>
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
