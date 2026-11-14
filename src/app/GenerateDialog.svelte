<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Generate tools. Tone and Noise make a new audio file and place it as
  a clip at the playhead on the selected track. Insert Silence makes
  room at the playhead: everything after it moves later on every track
  (or only the selected one), and a clip under the playhead is split.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { parseSeconds } from "../panels/editor/stretch";

  export type GenerateMode = "tone" | "noise" | "silence";
  export interface GenerateValues {
    seconds: number;
    frequency: number;
    levelDb: number;
    noise: "white" | "pink";
    channels: 1 | 2;
    allTracks: boolean;
    /** Insert Silence with a time selection: silence that range in place instead of pushing audio later. */
    overwrite: boolean;
  }

  interface Props {
    mode: GenerateMode;
    initial: GenerateValues;
    trackName: string | null;
    /** Seconds of the time selection, when there is one. */
    selection?: number | null;
    onApply: (v: GenerateValues) => void;
    onClose: () => void;
  }

  const { mode, initial, trackName, selection = null, onApply, onClose }: Props = $props();
  // svelte-ignore state_referenced_locally
  let v = $state<GenerateValues>({ ...initial });
  // svelte-ignore state_referenced_locally
  let lengthText = $state(String(initial.seconds));
  // svelte-ignore state_referenced_locally
  if (mode === "silence" && selection !== null) v.overwrite = true;
  const replacing = $derived(mode === "silence" && selection !== null && v.overwrite);
  const secs = $derived(parseSeconds(lengthText));
  const title = $derived(mode === "tone" ? "Generate Tone" : mode === "noise" ? "Generate Noise" : "Insert Silence");
  const ok = $derived(replacing || secs !== null && secs <= 3600 && (mode !== "tone" || (v.frequency >= 1 && v.frequency <= 20_000)));
  const apply = () => {
    if (!ok) return;
    if (replacing) onApply({ ...v, seconds: selection ?? 0 });
    else if (secs !== null) onApply({ ...v, seconds: secs });
  };
</script>

<Dialog {title} width={420} {onClose} onEnter={apply}>
  <p class="note">
    {#if mode === "silence" && replacing}
      Replaces the time selection with silence. Nothing moves; audio in the selection is removed.
    {:else if mode === "silence"}
      Makes room at the playhead. Clips after it move later; a clip under the playhead is split.
    {:else}
      Creates a new audio file in the session's Generated folder and places it at the playhead{trackName ? ` on ${trackName}` : ""}.
    {/if}
  </p>
  {#if mode === "silence" && selection !== null}
    <fieldset>
      <label><input type="radio" name="how" checked={v.overwrite} onchange={() => (v.overwrite = true)} /> Silence the time selection ({selection.toFixed(2)} s), moving nothing</label>
      <label><input type="radio" name="how" checked={!v.overwrite} onchange={() => (v.overwrite = false)} /> Insert new silence at the playhead, pushing audio later</label>
    </fieldset>
  {/if}
  {#if !replacing}
    <label class="fields"><span>Length</span><input type="text" bind:value={lengthText} aria-label="Length in seconds or minutes:seconds" /> <span class="unit">seconds, or m:ss</span></label>
  {/if}
  {#if mode === "tone"}
    <label class="fields"><span>Frequency</span><input type="number" min="1" max="20000" step="1" bind:value={v.frequency} /> <span class="unit">Hz</span></label>
  {/if}
  {#if mode === "noise"}
    <label class="fields"><span>Type</span>
      <select bind:value={v.noise}>
        <option value="pink">Pink (softer, like room tone)</option>
        <option value="white">White (even, bright)</option>
      </select>
    </label>
  {/if}
  {#if mode !== "silence"}
    <label class="fields"><span>Level</span><input type="number" min="-96" max="0" step="1" bind:value={v.levelDb} /> <span class="unit">dBFS {mode === "tone" ? "peak" : "average"}</span></label>
    <label class="fields"><span>Channels</span>
      <select bind:value={v.channels}>
        <option value={1}>Mono</option>
        <option value={2}>Stereo</option>
      </select>
    </label>
  {:else}
    <fieldset>
      <label><input type="radio" name="scope" checked={v.allTracks} onchange={() => (v.allTracks = true)} /> On every track{replacing ? "" : " (keeps them in sync; markers move too)"}</label>
      <label><input type="radio" name="scope" checked={!v.allTracks} disabled={!trackName} onchange={() => (v.allTracks = false)} /> Only on {trackName ?? "the selected track (select one first)"}</label>
    </fieldset>
  {/if}
  {#if secs === null && !replacing}<p class="warn">Type a length like 5, 2.5 or 1:30.</p>{/if}
  {#snippet footer()}
    <button type="button" onclick={onClose}>Cancel</button>
    <button type="button" class="primary" disabled={!ok} onclick={apply}>{replacing ? "Silence" : mode === "silence" ? "Insert" : "Generate"}</button>
  {/snippet}
</Dialog>

<style>
  .fields {
    display: grid;
    grid-template-columns: 76px 110px auto;
    gap: 8px;
    align-items: center;
    color: var(--text-dim);
  }
  .fields select {
    grid-column: span 2;
  }
  .unit {
    color: var(--text-dim);
  }
</style>
