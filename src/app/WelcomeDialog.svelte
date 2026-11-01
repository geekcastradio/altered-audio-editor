<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Shown on a fresh install, and again when a release adds a page worth stopping for.
-->
<script lang="ts">
  import type { WelcomePage } from "./welcome";

  interface Props {
    kind: "install" | "upgrade";
    pages: WelcomePage[];
    onGuide: () => void;
    onClose: () => void;
  }

  const { kind, pages, onGuide, onClose }: Props = $props();

  // Pictures are optional: a page whose file is not there just shows its text.
  const pictures = import.meta.glob("../assets/welcome/*.png", { eager: true, query: "?url", import: "default" }) as Record<string, string>;
  function pictureOf(name: string | undefined): string | null {
    return name ? (pictures[`../assets/welcome/${name}.png`] ?? null) : null;
  }

  let at = $state(0);
  const page = $derived(pages[at]!);
  const last = $derived(at === pages.length - 1);
  const picture = $derived(pictureOf(page.image));
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div class="dialog" role="dialog" tabindex="-1" aria-modal="true" aria-labelledby="welcome-title" onclick={(e) => e.stopPropagation()}>
    {#if kind === "upgrade"}<p class="kicker">What's new</p>{/if}
    <h1 id="welcome-title">{page.title}</h1>
    {#if picture}<img src={picture} alt="" />{/if}
    {#each page.body as para}<p>{para}</p>{/each}
    {#if pages.length > 1}<p class="dots" aria-label="Page {at + 1} of {pages.length}">{at + 1} / {pages.length}</p>{/if}
    <div class="actions">
      <button type="button" onclick={onGuide}>Open the User Guide</button>
      <span class="grow"></span>
      {#if at > 0}<button type="button" onclick={() => (at -= 1)}>Back</button>{/if}
      {#if last}
        <button type="button" class="primary" onclick={onClose}>{kind === "install" ? "Start Editing" : "Got It"}</button>
      {:else}
        <button type="button" class="primary" onclick={() => (at += 1)}>Next</button>
      {/if}
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
    width: 480px;
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 32px);
    overflow: auto;
    padding: 20px 24px 16px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  }
  h1 {
    margin: 0 0 10px;
    font-size: 17px;
  }
  .kicker {
    margin: 0 0 2px;
    color: var(--text-dim);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  img {
    display: block;
    width: 100%;
    margin: 0 0 12px;
    border: 1px solid var(--border-strong);
  }
  p {
    margin: 0 0 8px;
    line-height: 1.45;
  }
  .dots {
    color: var(--text-dim);
    font-size: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }
  .grow {
    flex: 1;
  }
</style>
