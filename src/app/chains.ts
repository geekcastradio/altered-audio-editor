// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Process chains: a saved list of editing steps run one after another as a
 * single undo step. The file is read and checked in the core; this runs
 * what passed. Each step is an action with a handler that works out an edit
 * from the session as it is at that moment, so step two sees step one's result.
 */

import type { Command } from "../commands/types";
import type { Op, SessionDoc } from "../stores/session";

export interface ChainStep {
  action: string;
  params?: Record<string, unknown>;
  on_error?: "stop" | "skip";
}

export interface ChainFile {
  version: number;
  name: string;
  description?: string;
  steps: ChainStep[];
}

export interface ChainLoaded {
  chain: ChainFile;
  problems: { step: number | null; message: string }[];
  /** Per step, its settings with defaults filled in. */
  resolved: Record<string, unknown>[];
}

/** What a step worked out. `op` is null when there was nothing to do. */
export interface StepOutcome {
  op: Op | null;
  message: string;
}

export type StepHandler = (doc: SessionDoc, params: Record<string, unknown>) => Promise<StepOutcome> | StepOutcome;

export interface StepReport {
  action: string;
  title: string;
  status: "done" | "nothing" | "skipped";
  message: string;
}

export interface ChainDeps {
  doc: () => SessionDoc | null;
  /** Applies an op to the session and returns the op that undoes it. */
  apply: (op: Op) => Promise<Op>;
  handlers: Record<string, { title: string; run: StepHandler }>;
}

export class ChainError extends Error {}

/** Why a chain cannot run, or null when it can. Nothing has been changed at this point. */
export function preflight(loaded: ChainLoaded, handlers: ChainDeps["handlers"]): string | null {
  if (loaded.problems.length > 0) {
    const lines = loaded.problems.map((p) => (p.step === null ? p.message : `Step ${p.step + 1}: ${p.message}`));
    return `This chain has problems and was not run:\n${lines.join("\n")}`;
  }
  const missing = loaded.chain.steps.map((s) => s.action).filter((a) => !handlers[a]);
  if (missing.length > 0) return `The app cannot run these steps yet: ${[...new Set(missing)].join(", ")}.`;
  return null;
}

/**
 * The whole chain as one history entry. The first run works the steps out
 * and applies them. Undo applies the recorded inverses backwards; redo
 * applies the recorded edits again, in order.
 */
export class ChainCommand implements Command {
  readonly label: string;
  readonly reports: StepReport[] = [];
  private ops: Op[] = [];
  private inverses: Op[] = [];
  private ran = false;

  constructor(
    private readonly loaded: ChainLoaded,
    private readonly deps: ChainDeps,
  ) {
    this.label = `Chain: ${loaded.chain.name}`;
  }

  /** True when at least one step changed the session. */
  get changed(): boolean {
    return this.ops.length > 0;
  }

  async do(): Promise<void> {
    if (this.ran) {
      for (const op of this.ops) await this.deps.apply(op);
      return;
    }
    const { chain, resolved } = this.loaded;
    for (let i = 0; i < chain.steps.length; i++) {
      const step = chain.steps[i]!;
      const h = this.deps.handlers[step.action]!;
      try {
        const doc = this.deps.doc();
        if (!doc) throw new ChainError("There is no session open.");
        const out = await h.run(doc, resolved[i] ?? {});
        if (!out.op) {
          this.reports.push({ action: step.action, title: h.title, status: "nothing", message: out.message });
          continue;
        }
        const inverse = await this.deps.apply(out.op);
        this.ops.push(out.op);
        this.inverses.push(inverse);
        this.reports.push({ action: step.action, title: h.title, status: "done", message: out.message });
      } catch (e) {
        const why = e instanceof Error ? e.message : String(e);
        if (step.on_error === "skip") {
          this.reports.push({ action: step.action, title: h.title, status: "skipped", message: why });
          continue;
        }
        await this.rollBack();
        throw new ChainError(`Step ${i + 1} (${h.title}) stopped the chain: ${why} Nothing was changed.`);
      }
    }
    this.ran = true;
  }

  async undo(): Promise<void> {
    await this.rollBack();
  }

  private async rollBack(): Promise<void> {
    for (const inv of [...this.inverses].reverse()) await this.deps.apply(inv);
    if (!this.ran) {
      this.ops = [];
      this.inverses = [];
    }
  }
}

/** One notice for the whole run. */
export function chainSummary(name: string, reports: StepReport[]): string {
  const done = reports.filter((r) => r.status === "done").length;
  const head = done === 0 ? `Chain "${name}" ran and changed nothing.` : `Chain "${name}" ran ${done} of ${reports.length} steps. One undo takes it all back.`;
  const lines = reports.map((r) => `${r.title}: ${r.status === "skipped" ? `skipped (${r.message})` : r.message}`);
  return [head, ...lines].join(" ");
}
