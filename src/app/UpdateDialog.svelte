<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Help > Check for Updates: a newer version was found. Nothing installs
  until Install and Restart is clicked.
-->
<script lang="ts">
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

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={() => !busy && onClose()}>
  <div class="dialog" role="dialog" tabindex="-1" aria-modal="true" aria-labelledby="update-title" onclick={(e) => e.stopPropagation()}>
    <h1 id="update-title">Version {info.version} is available</h1>
    <p class="version">You have version {info.current}.</p>
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    {#if info.notes}<div class="notes" tabindex="0" aria-label="What changed">{plainNotes(info.notes)}</div>{/if}
    {#if info.canInstall}
      <p class="fine">Save your work first. The update downloads, installs and restarts the app. Unsaved sessions are offered a recovery copy, but saving is safer.</p>
    {:else}
      <p class="fine">This copy cannot update itself (the portable zip, or a system without installer updates). Download the new version from the releases page.</p>
    {/if}
    <div class="actions">
      <button type="button" onclick={onClose} disabled={busy}>Later</button>
      <button type="button" onclick={onPage} disabled={busy}>Release Page</button>
      {#if info.canInstall}
        <button type="button" class="primary" onclick={onInstall} disabled={busy}>{busy ? "Installing..." : "Install and Restart"}</button>
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
    --dialog-pad-x: 24px;
    --dialog-pad-y: 18px;
    width: 440px;
    padding: 18px 24px 16px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
    user-select: text;
  }

  h1 {
    margin: 0 0 2px;
    font-size: 16px;
    font-weight: 600;
  }

  p {
    margin: 8px 0;
    line-height: 1.45;
  }

  .version {
    margin-top: 0;
    color: var(--value);
  }

  .notes {
    max-height: 200px;
    margin: 0 0 10px;
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

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
    margin-top: 14px;
  }

  button {
    min-width: 72px;
    height: 24px;
    padding: 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
  }

  button:hover:not(:disabled) {
    border-color: var(--accent);
  }

  button:disabled {
    opacity: 0.5;
  }

  .primary {
    border-color: var(--accent);
  }
</style>
