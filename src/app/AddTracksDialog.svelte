<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Multitrack > Add Track > Add Tracks: several mono or stereo tracks at
  once, optionally named ("Guest" gives Guest, Guest 2, Guest 3), placed
  after the selected track, MIDI tracks (notes played by a VST3
  instrument), or bus tracks (added below the audio tracks). Surround
  tracks show why they are not offered.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { newTrackNames } from "./track-names";

  export interface AddTracksValues {
    count: number;
    stereo: boolean;
    /** Bus tracks instead of audio tracks. */
    bus: boolean;
    /** MIDI tracks instead of audio tracks. */
    midi: boolean;
    base: string;
  }

  type Kind = "mono" | "stereo" | "midi" | "bus";

  interface Props {
    existing: readonly string[];
    afterName: string | null;
    /** Music Mode is on: MIDI tracks are offered. */
    music?: boolean;
    onApply: (v: AddTracksValues) => void;
    onClose: () => void;
  }

  const { existing, afterName, music = false, onApply, onClose }: Props = $props();
  let count = $state(2);
  let kind = $state<Kind>("mono");
  let base = $state("");
  const n = $derived(Math.max(1, Math.min(32, Math.round(Number(count) || 1))));
  const apply = () => onApply({ count: n, stereo: kind !== "mono", bus: kind === "bus", midi: kind === "midi", base: base || (kind === "bus" ? "Bus" : kind === "midi" ? "MIDI" : "") });
  const preview = $derived(newTrackNames(existing, Math.min(n, 4), base || (kind === "bus" ? "Bus" : kind === "midi" ? "MIDI" : "")));
</script>

<Dialog title="Add Tracks" width={400} {onClose} onEnter={apply}>
  <label class="row"><span>How many</span><input type="number" min="1" max="32" bind:value={count} /></label>
  <label class="row"><span>Type</span>
    <select bind:value={kind}>
      <option value="mono">Mono audio (one voice or mic)</option>
      <option value="stereo">Stereo audio (music, beds, mixes)</option>
      {#if music}<option value="midi">MIDI (notes played by an instrument plug-in)</option>{/if}
      <option value="bus">Stereo bus (groups tracks, takes sends)</option>
      <option disabled>5.1 surround (not planned for podcast work)</option>
    </select>
  </label>
  <label class="row"><span>Name</span><input type="text" placeholder="Track" bind:value={base} /></label>
  <p class="note">Adds {preview.join(", ")}{n > 4 ? `, and ${n - 4} more` : ""}{kind === "bus" ? " below the audio tracks" : afterName ? ` after ${afterName}` : " at the bottom"}.</p>
  {#snippet footer()}
    <button type="button" onclick={onClose}>Cancel</button>
    <button type="button" class="primary" onclick={apply}>Add</button>
  {/snippet}
</Dialog>
