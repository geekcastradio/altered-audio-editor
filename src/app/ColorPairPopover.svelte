<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Two colors for an analysis view (low and high), opened from the view's
  right-click menu. Changes show at once and are saved with the app's
  settings (also under Edit > Settings > Colors). Default goes back to
  the built-in colors.
-->
<script lang="ts">
  interface Props {
    x: number;
    y: number;
    title: string;
    lowLabel: string;
    highLabel: string;
    /** The current colors; empty means the built-in one. */
    low: string;
    high: string;
    /** What the color inputs show while on the built-in colors. */
    lowDefault: string;
    highDefault: string;
    /** Ready-made pairs to pick from, shown as swatches. */
    presets?: readonly { name: string; low: string; high: string; scheme?: string; swatch?: string }[];
    onChange: (low: string, high: string, scheme?: string) => void;
    onClose: () => void;
  }

  const { x, y, title, lowLabel, highLabel, low, high, lowDefault, highDefault, presets = [], onChange, onClose }: Props = $props();

  let box = $state<HTMLDivElement | null>(null);
  const left = $derived(Math.max(4, Math.min(x, window.innerWidth - 240)));
  const top = $derived(Math.max(4, Math.min(y, window.innerHeight - (presets.length ? 210 : 150))));
  $effect(() => {
    const away = (e: PointerEvent) => {
      if (box && !box.contains(e.target as Node)) onClose();
    };
    window.addEventListener("pointerdown", away, true);
    return () => window.removeEventListener("pointerdown", away, true);
  });
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} />

<div class="pop" bind:this={box} role="dialog" aria-label={title} style:left="{left}px" style:top="{top}px">
  <h2>{title}</h2>
  {#if presets.length}
    <div class="presets" role="group" aria-label="Color schemes">
      {#each presets as p (p.name)}
        <button type="button" class="preset" title={p.name} aria-label={p.name} style:background={p.swatch ?? `linear-gradient(to right, ${p.low}, ${p.high})`} onclick={() => onChange(p.low, p.high, p.scheme ?? "")}></button>
      {/each}
    </div>
  {/if}
  <label>
    <input type="color" value={low || lowDefault} oninput={(e) => onChange(e.currentTarget.value, high || highDefault)} />
    {lowLabel}
  </label>
  <label>
    <input type="color" value={high || highDefault} oninput={(e) => onChange(low || lowDefault, e.currentTarget.value)} />
    {highLabel}
  </label>
  <div class="row">
    <span class="ramp" style:background="linear-gradient(to right, {low || lowDefault}, {high || highDefault})"></span>
    <button type="button" disabled={!low && !high} onclick={() => onChange("", "")}>Default</button>
    <button type="button" onclick={onClose}>Done</button>
  </div>
</div>

<style>
  .pop {
    position: fixed;
    z-index: 6000;
    display: grid;
    gap: 6px;
    width: 220px;
    padding: 10px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 10px 28px rgba(0, 0, 0, 0.55);
    font-size: var(--font-size-small);
  }
  h2 {
    margin: -10px -10px 2px;
    padding: 5px 10px;
    background: var(--title-bar);
    color: var(--title-bar-text);
    font-size: var(--font-size-small);
    font-weight: 600;
  }
  label {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  input[type="color"] {
    width: 34px;
    height: 22px;
    padding: 0;
    border: 1px solid var(--border-strong);
    background: none;
  }
  .row {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }
  .preset {
    width: 34px;
    height: 16px;
    padding: 0;
    border-radius: 3px;
  }
  .preset:hover {
    border-color: var(--accent);
  }
  .ramp {
    flex: 1;
    height: 10px;
    border-radius: 3px;
  }
  button {
    height: 22px;
    padding: 0 8px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
  }
</style>
