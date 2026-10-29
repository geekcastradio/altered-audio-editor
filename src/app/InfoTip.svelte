<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  A small (i) in a circle. Hover or focus it for a short explanation;
  click to keep it open. Keeps long explanations out of the way until
  they are wanted.
-->
<script lang="ts">
  interface Props {
    /** The explanation. Plain text. */
    text: string;
    /** What the (i) is for, for screen readers. */
    label?: string;
  }

  const { text, label = "More information" }: Props = $props();

  let hover = $state(false);
  let pinned = $state(false);
  const open = $derived(hover || pinned);
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && pinned && ((pinned = false), e.stopPropagation())} />

<span class="tip" role="presentation" onmouseenter={() => (hover = true)} onmouseleave={() => (hover = false)}>
  <button type="button" class="i" aria-label={label} aria-expanded={open} onclick={() => (pinned = !pinned)} onfocus={() => (hover = true)} onblur={() => (hover = false)}>i</button>
  {#if open}
    <span class="pop" role="tooltip">{text}</span>
  {/if}
</span>

<style>
  .tip {
    position: relative;
    display: inline-flex;
    align-items: center;
  }

  .i {
    width: 16px;
    height: 16px;
    padding: 0;
    border: 1px solid var(--text-faint);
    border-radius: 50%;
    background: transparent;
    color: var(--text-dim);
    font-family: Georgia, serif;
    font-size: 11px;
    font-style: italic;
    line-height: 1;
    cursor: help;
  }

  .i:hover,
  .i[aria-expanded="true"] {
    border-color: var(--accent);
    color: var(--text);
  }

  .pop {
    position: absolute;
    z-index: 10;
    top: 22px;
    right: -4px;
    width: 260px;
    padding: 8px 10px;
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    background: var(--surface-1);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.5);
    color: var(--text);
    font-size: var(--font-size-small);
    font-style: normal;
    line-height: 1.4;
  }
</style>
