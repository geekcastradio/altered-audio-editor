<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Show Appearance: the show badge's color and cover picture in one small
  popup that hangs under the badge. Pick a swatch or any color, choose a
  picture from the library, choose a file, or drop a picture on the popup. Click anywhere else or press Escape to close it.
-->
<script lang="ts">
  import { popoverPosition } from "./cover";

  interface Picture {
    id: string;
    name: string;
    path: string;
    /** This picture is the current cover. */
    on: boolean;
  }

  interface Props {
    /** The badge's left edge, top and bottom, in window pixels. */
    anchor: { left: number; top: number; bottom: number };
    /** A palette token like "--clip-3", "#rrggbb", or null for the automatic color. */
    color: string | null;
    /** Path of the current cover picture, or null. */
    coverPath: string | null;
    /** The library's own pictures, to pick a cover from. */
    pictures: readonly Picture[];
    /** Thumbnails already loaded, by file path. */
    thumbs: Record<string, string>;
    /** Svelte action that loads a thumbnail for a path. */
    thumb: (node: HTMLElement, path: string) => { update: (path: string) => void };
    /** A file is being dragged over the cover box. */
    over?: boolean;
    /** The popup itself, so the panel can register it as a drop target for pictures. */
    zone?: HTMLElement | null;
    onColor: (color: string | null) => void;
    onPicture: (id: string) => void;
    onChoose: () => void;
    onRemove: () => void;
    onClose: () => void;
  }

  let {
    anchor,
    color,
    coverPath,
    pictures,
    thumbs,
    thumb,
    over = false,
    zone = $bindable(null),
    onColor,
    onPicture,
    onChoose,
    onRemove,
    onClose,
  }: Props = $props();

  const SWATCHES = ["--clip-1", "--clip-2", "--clip-3", "--clip-4", "--clip-5", "--clip-6"];
  const custom = $derived(color?.startsWith("#") ? color : "#7fa7d9");

  let width = $state(280);
  let height = $state(340);
  let view = $state({ w: window.innerWidth, h: window.innerHeight });
  const pos = $derived(popoverPosition(anchor, { w: width, h: height }, view));

  /** Moves the popup to the end of the page so no panel can clip or shift it. */
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return {
      destroy() {
        node.remove();
      },
    };
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} onresize={() => (view = { w: window.innerWidth, h: window.innerHeight })} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="shade" use:portal onclick={onClose}></div>
<div
  class="pop"
  use:portal
  role="dialog"
  aria-label="Show appearance"
  tabindex="-1"
  bind:this={zone}
  style:left="{pos.left}px"
  style:top="{pos.top}px"
  bind:offsetWidth={width}
  bind:offsetHeight={height}
>
  <h1>Color</h1>
  <div class="swatches">
    {#each SWATCHES as c (c)}
      <button type="button" class:on={color === c} style:background="var({c})" aria-label="Color {c.slice(-1)}" onclick={() => onColor(c)}></button>
    {/each}
  </div>
  <div class="row">
    <label class="custom">
      <span>Other color</span>
      <input type="color" value={custom} onchange={(e) => onColor(e.currentTarget.value)} />
    </label>
    <button type="button" class="text" class:on={color === null} onclick={() => onColor(null)}>Automatic</button>
  </div>

  <h1>Cover</h1>
  <div class="zone" class:over>
    <span class="preview">
      {#if coverPath}
        {#key coverPath}
          <span class="thumb" use:thumb={coverPath}>{#if thumbs[coverPath]}<img src={thumbs[coverPath]} alt="" draggable="false" />{/if}</span>
        {/key}
      {:else}
        <span class="none">No cover</span>
      {/if}
    </span>
    <span class="hint">{over ? "Drop to use as the cover" : "Drop a JPEG or PNG here"}</span>
  </div>
  <div class="row">
    <button type="button" class="text" onclick={onChoose}>Choose Picture...</button>
    <button type="button" class="text" disabled={!coverPath} onclick={onRemove}>Remove Cover</button>
  </div>

  {#if pictures.length > 0}
    <h1>From this library</h1>
    <div class="pics">
      {#each pictures as p (p.id)}
        <button type="button" class="pic" class:on={p.on} title={p.name} aria-label={p.name} onclick={() => onPicture(p.id)}>
          {#key p.path}
            <span class="thumb" use:thumb={p.path}>{#if thumbs[p.path]}<img src={thumbs[p.path]} alt="" draggable="false" />{/if}</span>
          {/key}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .shade {
    position: fixed;
    inset: 0;
    z-index: 5000;
  }

  .pop {
    position: fixed;
    z-index: 5001;
    box-sizing: border-box;
    display: grid;
    gap: 8px;
    width: 280px;
    max-height: calc(100vh - 16px);
    overflow-y: auto;
    padding: 12px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.55);
    font-size: var(--font-size-small);
  }

  h1 {
    margin: 4px 0 0;
    color: var(--text-dim);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  h1:first-child {
    margin-top: 0;
  }

  .swatches {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 6px;
  }

  .swatches button {
    height: 24px;
    border: 2px solid transparent;
    border-radius: 3px;
  }

  .swatches button:hover,
  .swatches button.on {
    border-color: #fff;
  }

  .row {
    display: flex;
    gap: 6px;
    align-items: center;
    justify-content: space-between;
  }

  .custom {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .custom input {
    width: 44px;
    height: 22px;
    padding: 0;
    border: 1px solid var(--border-strong);
    background: none;
  }

  .text {
    height: 22px;
    padding: 0 10px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
  }

  .text:hover:not(:disabled),
  .text.on {
    border-color: var(--accent);
  }

  .text:disabled {
    opacity: 0.45;
  }

  .zone {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 8px;
    border: 1px dashed var(--border-strong);
    border-radius: 4px;
    color: var(--text-dim);
  }

  .zone.over {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 14%, transparent);
    color: var(--text);
  }

  .preview {
    flex: none;
    display: grid;
    place-items: center;
    width: 56px;
    height: 56px;
    overflow: hidden;
    border-radius: 6px;
    background: var(--surface-3);
  }

  .none {
    font-size: 10px;
  }

  .thumb,
  .thumb img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .pics {
    display: grid;
    grid-template-columns: repeat(auto-fill, 40px);
    gap: 6px;
    max-height: 104px;
    overflow-y: auto;
  }

  .pic {
    width: 40px;
    height: 40px;
    padding: 0;
    overflow: hidden;
    border: 2px solid transparent;
    border-radius: 4px;
    background: var(--surface-3);
  }

  .pic:hover,
  .pic.on {
    border-color: var(--accent);
  }
</style>
