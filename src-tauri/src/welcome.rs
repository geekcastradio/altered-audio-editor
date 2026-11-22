// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! What the welcome shows at start-up. The Rust copy of `welcomePlan` in
//! `src/app/welcome.ts`; both are tested against
//! `shared-fixtures/app-rules.json`. The page text stays in TypeScript.

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanKind {
    Install,
    Upgrade,
}

/// What to show, from the settings as they were saved (`None` or JSON null
/// when nothing was saved yet) and each page's revision. Returns the kind and
/// the indices of the pages to show, or None to show nothing. Settings from
/// before the welcome existed count as having seen revision 1.
pub fn welcome_plan(saved: Option<&Value>, revs: &[f64]) -> Option<(PlanKind, Vec<usize>)> {
    let all = || (0..revs.len()).collect::<Vec<_>>();
    let saved = match saved {
        None | Some(Value::Null) => return Some((PlanKind::Install, all())),
        Some(v) => v,
    };
    if !(saved.is_object() || saved.is_array()) {
        return None;
    }
    let seen = match saved.get("welcomeRev").and_then(Value::as_f64) {
        Some(r) if r.is_finite() => 0.0_f64.max(r.floor()),
        _ if saved.get("welcomeSeen") == Some(&Value::Bool(false)) => 0.0,
        _ => 1.0,
    };
    if seen == 0.0 {
        return Some((PlanKind::Install, all()));
    }
    let news: Vec<usize> = revs
        .iter()
        .enumerate()
        .filter(|(_, r)| **r > seen)
        .map(|(i, _)| i)
        .collect();
    (!news.is_empty()).then_some((PlanKind::Upgrade, news))
}
