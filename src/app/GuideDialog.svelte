<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Help > User Guide: sections on the left, search on top, the text on the right.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import guideSource from "./guide/guide.md?raw";
  import { parseGuide, searchGuide, type Span } from "./guide/guide";

  interface Props {
    onClose: () => void;
  }

  const { onClose }: Props = $props();

  // Pictures are our own SVG files bundled with the program, drawn inline so they need no network or file access.
  const pictures = import.meta.glob("./guide/images/*.svg", { eager: true, query: "?raw", import: "default" }) as Record<string, string>;
  const picture = (name: string) => pictures[`./guide/images/${name}.svg`] ?? "";

  const all = parseGuide(guideSource);
  let query = $state("");
  let picked = $state(all[0]?.id ?? "");
  const shown = $derived(searchGuide(all, query));
  const current = $derived(shown.find((s) => s.id === picked) ?? shown[0] ?? null);
</script>

{#snippet runs(list: Span[])}
  {#each list as r, i (i)}{#if r.bold}<strong>{r.text}</strong>{:else if r.code}<code>{r.text}</code>{:else}{r.text}{/if}{/each}
{/snippet}

<Dialog title="User Guide" width={900} {onClose}>
  <input type="search" placeholder="Search the guide" aria-label="Search the guide" bind:value={query} data-autofocus />
  <div class="reader">
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
          {:else if b.kind === "img"}
            <figure>
              {@html picture(b.name)}
              <figcaption>{b.alt}</figcaption>
            </figure>
          {:else if b.kind === "ul"}
            <ul>{#each b.items as it, j (j)}<li>{@render runs(it)}</li>{/each}</ul>
          {:else}
            <ol>{#each b.items as it, j (j)}<li>{@render runs(it)}</li>{/each}</ol>
          {/if}
        {/each}
      {/if}
    </article>
  </div>
  {#snippet footer()}
    <button type="button" onclick={onClose}>Close</button>
  {/snippet}
</Dialog>

<style>
  .reader {
    display: flex;
    height: min(520px, calc(100vh - 210px));
    min-height: 200px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  nav {
    display: flex;
    flex-direction: column;
    flex: none;
    width: 230px;
    padding: 6px;
    overflow-y: auto;
    border-right: 1px solid var(--border);
  }
  nav button {
    flex-shrink: 0;
    padding: 6px 10px;
    margin-bottom: 2px;
    line-height: 1.3;
    white-space: normal;
    overflow-wrap: anywhere;
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
  figure {
    margin: 4px 0 14px;
  }
  figure :global(svg) {
    display: block;
    width: 100%;
    max-width: 640px;
    height: auto;
    border-radius: 6px;
  }
  figcaption {
    margin-top: 4px;
    color: var(--text-dim);
    font-size: 11px;
  }
  code {
    padding: 0 3px;
    border-radius: 3px;
    background: var(--surface-0);
  }
</style>
