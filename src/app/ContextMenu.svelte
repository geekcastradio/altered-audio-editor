<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Right-click menu. Opens at the pointer, stays on screen, closes on
  Escape, outside click, or after an item runs.
-->
<script lang="ts" module>
  export type ContextItem =
    | { kind: "item"; label: string; run: () => void; disabled?: boolean; shortcut?: string | undefined; checked?: boolean; reason?: string | null | undefined }
    | { kind: "sub"; label: string; items: readonly ContextItem[]; disabled?: boolean; reason?: string | null | undefined }
    | { kind: "sep" };
</script>

<script lang="ts">
  interface Props {
    x: number;
    y: number;
    items: readonly ContextItem[];
    onClose: () => void;
  }

  const { x, y, items, onClose }: Props = $props();

  let menu = $state<HTMLDivElement | null>(null);
  let left = $state(0);
  let top = $state(0);
  /**
   * The highlighted row at each open level. [3] is row 3 of the main menu;
   * [3, 1] is row 1 of the submenu opened by row 3. A trailing -1 means a
   * submenu is open with nothing highlighted yet.
   */
  let trail = $state<number[]>([]);

  $effect(() => {
    if (!menu) return;
    const r = menu.getBoundingClientRect();
    left = Math.max(0, Math.min(x, window.innerWidth - r.width - 2));
    top = Math.max(0, Math.min(y, window.innerHeight - r.height - 2));
    menu.focus();
  });

  function levelItems(depth: number): readonly ContextItem[] {
    let list: readonly ContextItem[] = items;
    for (let d = 0; d < depth; d++) {
      const it = list[trail[d] ?? -1];
      if (!it || it.kind !== "sub") return [];
      list = it.items;
    }
    return list;
  }

  const usable = (it: ContextItem) => it.kind !== "sep" && !it.disabled;

  function choose(it: ContextItem) {
    if (it.kind !== "item" || it.disabled) return;
    onClose();
    it.run();
  }

  function move(delta: number) {
    const depth = trail.length === 0 ? 0 : trail.length - 1;
    const list = levelItems(depth);
    const enabled = list.flatMap((it, i) => (usable(it) ? [i] : []));
    if (enabled.length === 0) return;
    const current = trail[depth] ?? -1;
    const at = enabled.indexOf(current);
    const next = at === -1 ? (delta > 0 ? 0 : enabled.length - 1) : (at + delta + enabled.length) % enabled.length;
    trail = [...trail.slice(0, depth), enabled[next]!];
  }

  function openSub(depth: number, index: number) {
    const it = levelItems(depth)[index];
    if (!it || it.kind !== "sub" || it.disabled) return;
    const first = it.items.findIndex(usable);
    trail = [...trail.slice(0, depth), index, first];
  }

  function onKey(e: KeyboardEvent) {
    e.stopPropagation();
    const depth = trail.length === 0 ? 0 : trail.length - 1;
    const here = levelItems(depth)[trail[depth] ?? -1];
    if (e.key === "Escape") {
      e.preventDefault();
      if (trail.length > 1) trail = trail.slice(0, -1);
      else onClose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      move(1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      move(-1);
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      if (here?.kind === "sub") openSub(depth, trail[depth]!);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      if (trail.length > 1) trail = trail.slice(0, -1);
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      if (here?.kind === "sub") openSub(depth, trail[depth]!);
      else if (here) choose(here);
    }
  }

  function onOutside(e: PointerEvent) {
    if (menu && !menu.contains(e.target as Node)) onClose();
  }

  /** Keeps a submenu on screen: flips it to the left edge or lifts it up when it would spill. */
  function fit(node: HTMLElement) {
    const r = node.getBoundingClientRect();
    if (r.right > window.innerWidth - 2) node.classList.add("flip");
    if (r.bottom > window.innerHeight - 2) node.style.top = `${Math.max(-node.offsetTop, window.innerHeight - 2 - r.bottom)}px`;
  }
</script>

<svelte:window onpointerdowncapture={onOutside} onblur={onClose} onresize={onClose} />

{#snippet list(rows: readonly ContextItem[], depth: number)}
  {#each rows as it, i (i)}
    {#if it.kind === "sep"}
      <div class="sep" role="separator"></div>
    {:else if it.kind === "sub"}
      <div class="row">
        <button
          type="button"
          role="menuitem"
          aria-haspopup="menu"
          aria-expanded={trail[depth] === i && trail.length > depth + 1}
          class:active={trail[depth] === i}
          disabled={it.disabled}
          title={it.disabled && it.reason ? it.reason : undefined}
          onpointerenter={() => (it.disabled ? (trail = trail.slice(0, depth)) : openSub(depth, i))}
          onclick={() => openSub(depth, i)}
        >
          <span class="label">{it.label}</span>
          <span class="arrow" aria-hidden="true">&#9656;</span>
        </button>
        {#if trail[depth] === i && trail.length > depth + 1}
          <div class="context sub" role="menu" use:fit>
            {@render list(it.items, depth + 1)}
          </div>
        {/if}
      </div>
    {:else}
      <button
        type="button"
        role={it.checked === undefined ? "menuitem" : "menuitemcheckbox"}
        aria-checked={it.checked}
        class:active={trail[depth] === i}
        disabled={it.disabled}
        title={it.disabled && it.reason ? it.reason : undefined}
        onpointerenter={() => (trail = it.disabled ? trail.slice(0, depth) : [...trail.slice(0, depth), i])}
        onclick={() => choose(it)}
      >
        {#if it.checked !== undefined}<span class="check" aria-hidden="true">{it.checked ? "✓" : ""}</span>{/if}
        <span class="label">{it.label}</span>
        {#if it.shortcut}<span class="key">{it.shortcut}</span>{/if}
      </button>
    {/if}
  {/each}
{/snippet}

<div
  class="context"
  role="menu"
  tabindex="-1"
  data-area-keep
  bind:this={menu}
  style:left="{left}px"
  style:top="{top}px"
  onkeydown={onKey}
  oncontextmenu={(e) => e.preventDefault()}
>
  {@render list(items, 0)}
</div>

<style>
  .check {
    width: 12px;
    color: var(--accent);
  }

  .check + .label {
    flex: 1;
    margin-left: 6px;
    text-align: left;
  }

  .row {
    position: relative;
  }

  .context.sub {
    width: max-content;
    position: absolute;
    top: -4px;
    left: 100%;
  }

  .context.sub:global(.flip) {
    left: auto;
    right: 100%;
  }

  .arrow {
    color: var(--text-dim);
  }

  .context {
    position: fixed;
    z-index: 6000;
    min-width: 190px;
    padding: 3px 0;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.55);
    outline: none;
  }

  button {
    white-space: nowrap;
    display: flex;
    justify-content: space-between;
    gap: 24px;
    width: 100%;
    height: 22px;
    padding: 0 12px 0 20px;
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    text-align: left;
  }

  button.active {
    background: var(--accent-dim);
  }

  button:disabled {
    color: var(--text-faint);
  }

  .key {
    color: var(--text-dim);
  }

  .sep {
    height: 1px;
    margin: 3px 0;
    background: var(--border);
  }
</style>
