<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Effects > Match EQ: measures two tracks and builds a Parametric EQ that
  moves one voice's tone toward the other's.
-->
<script lang="ts">
  import { MAX_GAIN_DB, type MatchFit } from "./match-eq";

  interface Props {
    tracks: { id: string; name: string }[];
    /** The track picked in the editor, offered as the one to change. */
    initialSource: string | null;
    scope: string;
    onMatch: (sourceId: string, referenceId: string) => Promise<MatchFit | string>;
    onApply: (sourceId: string, fit: MatchFit) => void;
    onClose: () => void;
  }

  const { tracks, initialSource, scope, onMatch, onApply, onClose }: Props = $props();

  // The dialog is mounted fresh each time it opens, so the starting picks are read once on purpose.
  // svelte-ignore state_referenced_locally
  let source = $state(initialSource ?? tracks[0]?.id ?? "");
  // svelte-ignore state_referenced_locally
  let reference = $state(tracks.find((t) => t.id !== (initialSource ?? tracks[0]?.id))?.id ?? "");
  let busy = $state(false);
  let fit = $state<MatchFit | null>(null);
  let message = $state("");

  const same = $derived(source === reference);

  async function run() {
    busy = true;
    message = "";
    fit = null;
    const r = await onMatch(source, reference);
    busy = false;
    if (typeof r === "string") message = r;
    else fit = r;
  }

  const hz = (f: number) => (f >= 1000 ? `${(f / 1000).toFixed(1)} kHz` : `${Math.round(f)} Hz`);
  const sign = (v: number) => `${v > 0 ? "+" : ""}${v.toFixed(1)} dB`;

  /** The EQ's bands in plain words, left out when they do nothing. */
  const lines = $derived.by(() => {
    if (!fit) return [];
    const p = fit.params;
    const out: string[] = [];
    if (p.ls_gain) out.push(`Low shelf ${sign(p.ls_gain)} below ${hz(p.ls_freq ?? 150)}`);
    for (const n of [1, 2, 3]) if (p[`b${n}_gain`]) out.push(`Band ${n}: ${sign(p[`b${n}_gain`]!)} at ${hz(p[`b${n}_freq`] ?? 1000)}`);
    if (p.hs_gain) out.push(`High shelf ${sign(p.hs_gain)} above ${hz(p.hs_freq ?? 7000)}`);
    return out;
  });
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={() => !busy && onClose()}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="meq-title" tabindex="-1" onclick={(e) => e.stopPropagation()}>
    <h1 id="meq-title">Match EQ</h1>
    <p class="note">Makes one voice sound more like another. It measures both tracks over {scope}, then builds a Parametric EQ for the track you want to change. The volume stays the same; only the tone moves.</p>
    <label class="row">
      <span>Change this track</span>
      <select bind:value={source} disabled={busy}>
        {#each tracks as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
      </select>
    </label>
    <label class="row">
      <span>To sound like</span>
      <select bind:value={reference} disabled={busy}>
        {#each tracks as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
      </select>
    </label>
    {#if same}<p class="note">Pick two different tracks.</p>{/if}
    {#if message}<p class="advice">{message}</p>{/if}
    {#if fit}
      <p class="advice">
        {#if lines.length === 0}
          These two already sound alike. No EQ is needed.
        {:else}
          The tone differed by {fit.errorBefore.toFixed(1)} dB on average; this EQ brings it to {fit.errorAfter.toFixed(1)} dB (limit {MAX_GAIN_DB} dB per band).
        {/if}
      </p>
      {#if lines.length}
        <ul>{#each lines as l (l)}<li>{l}</li>{/each}</ul>
        <p class="note">Hear it before you trust it: the numbers match the average sound, not every word. Fine-tune it in the Parametric EQ afterward.</p>
      {/if}
    {/if}
    <footer>
      {#if busy}
        <span class="note">Measuring...</span>
      {:else}
        {#if fit && fit.errorAfter < fit.errorBefore}
          <button type="button" onclick={() => fit && onApply(source, fit)}>Add EQ to Track</button>
        {/if}
        <button type="button" onclick={() => void run()} disabled={same || !source || !reference}>{fit ? "Measure Again" : "Match"}</button>
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
