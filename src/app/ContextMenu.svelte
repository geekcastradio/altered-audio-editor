<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Right-click menu. Opens at the pointer, stays on screen, closes on
  Escape, outside click, or after an item runs. Typing letters jumps to
  the first row in the open list that starts with them. A list that
  starts with a search row shows a search box instead: what is typed
  there lists the matching rows from anywhere in the menu.
-->
<script lang="ts" module>
  export type ContextItem =
    | { kind: "item"; label: string; run: () => void; disabled?: boolean; shortcut?: string | undefined; checked?: boolean; reason?: string | null | undefined }
    | { kind: "sub"; label: string; items: readonly ContextItem[]; disabled?: boolean; reason?: string | null | undefined }
    | { kind: "sep" }
    /** A search box at the top of the menu (first row only); `find` lists the rows matching the text typed. */
    | { kind: "search"; placeholder: string; find: (query: string) => readonly ContextItem[] };
</script>

<script lang="ts">
  import { isTypingKey, typeahead } from "./typeahead";

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
  /** What is typed in the search box, if the menu has one. */
  let query = $state("");
  let searchBox = $state<HTMLInputElement | null>(null);
  /** The top list: the menu's rows, or the search row and what it found. */
  const root = $derived.by((): readonly ContextItem[] => {
    const s = items[0];
    if (s?.kind !== "search" || !query.trim()) return items;
    const found = s.find(query.trim());
    return [s, ...(found.length ? found : [{ kind: "item", label: "No matches", disabled: true, run: () => {} } as ContextItem])];
  });

  $effect(() => {
    if (!menu) return;
    const r = menu.getBoundingClientRect();
    left = Math.max(0, Math.min(x, window.innerWidth - r.width - 2));
    top = Math.max(0, Math.min(y, window.innerHeight - r.height - 2));
    if (searchBox) searchBox.focus();
    else menu.focus();
  });

  function levelItems(depth: number): readonly ContextItem[] {
    let list: readonly ContextItem[] = root;
    for (let d = 0; d < depth; d++) {
      const it = list[trail[d] ?? -1];
      if (!it || it.kind !== "sub") return [];
      list = it.items;
    }
    return list;
  }

  const usable = (it: ContextItem) => it.kind !== "sep" && it.kind !== "search" && !it.disabled;

  /**
   * The trail when the pointer is on a disabled row at `depth`: nothing
   * highlighted there, but the submenu it sits in stays open (it used to
   * close, so a disabled row in a submenu made the whole submenu vanish).
   */
  function keepOpen(depth: number): number[] {
    return depth === 0 ? [] : [...trail.slice(0, depth), -1];
  }

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

  const jump = typeahead();

  // The highlighted row stays in view when keys move it down a long list.
  $effect(() => {
    void trail;
    queueMicrotask(() => {
      const rows = menu?.querySelectorAll<HTMLElement>("button.active");
      rows?.[rows.length - 1]?.scrollIntoView({ block: "nearest" });
    });
  });

  function onKey(e: KeyboardEvent) {
    e.stopPropagation();
    const depth = trail.length === 0 ? 0 : trail.length - 1;
    const here = levelItems(depth)[trail[depth] ?? -1];
    if (e.target === searchBox) {
      // Typing goes into the box; the arrows, Enter and Escape still work the list.
      if (e.key === "Escape" && query) {
        e.preventDefault();
        query = "";
        trail = [];
        return;
      }
      if (e.key === "Enter" && trail.length <= 1 && (trail[0] ?? -1) < 0) {
        e.preventDefault();
        const first = root.find(usable);
        if (first) choose(first);
        return;
      }
      if (!["ArrowDown", "ArrowUp", "Enter", "Escape"].includes(e.key)) return;
    } else if (isTypingKey(e)) {
      const list = levelItems(depth);
      const to = jump.next(e.key, list.map((it) => (usable(it) && it.kind !== "sep" && it.kind !== "search" ? it.label : null)), trail[depth] ?? -1);
      if (to !== null) {
        e.preventDefault();
        trail = [...trail.slice(0, depth), to];
      }
      return;
    }
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

  /**
   * Places a submenu beside its row, on screen: to the left when it would
   * spill off the right, lifted when it would run off the bottom. A menu
   * taller than the window scrolls (see the max-height in the styles).
   * Submenus are placed against the window, not their parent, so a
   * scrolling parent never cuts them off.
   */
  function fit(node: HTMLElement) {
    const row = node.parentElement?.getBoundingClientRect();
    if (!row) return;
    const r = node.getBoundingClientRect();
    const w = window.innerWidth;
    const h = window.innerHeight;
    const left = row.right + r.width > w - 2 ? Math.max(2, row.left - r.width) : row.right;
    const height = Math.min(r.height, h - 4);
    const top = Math.max(2, Math.min(row.top - 4, h - 2 - height));
    node.style.left = `${left}px`;
    node.style.top = `${top}px`;
  }

  /** A menu level scrolled: its open submenu no longer sits beside its row, so close it. */
  function scrolled(depth: number) {
    if (trail.length > depth + 1) trail = trail.slice(0, depth + 1);
  }
</script>

<svelte:window onpointerdowncapture={onOutside} onblur={onClose} onresize={onClose} />

{#snippet list(rows: readonly ContextItem[], depth: number)}
  {#each rows as it, i (i)}
    {#if it.kind === "search"}
      {#if depth === 0}
        <div class="search">
          <input
            type="search"
            spellcheck="false"
            placeholder={it.placeholder}
            aria-label={it.placeholder}
            bind:this={searchBox}
            value={query}
            oninput={(e) => {
              query = e.currentTarget.value;
              trail = [];
            }}
          />
        </div>
      {/if}
    {:else if it.kind === "sep"}
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
          onpointerenter={() => (it.disabled ? (trail = keepOpen(depth)) : openSub(depth, i))}
          onclick={() => openSub(depth, i)}
        >
          <span class="label">{it.label}</span>
          <span class="arrow" aria-hidden="true">&#9656;</span>
        </button>
        {#if trail[depth] === i && trail.length > depth + 1}
          <div class="context sub" role="menu" use:fit onscroll={() => scrolled(depth + 1)}>
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
        onpointerenter={() => (trail = it.disabled ? keepOpen(depth) : [...trail.slice(0, depth), i])}
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
  onscroll={() => scrolled(0)}
  oncontextmenu={(e) => e.preventDefault()}
>
  {@render list(root, 0)}
</div>

<style>
  /* The check mark sits in the left padding, so every label lines up whether or not it has one. */
  .check {
    position: absolute;
    left: 5px;
    width: 12px;
    color: var(--menu-hover);
  }
  button.active .check {
    color: #fff;
  }

  .row {
    position: relative;
  }

  .search {
    position: sticky;
    top: -3px;
    z-index: 1;
    padding: 3px 6px 4px;
    background: inherit;
  }

  .search input {
    width: 100%;
    box-sizing: border-box;
    height: 22px;
    padding: 0 6px;
    border: 1px solid #9a9da6;
    border-radius: 3px;
    background: #fff;
    color: var(--menu-text);
    font: inherit;
  }

  .context.sub {
    width: max-content;
  }

  .arrow {
    color: var(--menu-dim);
  }

  .context {
    position: fixed;
    z-index: 6000;
    min-width: 190px;
    padding: 3px 0;
    /* A light menu, as in Audition, so it stands out from the dark panels. */
    --menu-bg: #e4e5e9;
    --menu-text: #16171a;
    --menu-dim: #4c4f57;
    --menu-faint: #8b8e97;
    --menu-hover: #2f6fd0;
    border: 1px solid #f5f6f8;
    background: var(--menu-bg);
    color: var(--menu-text);
    box-shadow: 0 0 0 1px #000, 0 8px 22px rgba(0, 0, 0, 0.7);
    outline: none;
    /* Never taller than the window: a long list scrolls instead. */
    max-height: calc(100vh - 4px);
    overflow-y: auto;
    box-sizing: border-box;
  }

  button {
    position: relative;
    white-space: nowrap;
    display: flex;
    justify-content: space-between;
    gap: 24px;
    width: 100%;
    height: 22px;
    padding: 0 12px 0 20px;
    border: 0;
    background: none;
    color: var(--menu-text);
    font: inherit;
    text-align: left;
  }

  button.active {
    background: var(--menu-hover);
    color: #fff;
  }

  button.active .key {
    color: #dbe6fb;
  }

  button:disabled {
    color: var(--menu-faint);
  }

  .key {
    color: var(--menu-dim);
  }

  .sep {
    height: 1px;
    margin: 3px 0;
    background: #b4b7bf;
  }
</style>
