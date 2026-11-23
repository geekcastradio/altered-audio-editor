// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Automation lane choices and edits in Rust: which settings of a track or bus
//! a lane can move, a lane's name, range and resting value, and the edits to an
//! owner's lanes (add, replace, drop, automation mode). Mirrors the owner and
//! catalog parts of `src/panels/editor/automation.ts`, tested against
//! `shared-fixtures/auto-lanes.json`. Owners, lanes and the effect catalog are
//! JSON, as the session holds them.

use crate::automation::{pan_range, space_range, volume_range, AutoRange};
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// The spot settings a lane can move, in the order the picker lists them:
/// (param, label, only for stereo).
const SPACE_LANES: [(&str, &str, bool); 9] = [
    ("x", "3D: Left / right", false),
    ("y", "3D: Back / front", false),
    ("z", "3D: Height", false),
    ("size", "3D: Size", false),
    ("spread", "3D: Spread", false),
    ("width", "3D: Width", true),
    ("rotation", "3D: Rotation", true),
    ("spatialize", "3D: Spatialize", true),
    ("lfe", "3D: LFE send", false),
];

/// The default spot of a source in a 3D Space session.
const DEFAULT_SPACE_POS: [(&str, f64); 9] = [
    ("x", 0.0),
    ("y", 1.0),
    ("z", 0.0),
    ("size", 0.0),
    ("width", 1.0),
    ("rotation", 0.0),
    ("lfe", 0.0),
    ("spread", 0.0),
    ("spatialize", 0.0),
];

fn default_space(p: &str) -> Option<f64> {
    DEFAULT_SPACE_POS
        .iter()
        .find(|(k, _)| *k == p)
        .map(|(_, v)| *v)
}

fn arr<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v.get(k)
        .and_then(Value::as_array)
        .map_or(&[], |a| a.as_slice())
}

fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn num(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(0.0)
}

/// JavaScript truthiness of a field.
fn on(v: &Value, k: &str) -> bool {
    match v.get(k) {
        None | Some(Value::Null) | Some(Value::Bool(false)) => false,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        _ => true,
    }
}

/// A stable key for a target.
pub fn target_key(t: &Value) -> String {
    match text(t, "type") {
        "effect_param" => format!("fx:{}:{}", text(t, "slot_id"), text(t, "param")),
        "space" => format!("space:{}", text(t, "param")),
        other => other.to_string(),
    }
}

struct Found<'a> {
    info: &'a Value,
    param: &'a Value,
    value: f64,
}

fn param_of<'a>(owner: &'a Value, catalog: &'a [Value], t: &Value) -> Option<Found<'a>> {
    if text(t, "type") != "effect_param" {
        return None;
    }
    let slots = owner.get("effects").map_or(&[][..], |e| arr(e, "slots"));
    let slot = slots.iter().find(|s| text(s, "id") == text(t, "slot_id"))?;
    let info = catalog
        .iter()
        .find(|e| text(e, "id") == text(slot, "effect"))?;
    let param = arr(info, "params")
        .iter()
        .find(|p| text(p, "key") == text(t, "param") && !on(p, "hidden"))?;
    let key = text(param, "key");
    let value = slot
        .get("params")
        .and_then(|p| p.get(key))
        .and_then(Value::as_f64)
        .unwrap_or_else(|| num(param, "default"));
    Some(Found { info, param, value })
}

/// Every setting of an owner that a lane can move: volume, pan (or in a 3D
/// Space session the spot's settings), then each effect control in chain order.
pub fn auto_choices(owner: &Value, catalog: &[Value], in_3d: bool) -> Vec<Value> {
    let mut out = vec![json!({ "target": { "type": "volume" }, "label": "Volume" })];
    if in_3d {
        let stereo = text(owner, "channels") != "mono" && !on(owner, "mono");
        for (param, label, only_stereo) in SPACE_LANES {
            if stereo || !only_stereo {
                out.push(json!({ "target": { "type": "space", "param": param }, "label": label }));
            }
        }
    } else {
        out.push(json!({ "target": { "type": "pan" }, "label": "Pan" }));
    }
    let slots = owner.get("effects").map_or(&[][..], |e| arr(e, "slots"));
    for slot in slots {
        let info = catalog
            .iter()
            .find(|e| text(e, "id") == text(slot, "effect"));
        let Some(info) = info else { continue };
        for p in arr(info, "params") {
            if on(p, "hidden") || on(p, "retimes") || on(p, "toggle") || on(p, "choices") {
                continue;
            }
            out.push(json!({
                "target": { "type": "effect_param", "slot_id": slot["id"], "param": p["key"] },
                "label": format!("{}: {}", text(info, "name"), text(p, "name")),
                "group": format!("{}#{}", text(info, "name"), text(slot, "id")),
                "name": p["name"],
            }));
        }
    }
    out
}

/// The lane's name, for its header and undo.
pub fn auto_name(owner: &Value, catalog: &[Value], t: &Value) -> String {
    match text(t, "type") {
        "volume" => "Volume".into(),
        "pan" => "Pan".into(),
        "space" => SPACE_LANES
            .iter()
            .find(|l| l.0 == text(t, "param"))
            .map_or("3D", |l| l.1)
            .to_string(),
        _ => match param_of(owner, catalog, t) {
            Some(p) => format!("{}: {}", text(p.info, "name"), text(p.param, "name")),
            None => "Removed effect".into(),
        },
    }
}

/// True when the lane's effect is no longer in the chain.
pub fn is_orphan(owner: &Value, catalog: &[Value], t: &Value) -> bool {
    text(t, "type") == "effect_param" && param_of(owner, catalog, t).is_none()
}

pub fn auto_range(owner: &Value, catalog: &[Value], t: &Value) -> AutoRange {
    match text(t, "type") {
        "volume" => volume_range(),
        "pan" => pan_range(),
        "space" => space_range(text(t, "param")),
        _ => match param_of(owner, catalog, t) {
            None => AutoRange {
                top: 1.0,
                bottom: 0.0,
                step: 0.01,
                log: false,
                unit: String::new(),
            },
            Some(p) => {
                let (min, max, step) = (
                    num(p.param, "min"),
                    num(p.param, "max"),
                    num(p.param, "step"),
                );
                AutoRange {
                    top: max,
                    bottom: min,
                    step: if step > 0.0 { step } else { 0.01 },
                    log: on(p.param, "log") && min > 0.0,
                    unit: text(p.param, "unit").to_string(),
                }
            }
        },
    }
}

/// Where a lane with no points sits: the fader, the pan, or the effect's own setting.
pub fn auto_rest(owner: &Value, catalog: &[Value], t: &Value) -> f64 {
    match text(t, "type") {
        "volume" => num(owner, "volume_db"),
        "pan" => num(owner, "pan"),
        "space" => {
            let p = text(t, "param");
            owner
                .get("space")
                .filter(|s| !s.is_null())
                .and_then(|s| s.get(p))
                .and_then(Value::as_f64)
                .or_else(|| default_space(p))
                .unwrap_or(0.0)
        }
        _ => param_of(owner, catalog, t).map_or(0.0, |p| p.value),
    }
}

/// A new, empty lane for `target`.
pub fn new_lane(target: &Value, id: &str) -> Value {
    json!({ "id": id, "target": target, "enabled": true, "points": [] })
}

/// The lanes an owner shows: its envelopes in order, or a Volume lane to start
/// from when it has none.
pub fn shown_lanes(owner: &Value) -> Vec<Value> {
    let envs = arr(owner, "envelopes");
    if envs.is_empty() {
        vec![json!({ "id": "", "target": { "type": "volume" }, "enabled": true, "points": [] })]
    } else {
        envs.to_vec()
    }
}

/// The owner's envelopes with `lane` put in (replacing the one with its id, or
/// added at the end). `new_id` supplies an id for a lane that has none.
pub fn put_lane(owner: &Value, lane: &Value, new_id: &mut dyn FnMut() -> String) -> Vec<Value> {
    let envs = arr(owner, "envelopes");
    let mut l = lane.clone();
    if text(lane, "id").is_empty() {
        l["id"] = json!(new_id());
    }
    match envs.iter().position(|e| text(e, "id") == text(&l, "id")) {
        None => envs.iter().cloned().chain(std::iter::once(l)).collect(),
        Some(i) => envs
            .iter()
            .enumerate()
            .map(|(k, e)| if k == i { l.clone() } else { e.clone() })
            .collect(),
    }
}

pub fn drop_lane(owner: &Value, id: &str) -> Vec<Value> {
    arr(owner, "envelopes")
        .iter()
        .filter(|e| text(e, "id") != id)
        .cloned()
        .collect()
}

/// The setting a new lane takes: the first one that has no lane yet.
pub fn next_target(owner: &Value, catalog: &[Value], in_3d: bool) -> Option<Value> {
    let used: BTreeSet<String> = shown_lanes(owner)
        .iter()
        .map(|e| target_key(&e["target"]))
        .collect();
    auto_choices(owner, catalog, in_3d)
        .into_iter()
        .find(|c| !used.contains(&target_key(&c["target"])))
        .map(|c| c["target"].clone())
}

/// The track's automation mode as its header shows it: "read" while any lane
/// plays, "off" when every lane is off, or its record mode.
pub fn auto_mode(owner: &Value) -> String {
    if on(owner, "record_mode") {
        return text(owner, "record_mode").to_string();
    }
    let envs = arr(owner, "envelopes");
    if envs.is_empty() || envs.iter().any(|e| on(e, "enabled")) {
        "read"
    } else {
        "off"
    }
    .to_string()
}

fn with_enabled(owner: &Value, enabled: bool) -> Vec<Value> {
    arr(owner, "envelopes")
        .iter()
        .map(|e| {
            let mut e = e.clone();
            e["enabled"] = json!(enabled);
            e
        })
        .collect()
}

/// The lanes and record setting a mode choice leaves: every mode but "off"
/// plays the lanes; only write, latch and touch record.
pub fn mode_change(owner: &Value, mode: &str) -> (Vec<Value>, String) {
    let record = if matches!(mode, "write" | "latch" | "touch") {
        mode
    } else {
        "off"
    };
    (with_enabled(owner, mode != "off"), record.to_string())
}

/// Every lane on ("read") or off.
pub fn set_mode(owner: &Value, mode: &str) -> Vec<Value> {
    with_enabled(owner, mode == "read")
}
