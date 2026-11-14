<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  File > New Audio File: name, sample rate, channels and bit depth for
  an empty file in the Waveform editor. Enter confirms, Escape cancels.
-->
<script lang="ts" module>
  export interface NewAudioSettings {
    name: string;
    sampleRate: number;
    channels: 1 | 2;
    bitDepth: number;
  }
</script>

<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { formatRate } from "../panels/files/format";
  import { BIT_DEPTHS, SAMPLE_RATES } from "./session-properties";

  interface Props {
    initial: NewAudioSettings;
    onDone: (value: NewAudioSettings | null) => void;
  }

  const { initial, onDone }: Props = $props();

  // svelte-ignore state_referenced_locally
  let name = $state(initial.name);
  // svelte-ignore state_referenced_locally
  let sampleRate = $state(initial.sampleRate);
  // svelte-ignore state_referenced_locally
  let channels = $state<1 | 2>(initial.channels);
  // svelte-ignore state_referenced_locally
  let bitDepth = $state(initial.bitDepth);
  function done() {
    if (name.trim()) onDone({ name: name.trim(), sampleRate, channels, bitDepth });
  }
</script>

<Dialog title="New Audio File" onClose={() => onDone(null)} onEnter={done}>
  <label class="stack">
    <span>Name</span>
    <input type="text" spellcheck="false" bind:value={name} />
  </label>
  <div class="three">
    <label class="stack">
      <span>Sample rate</span>
      <select bind:value={sampleRate}>
        {#each SAMPLE_RATES as r (r)}
          <option value={r}>{formatRate(r)}</option>
        {/each}
      </select>
    </label>
    <label class="stack">
      <span>Channels</span>
      <select bind:value={channels}>
        <option value={1}>Mono</option>
        <option value={2}>Stereo</option>
      </select>
    </label>
    <label class="stack">
      <span>Bit depth</span>
      <select bind:value={bitDepth}>
        {#each BIT_DEPTHS as d (d.value)}
          <option value={d.value}>{d.label}</option>
        {/each}
      </select>
    </label>
  </div>
  <p class="note">The file starts empty. Paste audio into it, then use Save Audio As to choose where it goes.</p>
  {#snippet footer()}
    <button type="button" onclick={() => onDone(null)}>Cancel</button>
    <button type="button" class="primary" disabled={!name.trim()} onclick={done}>OK</button>
  {/snippet}
</Dialog>

<style>
  .three {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 10px;
  }
</style>
