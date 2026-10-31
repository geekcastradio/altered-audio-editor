<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Shown once, the first time the program runs.
-->
<script lang="ts">
  interface Props {
    onGuide: () => void;
    onClose: () => void;
  }

  const { onGuide, onClose }: Props = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div class="dialog" role="dialog" tabindex="-1" aria-modal="true" aria-labelledby="welcome-title" onclick={(e) => e.stopPropagation()}>
    <h1 id="welcome-title">Welcome to Altered Audio Editor</h1>
    <p>A free audio editor for podcasters and voiceover. Your first episode:</p>
    <ol>
      <li>File > New Session, then File > Import Files (or drag files in).</li>
      <li>Space bar plays and stops. Clip > Split at Playhead cuts.</li>
      <li>Add effects in the Voice Chain (Window > Voice Chain).</li>
      <li>File > Export > Session Mixdown when it sounds right.</li>
    </ol>
    <p class="fine">No account, no tracking. Your audio stays on your computer. You can reopen the guide any time from Help > User Guide.</p>
    <div class="actions">
      <button type="button" onclick={onGuide}>Open the User Guide</button>
      <button type="button" class="primary" onclick={onClose}>Start Editing</button>
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
    width: 440px;
    padding: 20px 24px 16px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  }
  h1 {
    margin: 0 0 10px;
    font-size: 17px;
  }
  p,
  li {
    margin: 0 0 8px;
    line-height: 1.45;
  }
  ol {
    margin: 0 0 10px;
    padding-left: 22px;
  }
  .fine {
    color: var(--text-dim);
    font-size: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 12px;
  }
</style>
