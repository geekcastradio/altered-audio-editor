<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Help > User Guide: sections on the left, search on top, the text on the right.
-->
<script lang="ts">
  import guideSource from "./guide/guide.md?raw";
  import { parseGuide, searchGuide, type Span } from "./guide/guide";

  interface Props {
    onClose: () => void;
  }

  const { onClose }: Props = $props();

  const all = parseGuide(guideSource);
  let query = $state("");
  let picked = $state(all[0]?.id ?? "");
  const shown = $derived(searchGuide(all, query));
  const current = $derived(shown.find((s) => s.id === picked) ?? shown[0] ?? null);
</script>

{#snippet runs(list: Span[])}
  {#each list as r, i (i)}{#if r.bold}<strong>{r.text}</strong>{:else if r.code}<code>{r.text}</code>{:else}{r.text}{/if}{/each}
{/snippet}

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div class="dialog" role="dialog" tabindex="-1" aria-modal="true" aria-labelledby="guide-title" onclick={(e) => e.stopPropagation()}>
    <header>
      <h1 id="guide-title">User Guide</h1>
      <!-- svelte-ignore a11y_autofocus -->
      <input type="search" placeholder="Search the guide" aria-label="Search the guide" bind:value={query} autofocus />
      <button type="button" onclick={onClose}>Close</button>
    </header>
    <div class="body">
      <nav aria-label="Guide sections">
        {#each shown as s (s.id)}
          <button type="button" class:on={current?.id === s.id} onclick={() => (picked = s.id)}>{s.title}</button>
        {:else}
          <p class="none">No section has all of those words.</p>
        {/each}
      </nav>
      <article>
        {#if current}
          <h2>{current.title}</h2>
          {#each current.blocks as b, i (i)}
            {#if b.kind === "p"}
              <p>{@render runs(b.spans)}</p>
            {:else if b.kind === "ul"}
              <ul>{#each b.items as it, j (j)}<li>{@render runs(it)}</li>{/each}</ul>
            {:else}
              <ol>{#each b.items as it, j (j)}<li>{@render runs(it)}</li>{/each}</ol>
            {/if}
          {/each}
        {/if}
      </article>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 5000;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.5);
  }
  .dialog {
    display: flex;
    flex-direction: column;
    width: min(900px, 94vw);
    height: min(640px, 90vh);
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  }
  header {
    display: flex;
    gap: 12px;
    align-items: center;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
  }
  h1 {
    margin: 0;
    font-size: 15px;
    flex: none;
  }
  input {
    flex: 1;
    min-width: 0;
  }
  .body {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  nav {
    display: flex;
    flex-direction: column;
    flex: none;
    width: 210px;
    padding: 6px;
    overflow-y: auto;
    border-right: 1px solid var(--border);
  }
  nav button {
    padding: 5px 8px;
    border: 0;
    border-radius: 3px;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }
  nav button:hover {
    background: var(--surface-3, rgba(255, 255, 255, 0.06));
  }
  nav button.on {
    background: var(--accent-dim, rgba(255, 255, 255, 0.12));
    font-weight: 600;
  }
  .none {
    padding: 6px 8px;
    color: var(--text-dim);
  }
  article {
    flex: 1;
    min-width: 0;
    padding: 6px 22px 22px;
    overflow-y: auto;
    line-height: 1.5;
    user-select: text;
  }
  h2 {
    margin: 12px 0 8px;
    font-size: 18px;
  }
  p,
  li {
    margin: 0 0 8px;
  }
  ul,
  ol {
    margin: 0 0 8px;
    padding-left: 22px;
  }
  code {
    padding: 0 3px;
    border-radius: 3px;
    background: var(--surface-0);
  }
</style>
