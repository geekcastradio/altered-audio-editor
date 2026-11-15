<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Soundboard panel. Nine pads, each holding one audio file. Pressing a pad
  puts its file on the session at the playhead, on the selected track, as
  one undoable step. An empty pad asks for a file. Give a pad a key in
  Edit > Keyboard Shortcuts (Multitrack > Soundboard).
-->
<script lang="ts">
  import { PAD_COUNT, type Pad, type PadsStore } from "./soundboard";

  interface Props {
    pads: PadsStore;
    onPress: (index: number) => void;
    onAssign: (index: number) => void;
    onClear: (index: number) => void;
  }

  const { pads, onPress, onAssign, onClear }: Props = $props();

  let list = $state<Pad[]>([]);
  $effect(() =>
    pads.subscribe((p) => {
      list = p;
    }),
  );
</script>

<section class="board" aria-label="Soundboard">
  <p class="hint">Press a pad to put its sound on the session at the playhead. Pads are saved with the program, not with a session.</p>
  <div class="grid">
    {#each list.slice(0, PAD_COUNT) as pad, i (i)}
      <div class="cell" class:empty={!pad.path}>
        <button type="button" class="pad" title={pad.path ?? "Choose a sound for this pad"} onclick={() => (pad.path ? onPress(i) : onAssign(i))}>
          <span class="n">{i + 1}</span>
          <span class="name">{pad.path ? pad.label : "Empty"}</span>
        </button>
        {#if pad.path}
          <span class="tools">
            <button type="button" title="Choose a different sound" onclick={() => onAssign(i)}>Change</button>
            <button type="button" title="Empty this pad" onclick={() => onClear(i)}>Clear</button>
          </span>
        {/if}
      </div>
    {/each}
  </div>
</section>

<style>
  .board {
    container-type: inline-size;
    height: 100%;
    overflow: auto;
    padding: 8px;
    background: var(--surface-1);
    color: var(--text);
  }

  .hint {
    margin: 0 0 8px;
    color: var(--text-faint);
    font-size: var(--font-size-small);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 6px;
  }

  .cell {
    display: grid;
    gap: 2px;
  }

  .pad {
    display: grid;
    align-content: center;
    gap: 2px;
    min-height: 54px;
    padding: 6px;
    border: 1px solid var(--border-strong);
    border-radius: 3px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
    text-align: left;
  }

  .empty .pad {
    color: var(--text-faint);
  }

  .n {
    font-size: var(--font-size-small);
    color: var(--text-faint);
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tools {
    display: flex;
    gap: 4px;
  }

  .tools button {
    flex: 1;
    height: 18px;
    padding: 0 4px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-2);
    color: var(--text);
    font: inherit;
    font-size: var(--font-size-small);
  }
</style>
