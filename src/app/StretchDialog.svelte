<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Clip > Time Stretch: makes a clip play for a set length without
  changing its audio range. Keep pitch for speech and ad reads; change
  pitch with speed for a tape-style effect. Quick buttons fit common ad
  slot lengths, and Fit to Time Selection uses the selected range.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { NATURAL_LIMIT, parseSeconds, percent } from "../panels/editor/stretch";

  interface Props {
    clipName: string;
    /** Seconds at no stretch. */
    natural: number;
    /** Seconds now. */
    current: number;
    keepPitch: boolean;
    /** Seconds of the time selection, when there is one. */
    selection: number | null;
    stretched: boolean;
    onApply: (seconds: number, keepPitch: boolean, toSelection: boolean) => void;
    onReset: () => void;
    onClose: () => void;
  }

  const { clipName, natural, current, keepPitch, selection, stretched, onApply, onReset, onClose }: Props = $props();

  const fmt = (s: number) => {
    const m = Math.floor(s / 60);
    const rest = (s - m * 60).toFixed(3).padStart(6, "0");
    return m > 0 ? `${m}:${rest}` : (s).toFixed(3);
  };
  // svelte-ignore state_referenced_locally
  let text = $state(fmt(current));
  // svelte-ignore state_referenced_locally
  let pitch = $state(keepPitch);
  let pct = $state("");
  const secs = $derived(parseSeconds(text));
  const ratio = $derived(secs ? secs / natural : null);
  const far = $derived(ratio !== null && pitch && Math.abs(ratio - 1) > NATURAL_LIMIT);
  const outOfRange = $derived(ratio !== null && (ratio < 0.25 || ratio > 4));

  function fromPercent() {
    const p = Number(pct);
    if (Number.isFinite(p) && p > 0) text = fmt((natural * p) / 100);
  }
</script>

<Dialog title="Time Stretch" {onClose} onEnter={() => secs !== null && !outOfRange && onApply(secs, pitch, false)}>
  <p class="note">{clipName}: {fmt(natural)} s at natural speed{stretched ? `, now ${fmt(current)} s` : ""}.</p>

  <label class="row">
    <span>New length</span>
    <input type="text" bind:value={text} aria-label="New length in seconds, or minutes:seconds" />
  </label>
  <div class="chips" role="group" aria-label="Common lengths">
    <button type="button" onclick={() => (text = fmt(15))}>15 s</button>
    <button type="button" onclick={() => (text = fmt(30))}>30 s</button>
    <button type="button" onclick={() => (text = fmt(60))}>60 s</button>
    <button type="button" disabled={selection === null} title={selection === null ? "Make a time selection first." : "Fit the clip into the time selection and move it there."} onclick={() => selection !== null && onApply(selection, pitch, true)}>Fit to Time Selection</button>
  </div>
  {#if selection === null}<p class="hint chips-hint">Fit to Time Selection needs a time selection in the Multitrack.</p>{/if}
  <label class="row">
    <span>Or percent</span>
    <span class="pct"><input class="short" type="number" min="25" max="400" step="0.5" bind:value={pct} onchange={fromPercent} aria-label="Percent of natural length" /> % of natural length</span>
  </label>

  <fieldset>
    <label><input type="radio" name="mode" checked={pitch} onchange={() => (pitch = true)} /> Keep pitch (speech, ad reads)</label>
    <label><input type="radio" name="mode" checked={!pitch} onchange={() => (pitch = false)} /> Change pitch with speed (tape style)</label>
  </fieldset>

  {#if secs === null}
    <p class="warn">Type a length in seconds (30) or minutes and seconds (1:05.5).</p>
  {:else if outOfRange}
    <p class="warn">A clip can play from 25% to 400% of its natural length.</p>
  {:else if ratio !== null}
    <p class="note">That plays at {percent(ratio)} of natural length.{far ? " Beyond about 25% either way, speech starts to sound processed." : ""}</p>
  {/if}

  {#snippet footer()}
    {#if stretched}<button type="button" class="start" onclick={onReset}>Remove Stretch</button>{/if}
    <button type="button" onclick={onClose}>Cancel</button>
    <button type="button" class="primary" disabled={secs === null || outOfRange} onclick={() => secs !== null && onApply(secs, pitch, false)}>Apply</button>
  {/snippet}
</Dialog>

<style>
  .pct {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .chips-hint {
    padding-left: calc(var(--dlg-label) + 8px);
  }
</style>
