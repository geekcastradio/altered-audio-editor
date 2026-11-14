<!--
  SPDX-License-Identifier: GPL-3.0-or-later
  Copyright (c) The GeekCast Radio Network, LLC.
  This file is part of Altered Audio Editor.

  The one shell every dialog uses: backdrop, panel, title bar, Escape to
  close, Enter in a field to run the main action, and the first field
  focused when it opens. The look is in src/styles/dialog.css. Put the
  buttons in the footer snippet, Cancel first and the main action last.
-->
<script lang="ts">
  import { onMount, type Snippet } from "svelte";

  interface Props {
    title: string;
    onClose: () => void;
    /** Pixels. The dialog never goes wider than the window. */
    width?: number;
    /** Enter in a text field runs this. Leave it out when Enter should do nothing. */
    onEnter?: () => void;
    /** Sits above other dialogs, for one that another dialog opens. */
    raised?: boolean;
    /** Pixels for the label column of .row lines. Default 96. */
    labelWidth?: number;
    /** False while work is running that must not be walked away from: Escape and a click outside do nothing. */
    canClose?: boolean;
    children: Snippet;
    footer?: Snippet;
  }

  const { title, onClose, width = 440, onEnter, raised = false, labelWidth = 96, canClose = true, children, footer }: Props = $props();

  const uid = $props.id();
  const titleId = `${uid}-title`;
  let panel = $state<HTMLDivElement | null>(null);
  let pressedOnBackdrop = false;

  /** Only the dialog on top answers the keyboard, so one opened over another leaves the first alone. */
  const onTop = () => {
    const open = document.querySelectorAll('[aria-modal="true"]');
    return open.length > 0 && open[open.length - 1] === panel;
  };

  onMount(() => {
    const el = panel;
    if (!el) return;
    // A dialog can name the control it wants first with data-autofocus.
    const field = el.querySelector<HTMLElement>("[data-autofocus]:not(:disabled)") ?? el.querySelector<HTMLElement>("input:not([type=hidden]):not(:disabled), select:not(:disabled), textarea:not(:disabled)");
    const target = field ?? el.querySelector<HTMLElement>("footer .primary:not(:disabled)") ?? el;
    target.focus();
    if (target instanceof HTMLInputElement && (target.type === "text" || target.type === "number")) target.select();
  });

  function key(e: KeyboardEvent) {
    if (!onTop()) return;
    if (e.key === "Escape") {
      e.preventDefault();
      if (canClose) onClose();
    } else if (e.key === "Enter" && onEnter && !e.isComposing && !e.shiftKey) {
      const t = e.target;
      const text = t instanceof HTMLInputElement && !["checkbox", "radio", "button", "submit", "range"].includes(t.type);
      if (text) {
        e.preventDefault();
        onEnter();
      }
    }
  }
</script>

<svelte:window onkeydown={key} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div
  class="dlg-backdrop"
  style:z-index={raised ? 5100 : undefined}
  onpointerdown={(e) => (pressedOnBackdrop = e.target === e.currentTarget)}
  onclick={(e) => {
    // A drag that starts inside the dialog and ends outside must not close it.
    if (pressedOnBackdrop && e.target === e.currentTarget && canClose) onClose();
    pressedOnBackdrop = false;
  }}
>
  <div class="dialog dlg" role="dialog" aria-modal="true" aria-labelledby={titleId} tabindex="-1" style:width="min({width}px, 94vw)" style:--dlg-label="{labelWidth}px" bind:this={panel}>
    <h1 id={titleId}>{title}</h1>
    {@render children()}
    {#if footer}<footer>{@render footer()}</footer>{/if}
  </div>
</div>
