<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Help > About.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";
  import logo from "../assets/logo-dark.png";

  interface Props {
    version: string | null;
    onClose: () => void;
  }

  const { version, onClose }: Props = $props();
</script>

<Dialog title="About Altered Audio Editor" width={420} {onClose}>
  <div class="text">
    <img class="logo" src={logo} alt="" width="372" height="134" />
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
  </div>
  {#snippet footer()}
    <button type="button" class="primary" onclick={onClose}>OK</button>
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
  .logo {
    display: block;
    width: 100%;
    height: auto;
    border-radius: 4px;
  }
  .version {
    color: var(--value);
  }
  .privacy {
    padding: 6px 8px;
    border-left: 2px solid var(--accent);
    background: var(--surface-1);
    color: var(--text-dim);
    font-size: var(--font-size-small);
  }
  .fine {
    color: var(--text-dim);
    font-size: var(--font-size-small);
  }
</style>
