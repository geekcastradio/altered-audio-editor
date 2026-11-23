# Working with more than one AI assistant

Written 2026-11-23. The aim: Claude, Gemini and Copilot help at the same time without overwriting each other. It is a hub and spoke: one assistant integrates, the others hand it small, finished, checkable pieces.

## Roles

| Tool | Does | Does not |
| --- | --- | --- |
| **Claude** (integrator) | Merges, syncs branches, fixes CI, builds the release (never publishes), writes hand-test lists, does work that touches the engine, the command stack or many files. | Take a task another tool has claimed. |
| **Gemini** (scout and reviewer) | Reads the whole repo and writes specs and audits into `docs/` (for example "which files change for X"), reviews pull requests against `.gemini/styleguide.md`, and can build small self-contained modules on a `gemini/` branch. | Merge, bump versions, or edit shared files (`lib.rs`, `decisions.md`, menus) in the same pull request as a spec. |
| **Copilot** (builder for small, self-contained tasks) | Takes issues for Rust ports with shared fixtures, docs and small bug fixes, on a `copilot/` branch, and opens a pull request. Inline completions while you edit. | Touch audio devices, plug-ins or the docking layout without a person reviewing first. |
| **You** | Hand-test, merge anything the integrator has not, tag and publish releases, decide scope and shortcuts. | |

## Rules that stop the mess

1. **One task, one owner, one branch.** Write who has a task next to it in `docs/next-work.md` (add "Claimed by: tool, branch") before starting. Do not start a claimed task.
2. **Separate branches, each from the latest `main`.** Names start with the tool: `claude/`, `gemini/`, `copilot/`.
3. **Only one agent merges to `main`: the integrator (Claude), or you.** Merge only when all five checks are green. Other tools open pull requests and stop.
4. **Shared files are the usual cause of conflicts.** `docs/decisions.md` (everyone appends), `src-tauri/src/lib.rs` (the alphabetical `pub mod` list), `src/app/menus/model.ts` and `CLAUDE.md`. Keep edits to them to a few lines, add yours at the end or in alphabetical place, and after any merge to `main` the integrator merges `main` into every open branch. If a conflict in `decisions.md` is only two appended entries, keep both.
5. **Specs are tracked, not local.** Put plans in `docs/next-work.md` or a pull request description, not in a git-ignored folder, so every tool and you can read them.
6. **Do not edit the same folder at the same time.** On your computer, give each tool its own working copy (see below). In the cloud, each tool already has its own.
7. **Never touch versions, tags or releases** except the integrator starting the Release workflow. You publish.
8. **Private files:** `CLAUDE.local.md` and `private/` are never copied into tracked files.
9. **A failed run is deleted once a green run replaces it** on the same branch (project lead's rule, 2026-11-23). Failed commits stay in git history; do not rewrite history on `main` or on a branch someone else uses.

## A feature, start to finish

1. **Spec** (Gemini or Claude): write what it should do, "done when" checks, which files it will touch, and which tests prove it. Put it in `docs/next-work.md`. Keep it behavior only.
2. **Claim** it: add "Claimed by" and the branch name.
3. **Build** (the claiming tool): branch from `main`, small commits, run the checks in `AGENTS.md`, open a pull request that links the spec.
4. **Review** (a different tool): Gemini reviews Claude's or Copilot's pull request against the style guide; Claude reviews Gemini's or Copilot's code. The reviewer writes comments; the author fixes them.
5. **Merge** (Claude or you) on five green checks. The integrator then syncs the other open branches.
6. **Hand test** (you) for anything touching audio, devices or the look.

## Two working copies on your computer (Windows)

Use a Git worktree so two tools never share one folder:

```
cd C:\dev\altered-audio
git worktree add ..\altered-audio-gemini -b gemini/audit-dsp
```

Run one tool in each folder. Cautions from this project:

- Each copy builds its own Rust `target` folder: several GB of disk and a long first build. Do not run two Rust builds and two dev servers at once if memory is tight.
- Delete a worktree when its branch is merged: `git worktree remove ..\altered-audio-gemini`.
- Do not point two copies at one shared `target` folder; they will wait on each other's build lock.

## What the plan Gemini gave you gets right, and what to change

- Right: one rules file for all tools (`AGENTS.md`, done), one gatekeeper for `main`, specs handed to the builder, worktrees for parallel work.
- Change: specs live in `docs/next-work.md`, not a git-ignored `docs/plans/`. A local-only spec is invisible to the other tools and to you on another device.
- Change: `CLAUDE.md` stays as it is (it holds private notes and the full rules). `GEMINI.md` and `.github/copilot-instructions.md` already point to `AGENTS.md`, so a `.gemini/settings.json` is optional.
- Skip for now: a local language model on your PC for tests and boilerplate. It adds one more thing to review and the tests it writes still have to be checked.
- Check: the quota numbers in the plan (1,000 requests a day and so on) come from Gemini, not from the project. Check the limits shown in each tool before relying on them. GitHub's Copilot coding agent, which takes issues and opens pull requests, may need a paid Copilot plan; the free plan's inline completions in the editor do not do that.
