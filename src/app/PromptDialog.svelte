<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  Asks for one line of text, such as a name. Enter confirms, Escape cancels.
-->
<script lang="ts">
  import Dialog from "./Dialog.svelte";

  interface Props {
    title: string;
    label: string;
    value?: string;
    confirm?: string;
    onDone: (value: string | null) => void;
  }

  const { title, label, value = "", confirm = "OK", onDone }: Props = $props();

  // svelte-ignore state_referenced_locally
  let text = $state(value);
</script>

<Dialog {title} width={380} onClose={() => onDone(null)} onEnter={() => text.trim() && onDone(text.trim())}>
  <label class="stack">
    <span>{label}</span>
    <input type="text" spellcheck="false" bind:value={text} />
  </label>
  {#snippet footer()}
    <button type="button" onclick={() => onDone(null)}>Cancel</button>
    <button type="button" class="primary" disabled={!text.trim()} onclick={() => onDone(text.trim())}>{confirm}</button>
  {/snippet}
</Dialog>
