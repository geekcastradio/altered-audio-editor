<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Multitrack > Auto Mic Mix: picks the mic tracks, measures who is talking,
  shows each voice's talk time, and writes volume lanes that turn a mic down
  while someone else speaks.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { REDUCE_CHOICES, describeTalk, type MixAnalysis, type MixOptions } from "./multitrack";

  interface Props {
    tracks: { id: string; name: string }[];
    onAnalyze: (ids: string[], options: MixOptions) => Promise<MixAnalysis | string>;
    onApply: (analysis: MixAnalysis) => void;
    onClose: () => void;
  }

  const { tracks, onAnalyze, onApply, onClose }: Props = $props();

  // Mounted fresh each time it opens, so every mic starts ticked on purpose.
  // svelte-ignore state_referenced_locally
  let picked = $state<string[]>(tracks.map((t) => t.id));
  let reduce = $state("standard");
  let busy = $state(false);
  let result = $state<MixAnalysis | null>(null);
  let message = $state("");

  const ready = $derived(picked.length >= 2);

  function toggle(id: string, on: boolean) {
    picked = on ? [...picked, id] : picked.filter((p) => p !== id);
    result = null;
  }

  async function run() {
    busy = true;
    message = "";
    result = null;
    const db = REDUCE_CHOICES.find((c) => c.id === reduce)?.db ?? 12;
    const r = await onAnalyze(picked, { reduceDb: db });
    busy = false;
    if (typeof r === "string") message = r;
    else result = r;
  }

  const lines = $derived(result ? describeTalk(result) : []);
</script>

<Dialog title="Auto Mic Mix" width={480} canClose={!busy} labelWidth={120} {onClose}>
  <p class="note">For a recording with one mic per person. It finds who is talking and turns the other mics down, so room sound and bleed stop stacking up. Each mic is judged against its own quiet level. When nobody talks, everything stays open.</p>
  <fieldset class="mics" disabled={busy}>
    <legend>Mic tracks</legend>
    {#each tracks as t (t.id)}
      <label class="check">
        <input type="checkbox" checked={picked.includes(t.id)} onchange={(e) => toggle(t.id, e.currentTarget.checked)} />
        <span>{t.name}</span>
      </label>
    {/each}
  </fieldset>
  <label class="row">
    <span>Turn down by</span>
    <select bind:value={reduce} disabled={busy} onchange={() => (result = null)}>
      {#each REDUCE_CHOICES as c (c.id)}<option value={c.id}>{c.label}</option>{/each}
    </select>
  </label>
  {#if !ready}<p class="note">Tick at least two tracks.</p>{/if}
  {#if message}<p class="advice">{message}</p>{/if}
  {#if result}
    <p class="advice">Who talked, from most to least:</p>
    <ul>{#each lines as l (l)}<li>{l}</li>{/each}</ul>
    <p class="note">This is by level, so laughing and crosstalk count as talking. Listen before you trust it. Writing the mix replaces each picked track's volume lane, and one undo restores them all.</p>
  {/if}
  {#snippet footer()}
    {#if busy}
      <span class="note">Listening to every track...</span>
    {:else}
      <button type="button" onclick={onClose}>Close</button>
      <button type="button" class:primary={!result} onclick={() => void run()} disabled={!ready}>{result ? "Measure Again" : "Measure"}</button>
      {#if result}
        <button type="button" class="primary" onclick={() => result && onApply(result)}>Write Volume Lanes</button>
      {/if}
    {/if}
  {/snippet}
</Dialog>

<style>
  .mics {
    max-height: 160px;
    overflow-y: auto;
    color: var(--text-dim);
  }
  legend {
    padding: 0 4px;
  }
  .check {
    display: flex;
    gap: 6px;
    align-items: center;
    color: var(--text);
  }
  .advice {
    margin: 0;
    padding: 6px 8px;
    border-left: 2px solid var(--accent);
    background: var(--surface-1);
    color: var(--text);
    font-size: var(--font-size-small);
  }
  ul {
    margin: 0;
    padding-left: 18px;
    color: var(--value);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
  }
</style>
