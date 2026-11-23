// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The words around running a process chain: why a chain cannot run, and the one notice
//! after a run. The Rust copy of `preflight` and `chainSummary` in `src/app/chains.ts`,
//! tested against `shared-fixtures/chain-run-notes.json`. The command that runs the steps
//! stays in TypeScript.

/// A problem the core found in a chain file. `step` is None for the file as a whole.
#[derive(Debug, Clone)]
pub struct Problem {
    pub step: Option<usize>,
    pub message: String,
}

/// Why a chain cannot run, or None when it can. `steps` are the step actions in order;
/// `known` are the actions the app has a handler for.
pub fn preflight(problems: &[Problem], steps: &[String], known: &[String]) -> Option<String> {
    if !problems.is_empty() {
        let lines: Vec<String> = problems
            .iter()
            .map(|p| match p.step {
                None => p.message.clone(),
                Some(s) => format!("Step {}: {}", s + 1, p.message),
            })
            .collect();
        return Some(format!(
            "This chain has problems and was not run:\n{}",
            lines.join("\n")
        ));
    }
    let mut missing: Vec<&str> = Vec::new();
    for a in steps {
        if !known.contains(a) && !missing.contains(&a.as_str()) {
            missing.push(a);
        }
    }
    if missing.is_empty() {
        None
    } else {
        Some(format!(
            "The app cannot run these steps yet: {}.",
            missing.join(", ")
        ))
    }
}

/// How a step ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Done,
    Nothing,
    Skipped,
}

/// What one step reported.
#[derive(Debug, Clone)]
pub struct Report {
    pub title: String,
    pub status: Status,
    pub message: String,
}

/// One notice for the whole run.
pub fn chain_summary(name: &str, reports: &[Report]) -> String {
    let done = reports.iter().filter(|r| r.status == Status::Done).count();
    let head = if done == 0 {
        format!("Chain \"{name}\" ran and changed nothing.")
    } else {
        format!(
            "Chain \"{name}\" ran {done} of {} steps. One undo takes it all back.",
            reports.len()
        )
    };
    let mut parts = vec![head];
    for r in reports {
        parts.push(if r.status == Status::Skipped {
            format!("{}: skipped ({})", r.title, r.message)
        } else {
            format!("{}: {}", r.title, r.message)
        });
    }
    parts.join(" ")
}
