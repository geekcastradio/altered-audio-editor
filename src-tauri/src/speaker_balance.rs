// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Speaker Balance arithmetic: bring every measured voice track to the
//! quietest one. The Rust copy of `balance` and `describeBalance` in
//! `src/app/speaker-balance.ts`; both are tested against
//! `shared-fixtures/speaker-balance.json`. Which tracks count as voices, and
//! the clip gain changes, stay in TypeScript with the session.

use crate::js_compat::{js_round, js_to_fixed1};
use crate::loudness_match::{MatchItem, MatchRow};

/// Differences smaller than this are left alone, in LU.
pub const BALANCE_TOLERANCE: f64 = 0.5;

/// What a loudness report gives the balance: integrated loudness and true peak.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BalanceReading {
    pub integrated: f64,
    pub true_peak: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Balance {
    pub rows: Vec<MatchRow>,
    /// Index into `rows` of the quietest measurable track, which stays where it is.
    pub reference: Option<usize>,
}

fn round1(x: f64) -> f64 {
    js_round(x * 10.0) / 10.0
}

/// Rows that bring each measured track to the quietest one. A missing report
/// leaves its track unmeasured.
pub fn balance(items: &[MatchItem], reports: &[Option<BalanceReading>]) -> Balance {
    let mut rows: Vec<MatchRow> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let r = reports.get(i).copied().flatten();
            let measured = r.map_or(f64::NEG_INFINITY, |r| r.integrated);
            let ok = r.is_some() && measured.is_finite() && measured > -70.0;
            MatchRow {
                item: item.clone(),
                measured,
                true_peak: r.map_or(f64::NEG_INFINITY, |r| r.true_peak),
                change: 0.0,
                peak_after: r.map_or(0.0, |r| r.true_peak),
                measurable: ok,
                peak_warning: false,
            }
        })
        .collect();
    let measured: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].measurable).collect();
    if measured.len() < 2 {
        return Balance {
            reference: measured.first().copied(),
            rows,
        };
    }
    let mut reference = measured[0];
    for &i in &measured[1..] {
        if rows[i].measured < rows[reference].measured {
            reference = i;
        }
    }
    let floor = rows[reference].measured;
    for &i in &measured {
        let diff = floor - rows[i].measured;
        rows[i].change = if diff.abs() < BALANCE_TOLERANCE {
            0.0
        } else {
            round1(diff)
        };
        rows[i].peak_after = round1(rows[i].true_peak + rows[i].change);
    }
    Balance {
        rows,
        reference: Some(reference),
    }
}

/// One line for the notice.
pub fn describe_balance(b: &Balance) -> String {
    let Some(reference) = b.reference else {
        return "No voice tracks with audio to balance.".to_string();
    };
    let measurable = b.rows.iter().filter(|r| r.measurable).count();
    if measurable < 2 {
        return "Speaker Balance needs at least two voice tracks with audio.".to_string();
    }
    let moved: Vec<&MatchRow> = b
        .rows
        .iter()
        .filter(|r| r.measurable && r.change != 0.0)
        .collect();
    if moved.is_empty() {
        return format!("The voices already sit within {BALANCE_TOLERANCE} LU of each other.");
    }
    let list = moved
        .iter()
        .map(|r| {
            format!(
                "{} {}{} dB",
                r.item.label,
                if r.change > 0.0 { "+" } else { "" },
                js_to_fixed1(r.change)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let r = &b.rows[reference];
    format!(
        "Balanced to {} ({} LUFS): {}.",
        r.item.label,
        js_to_fixed1(r.measured),
        list
    )
}
