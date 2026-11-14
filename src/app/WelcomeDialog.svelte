<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Shown on a fresh install, and again when a release adds a page worth stopping for.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
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

<Dialog title={page.title} width={480} {onClose}>
  {#if kind === "upgrade"}<p class="kicker">What's new</p>{/if}
  {#if picture}<img src={picture} alt="" />{/if}
  {#each page.body as para}<p class="para">{para}</p>{/each}
  {#if pages.length > 1}<p class="hint" aria-label="Page {at + 1} of {pages.length}">{at + 1} / {pages.length}</p>{/if}
  {#snippet footer()}
    <button type="button" class="start" onclick={onGuide}>Open the User Guide</button>
    {#if at > 0}<button type="button" onclick={() => (at -= 1)}>Back</button>{/if}
    {#if last}
      <button type="button" class="primary" onclick={onClose}>{kind === "install" ? "Start Editing" : "Got It"}</button>
    {:else}
      <button type="button" class="primary" onclick={() => (at += 1)}>Next</button>
    {/if}
  {/snippet}
</Dialog>

<style>
  .kicker {
    margin: 0;
    color: var(--text-dim);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  img {
    display: block;
    width: 100%;
    border: 1px solid var(--border-strong);
  }
  .para {
    margin: 0;
    line-height: 1.45;
  }
</style>
