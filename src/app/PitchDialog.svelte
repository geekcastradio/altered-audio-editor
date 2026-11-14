<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Effects > Pitch and Time > Pitch Tune and Pitch Curve (Waveform editor).
  Tune pulls a voice toward the notes of a key and scale. Bend moves the
  pitch along a line over the selection. Both keep the voice's character
  and leave breaths and consonants alone. Made for one voice at a time
  and small moves.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { MAX_CENTS, NOTES, SCALES, isNoChange, type PitchChoices } from "./pitch-edit";

  interface Props {
    choices: PitchChoices;
    /** What the edit will work on, such as "the selection" or "the whole file". */
    scope: string;
    busy: boolean;
    onApply: (c: PitchChoices) => void;
    onClose: () => void;
  }

  const { choices, scope, busy, onApply, onClose }: Props = $props();
  // svelte-ignore state_referenced_locally
  let c = $state<PitchChoices>({ ...choices });
  // svelte-ignore state_referenced_locally
  let middle = $state(choices.bendMiddle !== null);
  const shown = $derived<PitchChoices>({ ...c, bendMiddle: middle ? (c.bendMiddle ?? 0) : null });
  const idle = $derived(isNoChange(shown));
</script>

<Dialog title="Pitch Tune and Curve" width={460} raised canClose={!busy} labelWidth={120} {onClose}>
  <p class="note">Works on {scope}. Made for one voice or instrument at a time and for small moves. Breaths, consonants and silence are left as they are.</p>

  <div class="tabs" role="tablist">
    <button type="button" role="tab" aria-selected={c.mode === "tune"} class:on={c.mode === "tune"} onclick={() => (c.mode = "tune")}>Tune to a key</button>
    <button type="button" role="tab" aria-selected={c.mode === "bend"} class:on={c.mode === "bend"} onclick={() => (c.mode = "bend")}>Bend (pitch curve)</button>
  </div>

  {#if c.mode === "tune"}
    <label class="sliders"><span>Key</span>
      <select bind:value={c.key}>
        {#each NOTES as n, i (n)}<option value={i}>{n}</option>{/each}
      </select>
    </label>
    <label class="sliders"><span>Scale</span>
      <select bind:value={c.scale}>
        {#each SCALES as s (s.id)}<option value={s.id}>{s.label}</option>{/each}
      </select>
    </label>
    <label class="sliders"><span>Strength</span>
      <input type="range" min="0" max="100" step="1" bind:value={c.strength} /><output>{Math.round(c.strength)}%</output>
    </label>
    <label class="sliders"><span>Speed</span>
      <input type="range" min="0" max="300" step="5" bind:value={c.speedMs} /><output>{c.speedMs === 0 ? "instant" : `${Math.round(c.speedMs)} ms`}</output>
    </label>
    <label class="sliders"><span>Leave alone past</span>
      <input type="range" min="10" max={MAX_CENTS} step="10" bind:value={c.maxCents} /><output>{Math.round(c.maxCents)} cents</output>
    </label>
    <p class="hint">Strength is how much of the way to the note it goes. A low speed number follows the correction quickly (the effect people hear as robotic); a higher one keeps the natural slides. Notes further off than the last setting are left alone, since they are probably meant.</p>
  {:else}
    <label class="sliders"><span>At the start</span>
      <input type="range" min={-MAX_CENTS} max={MAX_CENTS} step="5" bind:value={c.bendStart} /><output>{Math.round(c.bendStart)} cents</output>
    </label>
    <label class="sliders"><span><input type="checkbox" bind:checked={middle} /> In the middle</span>
      <input type="range" min={-MAX_CENTS} max={MAX_CENTS} step="5" disabled={!middle} value={c.bendMiddle ?? 0} oninput={(e) => (c.bendMiddle = Number(e.currentTarget.value))} /><output>{middle ? Math.round(c.bendMiddle ?? 0) : "-"}</output>
    </label>
    <label class="sliders"><span>At the end</span>
      <input type="range" min={-MAX_CENTS} max={MAX_CENTS} step="5" bind:value={c.bendEnd} /><output>{Math.round(c.bendEnd)} cents</output>
    </label>
    <p class="hint">100 cents is one semitone. The pitch moves in straight lines between these points, across {scope}.</p>
  {/if}

  {#snippet footer()}
    <button type="button" onclick={onClose} disabled={busy}>Cancel</button>
    <button type="button" class="primary" disabled={busy || idle} onclick={() => onApply(shown)}>{busy ? "Working..." : "Apply"}</button>
  {/snippet}
</Dialog>

<style>
  .tabs {
    display: flex;
    gap: 4px;
  }
  .tabs button {
    flex: 1;
  }
  .tabs .on {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--on-accent, #fff);
  }
  .sliders {
    display: grid;
    grid-template-columns: var(--dlg-label) minmax(0, 1fr) 70px;
    gap: 8px;
    align-items: center;
    color: var(--text-dim);
  }
  .sliders select {
    grid-column: 2 / 4;
  }
  output {
    text-align: right;
    color: var(--text);
  }
</style>
