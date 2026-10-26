<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Help > About.
-->
<script lang="ts">
  import logo from "../assets/logo-dark.png";

  interface Props {
    version: string | null;
    onClose: () => void;
  }

  const { version, onClose }: Props = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onClose()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onClose}>
  <div class="dialog" role="dialog" tabindex="-1" aria-modal="true" aria-labelledby="about-title" onclick={(e) => e.stopPropagation()}>
    <img class="logo" src={logo} alt="" width="372" height="134" />
    <h1 id="about-title" class="visually-hidden">Altered Audio Editor</h1>
    <p class="version">Version {version ?? "unknown"}</p>
    <p>A free, open-source multitrack audio editor for podcasters and voiceover.</p>
    <p class="privacy">
      No account, no sign-in, no telemetry, no tracking. The app goes online only when you pick Help > Check for Updates or a link
      in the Help menu, or if you turn on the weekly update check (off by default). Your audio and sessions stay on your computer.
    </p>
    <p>Copyright (c) The GeekCast Radio Network, LLC.</p>
    <p class="fine">
      Licensed under the GNU General Public License v3.0 or later. This program comes with no warranty.
      You may redistribute it under the terms of that license.
    </p>
    <p class="fine">Not affiliated with or endorsed by Adobe.</p>
    <div class="actions">
      <button type="button" onclick={onClose}>OK</button>
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
    width: 420px;
    padding: 20px 24px 16px;
    border: 1px solid var(--border-strong);
    background: var(--surface-2);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
    user-select: text;
  }

  .logo {
    display: block;
    width: 100%;
    height: auto;
    margin: -8px 0 4px;
    border-radius: 4px;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
  }

  .privacy {
    padding: 6px 8px;
    border-left: 2px solid var(--accent);
    background: var(--surface-1);
    color: var(--text-dim);
    font-size: var(--font-size-small);
  }

  h1 {
    margin: 0 0 2px;
    font-size: 16px;
    font-weight: 600;
  }

  .version {
    margin-top: 0;
    color: var(--value);
  }

  p {
    margin: 8px 0;
    line-height: 1.45;
  }

  .fine {
    color: var(--text-dim);
    font-size: var(--font-size-small);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 14px;
  }

  button {
    min-width: 72px;
    height: 24px;
    border: 1px solid var(--border-strong);
    border-radius: 2px;
    background: var(--surface-3);
    color: var(--text);
    font: inherit;
  }

  button:hover {
    border-color: var(--accent);
  }
</style>
