// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! List selection, folder navigation and solo rules in Rust: click and arrow
//! selection in lists, the parent of a folder path, which tracks are silent
//! right now, and the solo edit. Mirrors `files/selection.ts`,
//! `drive-manager/paths.ts` (`parentFolder`), `mixer/solo.ts` and
//! `properties/remix.ts`, tested against `shared-fixtures/panel-nav.json`.

use crate::js_compat::js_to_fixed;
use crate::music_fit::FIT_XFADE;
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// Standard Windows list selection: click, Ctrl+click, Shift+click. Returns
/// the selected ids and the anchor.
pub fn click_selection(
    current: &[String],
    anchor: Option<&str>,
    clicked: &str,
    visible: &[String],
    ctrl: bool,
    shift: bool,
) -> (Vec<String>, String) {
    if shift {
        if let Some(anchor) = anchor.filter(|a| visible.iter().any(|v| v == a)) {
            let a = visible.iter().position(|v| v == anchor).unwrap_or(0);
            // indexOf gives -1 for an id that is not listed, as JavaScript does.
            let b = visible
                .iter()
                .position(|v| v == clicked)
                .map_or(-1, |i| i as i64);
            let a = a as i64;
            let (lo, hi) = (a.min(b), a.max(b));
            let range: Vec<String> = slice(visible, lo, hi + 1);
            let mut base: Vec<String> = if ctrl {
                current
                    .iter()
                    .filter(|id| !range.contains(id))
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };
            base.extend(range);
            return (base, anchor.to_string());
        }
    }
    if ctrl {
        let mut next: Vec<String> = Vec::new();
        for id in current {
            if !next.contains(id) {
                next.push(id.clone());
            }
        }
        if let Some(i) = next.iter().position(|x| x == clicked) {
            next.remove(i);
        } else {
            next.push(clicked.to_string());
        }
        return (next, clicked.to_string());
    }
    (vec![clicked.to_string()], clicked.to_string())
}

/// `Array.prototype.slice` with a start that may be -1 (counted from the end).
fn slice(v: &[String], from: i64, to: i64) -> Vec<String> {
    let n = v.len() as i64;
    let fix = |i: i64| if i < 0 { (n + i).max(0) } else { i.min(n) };
    let (a, b) = (fix(from), fix(to));
    if a >= b {
        Vec::new()
    } else {
        v[a as usize..b as usize].to_vec()
    }
}

/// Arrow-key movement. The id to select, or None to stay put.
pub fn step_selection(visible: &[String], from: Option<&str>, delta: i64) -> Option<String> {
    if visible.is_empty() {
        return None;
    }
    match from.and_then(|f| visible.iter().position(|v| v == f)) {
        None => Some(if delta > 0 {
            visible[0].clone()
        } else {
            visible[visible.len() - 1].clone()
        }),
        Some(i) => {
            let at = (i as i64 + delta).clamp(0, visible.len() as i64 - 1);
            Some(visible[at as usize].clone())
        }
    }
}

fn is_drive(s: &str) -> bool {
    let c: Vec<char> = s.chars().collect();
    c.len() >= 2 && c[0].is_ascii_alphabetic() && c[1] == ':'
}

/// The folder above `path`, or None at the top. Handles Windows
/// (`C:\Shows\Raw`) and POSIX (`/home/host/Raw`) paths.
pub fn parent_folder(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    let n = path.chars().count();
    let rest: String = path.chars().skip(2).collect();
    if is_drive(path) && (n == 2 || (n == 3 && (rest == "\\" || rest == "/"))) || path == "/" {
        return None;
    }
    let sep = if path.contains('\\') { '\\' } else { '/' };
    let trimmed = path.strip_suffix(sep).unwrap_or(path);
    let i = trimmed.rfind(sep)?;
    let up = &trimmed[..i];
    if up.chars().count() == 2 && is_drive(up) {
        return Some(format!("{up}\\"));
    }
    Some(if up.is_empty() {
        "/".to_string()
    } else {
        up.to_string()
    })
}

fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k)
        .and_then(Value::as_array)
        .map_or(&[], |a| a.as_slice())
}

fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn on(v: &Value, k: &str) -> bool {
    match v.get(k) {
        None | Some(Value::Null) | Some(Value::Bool(false)) => false,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        _ => true,
    }
}

/// Track header and mixer changes as one op. With exclusive solo on, soloing a
/// track unsolos the other tracks and buses in the same undoable step.
pub fn track_op(doc: Option<&Value>, id: &str, changes: &Value, exclusive: bool) -> Value {
    let op = json!({ "op": "set_track", "id": id, "changes": changes });
    let Some(doc) = doc else { return op };
    if !exclusive || changes.get("solo") != Some(&json!(true)) {
        return op;
    }
    let others: Vec<&Value> = arr(doc, "tracks")
        .iter()
        .filter(|t| text(t, "id") != id && on(t, "solo"))
        .collect();
    let buses: Vec<&Value> = arr(doc, "buses").iter().filter(|b| on(b, "solo")).collect();
    if others.is_empty() && buses.is_empty() {
        return op;
    }
    let mut ops = vec![op];
    ops.extend(
        others
            .iter()
            .map(|t| json!({ "op": "set_track", "id": t["id"], "changes": { "solo": false } })),
    );
    ops.extend(
        buses
            .iter()
            .map(|b| json!({ "op": "set_bus", "id": b["id"], "changes": { "solo": false } })),
    );
    json!({ "op": "batch", "ops": ops })
}

fn feeds(v: &Value) -> Vec<String> {
    let mut next = Vec::new();
    if v.get("output").and_then(|o| o.get("type")) == Some(&json!("bus")) {
        next.push(
            v["output"]
                .get("bus_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        );
    }
    next.extend(
        arr(v, "sends")
            .iter()
            .map(|x| text(x, "bus_id").to_string()),
    );
    next
}

fn reaches(buses: &[Value], bus_id: &str, seen: &mut BTreeSet<String>) -> bool {
    if !seen.insert(bus_id.to_string()) {
        return false;
    }
    let Some(b) = buses.iter().rev().find(|b| text(b, "id") == bus_id) else {
        return false;
    };
    if on(b, "solo") {
        return true;
    }
    feeds(b).iter().any(|id| reaches(buses, id, seen))
}

/// The tracks not heard right now: muted ones, and while anything is soloed,
/// every track that is not soloed and does not feed a soloed bus.
pub fn silent_tracks(doc: Option<&Value>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let Some(doc) = doc else { return out };
    let buses = arr(doc, "buses");
    let any_solo =
        arr(doc, "tracks").iter().any(|t| on(t, "solo")) || buses.iter().any(|b| on(b, "solo"));
    for t in arr(doc, "tracks") {
        if on(t, "muted") {
            out.insert(text(t, "id").to_string());
            continue;
        }
        if !any_solo || on(t, "solo") {
            continue;
        }
        if !feeds(t)
            .iter()
            .any(|id| reaches(buses, id, &mut BTreeSet::new()))
        {
            out.insert(text(t, "id").to_string());
        }
    }
    out
}

/// Seconds the plan plays once applied: `segments` as (start, end) pairs.
pub fn plan_duration(segments: &[(f64, f64)], stretch: f64) -> f64 {
    if segments.is_empty() {
        return 0.0;
    }
    let body: f64 = segments.iter().map(|(s, e)| (e - s).max(0.0)).sum();
    ((body - (segments.len() as f64 - 1.0) * FIT_XFADE) * stretch).max(0.0)
}

/// "42.500" under a minute, "1:05.250" from a minute up.
pub fn format_seconds(s: f64) -> String {
    let safe = if s.is_finite() && s > 0.0 { s } else { 0.0 };
    let m = (safe / 60.0).floor();
    let rest = js_to_fixed(safe - m * 60.0, 3);
    let rest = format!("{rest:0>6}");
    if m > 0.0 {
        format!("{m}:{rest}")
    } else {
        js_to_fixed(safe, 3)
    }
}
