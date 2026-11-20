<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  A tab for each open session, along the top of the Multitrack. Click a tab to
  bring that session forward; the x closes it (it asks about unsaved changes
  first, the same as closing it from the Files panel).
-->
<script lang="ts">
  import type { SessionRow } from "../files/session-row";
  import { tabLabel } from "./session-tabs";

  interface Props {
    rows: readonly SessionRow[];
    onSwitch: (id: string) => void;
    onClose: (id: string) => void;
  }

  const { rows, onSwitch, onClose }: Props = $props();
</script>

<div class="tabs" role="tablist" aria-label="Open sessions">
  {#each rows as row (row.id)}
    <div class="tab" class:front={row.front} role="presentation">
      <button type="button" role="tab" aria-selected={row.front} class="name" title={row.name} onclick={() => onSwitch(row.id)}>{tabLabel(row)}</button>
      <button type="button" class="x" aria-label="Close {row.name}" title="Close" onclick={() => onClose(row.id)}>&times;</button>
    </div>
  {/each}
</div>

<style>
  .tabs {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    flex: none;
    overflow-x: auto;
    padding: 6px 8px 0;
    border-bottom: 1px solid var(--border-strong);
    background: var(--gutter);
  }
  .tab {
    display: flex;
    align-items: center;
    min-height: 28px;
    max-width: 240px;
    border: 1px solid var(--border);
    border-bottom: none;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    background: var(--surface-2);
    color: var(--text-dim);
  }
  .tab:hover {
    background: var(--surface-3);
    color: var(--text);
  }
  .tab.front {
    position: relative;
    margin-bottom: -1px;
    border-color: var(--border-strong);
    border-top: 2px solid var(--accent);
    background: var(--surface-0);
    color: var(--text);
    font-weight: 600;
  }
  .name {
    min-width: 0;
    padding: 5px 6px 5px 14px;
    font-weight: inherit;
    border: none;
    background: none;
    color: inherit;
    font-size: var(--font-size-small);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
  }
  .x {
    padding: 0 10px 0 6px;
    border: none;
    background: none;
    color: var(--text-faint);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
  }
  .x:hover {
    color: var(--text);
  }
</style>
