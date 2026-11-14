<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Help > Check for Updates: a newer version was found. Nothing installs
  until Install and Restart is clicked.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import { plainNotes, type UpdateInfo } from "./update";

  interface Props {
    info: UpdateInfo;
    busy: boolean;
    onInstall: () => void;
    onPage: () => void;
    onClose: () => void;
  }

  const { info, busy, onInstall, onPage, onClose }: Props = $props();
</script>

<Dialog title="Version {info.version} is available" canClose={!busy} {onClose}>
  <div class="text">
    <p class="version">You have version {info.current}.</p>
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    {#if info.notes}<div class="notes" tabindex="0" aria-label="What changed">{plainNotes(info.notes)}</div>{/if}
    {#if info.canInstall}
      <p class="fine">Save your work first. The update downloads, installs and restarts the app. Unsaved sessions are offered a recovery copy, but saving is safer.</p>
    {:else}
      <p class="fine">This copy cannot update itself (the portable zip, or a system without installer updates). Download the new version from the releases page.</p>
    {/if}
  </div>
  {#snippet footer()}
    <button type="button" onclick={onClose} disabled={busy}>Later</button>
    <button type="button" onclick={onPage} disabled={busy}>Release Page</button>
    {#if info.canInstall}
      <button type="button" class="primary" onclick={onInstall} disabled={busy}>{busy ? "Installing..." : "Install and Restart"}</button>
    {/if}
  {/snippet}
</Dialog>

<style>
  .text {
    display: grid;
    gap: 8px;
    user-select: text;
  }
  .text p {
    margin: 0;
    line-height: 1.45;
  }
  .version {
    color: var(--value);
  }
  .notes {
    max-height: 200px;
    overflow: auto;
    padding: 6px 8px;
    border-left: 2px solid var(--accent);
    background: var(--surface-1);
    white-space: pre-wrap;
  }
  .fine {
    color: var(--text-dim);
    font-size: var(--font-size-small);
  }
</style>
