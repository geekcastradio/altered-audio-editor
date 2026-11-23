<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Three buttons in the header: Light Mode (sun), Sync with System (monitor)
  and Dark Mode (moon). Each has a hover description.
-->
<script lang="ts">
  import { THEMES, type ThemeChoice } from "./theme";

  let { value, onChoose }: { value: string; onChoose: (c: ThemeChoice) => void } = $props();
</script>

<div class="themes" role="radiogroup" aria-label="Look">
  {#each THEMES as t (t.id)}
    <button type="button" role="radio" aria-checked={value === t.id} aria-label={t.label} title={t.hint} class:on={value === t.id} onclick={() => onChoose(t.id)}>
      <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        {#if t.id === "light"}
          <circle cx="8" cy="8" r="2.8" />
          <path d="M8 1.5v1.6M8 12.9v1.6M1.5 8h1.6M12.9 8h1.6M3.4 3.4l1.1 1.1M11.5 11.5l1.1 1.1M3.4 12.6l1.1-1.1M11.5 4.5l1.1-1.1" />
        {:else if t.id === "dark"}
          <path d="M13.2 9.6A5.6 5.6 0 0 1 6.4 2.8a5.6 5.6 0 1 0 6.8 6.8z" />
        {:else}
          <rect x="1.8" y="2.6" width="12.4" height="8.2" rx="1" />
          <path d="M5.6 13.4h4.8M8 10.8v2.6" />
        {/if}
      </svg>
    </button>
  {/each}
</div>

<style>
  .themes {
    display: inline-flex;
    margin: 0 0 0 auto;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  button {
    display: grid;
    place-items: center;
    width: 26px;
    height: 20px;
    padding: 0;
    border: 0;
    border-right: 1px solid var(--border-strong);
    background: var(--surface-3);
    color: var(--text-dim);
    cursor: pointer;
  }
  button:last-child {
    border-right: 0;
  }
  button:hover {
    color: var(--text);
  }
  button.on {
    background: var(--accent);
    color: var(--on-accent, #fff);
  }
</style>
