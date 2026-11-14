<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Effects > Match EQ: measures two tracks and builds a Parametric EQ that
  moves one voice's tone toward the other's.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { MAX_GAIN_DB, VOICE_TARGET_OPTS, type MatchFit } from "./match-eq";

  interface Props {
    tracks: { id: string; name: string }[];
    /** The track picked in the editor, offered as the one to change. */
    initialSource: string | null;
    scope: string;
    /** A null reference means the built-in neutral voice target. */
    onMatch: (sourceId: string, referenceId: string | null) => Promise<MatchFit | string>;
    onApply: (sourceId: string, fit: MatchFit) => void;
    onClose: () => void;
  }

  const { tracks, initialSource, scope, onMatch, onApply, onClose }: Props = $props();

  // The dialog is mounted fresh each time it opens, so the starting picks are read once on purpose.
  // svelte-ignore state_referenced_locally
  let source = $state(initialSource ?? tracks[0]?.id ?? "");
  // svelte-ignore state_referenced_locally
  let reference = $state(tracks.find((t) => t.id !== (initialSource ?? tracks[0]?.id))?.id ?? "");
  let mode = $state<"track" | "voice">("track");
  let busy = $state(false);
  let fit = $state<MatchFit | null>(null);
  let message = $state("");

  const same = $derived(mode === "track" && source === reference);
  const cap = $derived(mode === "voice" ? VOICE_TARGET_OPTS.capDb : MAX_GAIN_DB);

  async function run() {
    busy = true;
    message = "";
    fit = null;
    const r = await onMatch(source, mode === "voice" ? null : reference);
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

<Dialog title="Match EQ" canClose={!busy} labelWidth={120} {onClose}>
  <p class="note">
    {#if mode === "voice"}
      Nudges a voice toward a neutral speech tone. It measures the track over {scope}, then builds a gentle Parametric EQ (no band moves more than {VOICE_TARGET_OPTS.capDb} dB). The target is a starting point: trust your ears over the numbers.
    {:else}
      Makes one voice sound more like another. It measures both tracks over {scope}, then builds a Parametric EQ for the track you want to change. The volume stays the same; only the tone moves.
    {/if}
  </p>
  <label class="row">
    <span>Match to</span>
    <select bind:value={mode} disabled={busy} onchange={() => (fit = null)}>
      <option value="track">Another track</option>
      <option value="voice">Neutral voice target</option>
    </select>
  </label>
  <label class="row">
    <span>Change this track</span>
    <select bind:value={source} disabled={busy}>
      {#each tracks as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
    </select>
  </label>
  {#if mode === "track"}
    <label class="row">
      <span>To sound like</span>
      <select bind:value={reference} disabled={busy}>
        {#each tracks as t (t.id)}<option value={t.id}>{t.name}</option>{/each}
      </select>
    </label>
  {/if}
  {#if same}<p class="note">Pick two different tracks.</p>{/if}
  {#if message}<p class="advice">{message}</p>{/if}
  {#if fit}
    <p class="advice">
      {#if lines.length === 0}
        {mode === "voice" ? "This voice is already close to the target. No EQ is needed." : "These two already sound alike. No EQ is needed."}
      {:else}
        The tone differed by {fit.errorBefore.toFixed(1)} dB on average; this EQ brings it to {fit.errorAfter.toFixed(1)} dB (limit {cap} dB per band).
      {/if}
    </p>
    {#if lines.length}
      <ul>{#each lines as l (l)}<li>{l}</li>{/each}</ul>
      <p class="note">Hear it before you trust it: the numbers match the average sound, not every word. Fine-tune it in the Parametric EQ afterward.</p>
    {/if}
  {/if}
  {#snippet footer()}
    {#if busy}
      <span class="note">Measuring...</span>
    {:else}
      <button type="button" onclick={onClose}>Close</button>
      <button type="button" class:primary={!fit} onclick={() => void run()} disabled={same || !source || (mode === "track" && !reference)}>{fit ? "Measure Again" : "Match"}</button>
      {#if fit && fit.errorAfter < fit.errorBefore}
        <button type="button" class="primary" onclick={() => fit && onApply(source, fit)}>Add EQ to Track</button>
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
  ul {
    margin: 0;
    padding-left: 18px;
    color: var(--value);
    font-family: var(--font-mono);
    font-size: var(--font-size-small);
  }
</style>
