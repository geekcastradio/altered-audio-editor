<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Multitrack > Recording Options: the latency offset that lines takes up
  with what they were played against, and how punch-in recording works.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import type { RecordOptions } from "./recording";

  interface Props {
    initial: RecordOptions;
    onApply: (o: RecordOptions) => void;
    onClose: () => void;
  }

  const { initial, onApply, onClose }: Props = $props();
  // svelte-ignore state_referenced_locally
  let o = $state<RecordOptions>({ ...initial });
</script>

<Dialog title="Recording Options" width={460} {onClose}>
  <fieldset>
    <legend>Latency offset</legend>
    <label class="inline">Trim from each take <input class="short" type="number" min="0" max="1000" step="1" bind:value={o.offsetMs} /> ms</label>
    <p class="note">Takes can land slightly late against the music or other tracks. To find the number: record a clap or a click track through your mic, zoom in, and measure how far the recorded clap sits after the original. Most USB interfaces land between 5 and 30 ms.</p>
  </fieldset>
  <fieldset>
    <legend>Punch-in</legend>
    <label><input type="checkbox" bind:checked={o.punch} /> With a time selection, record only into the selection</label>
    <label class="inline">Play before it <input class="short" type="number" min="0" max="10" step="0.5" bind:value={o.prerollSec} disabled={!o.punch} /> seconds</label>
    <label><input type="checkbox" bind:checked={o.replace} disabled={!o.punch} /> Replace what was in the selection on the recorded tracks</label>
    <p class="note">Recording stops by itself at the end of the selection. Without a time selection, recording works as usual from the playhead.</p>
  </fieldset>
  {#snippet footer()}
    <button type="button" onclick={onClose}>Cancel</button>
    <button type="button" class="primary" onclick={() => onApply({ ...o, offsetMs: Number(o.offsetMs) || 0, prerollSec: Number(o.prerollSec) || 0 })}>OK</button>
  {/snippet}
</Dialog>

<style>
  legend {
    padding: 0 4px;
    color: var(--text-dim);
  }
  .inline {
    display: flex;
    gap: 6px;
    align-items: center;
  }
</style>
