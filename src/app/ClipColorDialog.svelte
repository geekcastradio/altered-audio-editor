<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Picks a clip or track color from the palette or any custom color, or
  the default (the track's color for a clip). Escape cancels.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";

  interface Props {
    /** Current color: a token like "--clip-3", "#rrggbb", or null for the track color. */
    current: string | null;
    /** The picked color ("" for the track color), or null when cancelled. */
    onDone: (color: string | null) => void;
    title?: string;
    noneLabel?: string;
  }

  const { current, onDone, title = "Clip Color", noneLabel = "Use Track Color" }: Props = $props();
  const SWATCHES = ["--clip-1", "--clip-2", "--clip-3", "--clip-4", "--clip-5", "--clip-6"];
  const custom = $derived(current?.startsWith("#") ? current : "#7fa7d9");
</script>

<Dialog {title} width={290} onClose={() => onDone(null)}>
  <div class="swatches">
    {#each SWATCHES as c, i (c)}
      <button
        type="button"
        class:on={current === c}
        data-autofocus={i === 0 ? "" : undefined}
        style:background="var({c})"
        aria-label="Color {c.slice(-1)}"
        onclick={() => onDone(c)}
      ></button>
    {/each}
  </div>
  <label class="custom">
    <span>Other color</span>
    <input type="color" value={custom} onchange={(e) => onDone(e.currentTarget.value)} />
  </label>
  {#snippet footer()}
    <button type="button" class="start" class:on={current === null} onclick={() => onDone("")}>{noneLabel}</button>
    <button type="button" onclick={() => onDone(null)}>Cancel</button>
  {/snippet}
</Dialog>

<style>
  .swatches {
    display: grid;
    grid-template-columns: repeat(6, 32px);
    gap: 8px;
  }
  .swatches button {
    width: 32px;
    height: 24px;
    padding: 0;
    border: 2px solid transparent;
    border-radius: 3px;
  }
  .swatches button:hover,
  .swatches button.on {
    border-color: #fff;
  }
  .custom {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: var(--font-size-small);
  }
  .custom input {
    width: 48px;
    height: 24px;
    padding: 0;
    border: 1px solid var(--border-strong);
    background: none;
  }
  .on {
    border-color: var(--accent);
  }
</style>
