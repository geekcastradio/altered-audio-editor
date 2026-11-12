<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Multitrack > Auto Mic Mix: picks the mic tracks, measures who is talking,
  shows each voice's talk time, and writes volume lanes that turn a mic down
  while someone else speaks.
-->
<script lang="ts">
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

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={() => !busy && onClose()}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="mm-title" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <h1 id="mm-title">Auto Mic Mix</h1>
    <p class="note">For a recording with one mic per person. It finds who is talking and turns the other mics down, so room sound and bleed stop stacking up. Each mic is judged against its own quiet level. When nobody talks, everything stays open.</p>
    <fieldset disabled={busy}>
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
    <footer>
      {#if busy}
        <span class="note">Listening to every track...</span>
      {:else}
        {#if result}
          <button type="button" onclick={() => result && onApply(result)}>Write Volume Lanes</button>
        {/if}
        <button type="button" onclick={() => void run()} disabled={!ready}>{result ? "Measure Again" : "Measure"}</button>
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
    width: min(480px, 92vw);
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

  fieldset {
    display: grid;
    gap: 4px;
    margin: 0;
    padding: 6px 8px;
    border: 1px solid var(--border-strong);
    color: var(--text-dim);
    max-height: 160px;
    overflow-y: auto;
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

  ul {
    margin: 0;
    padding-left: 18px;
    color: var(--value);
    font-family: var(--font-mono);
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
