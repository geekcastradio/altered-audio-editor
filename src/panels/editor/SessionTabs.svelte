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
    gap: 1px;
    flex: none;
    overflow-x: auto;
    border-bottom: 1px solid var(--border);
    background: var(--surface-1);
  }
  .tab {
    display: flex;
    align-items: center;
    max-width: 220px;
    border-right: 1px solid var(--border);
    color: var(--text-dim);
  }
  .tab.front {
    background: var(--surface-0);
    color: var(--text);
    box-shadow: inset 0 -2px 0 var(--accent);
  }
  .name {
    min-width: 0;
    padding: 4px 4px 4px 10px;
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
    padding: 0 7px;
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
