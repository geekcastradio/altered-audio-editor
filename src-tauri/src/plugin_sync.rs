// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Plug-in slot bookkeeping and chain drag edits in Rust. Mirrors the pure
//! parts of `src/app/plugin-sync.ts` (which slots exist, and the edits that
//! bring plug-in events and fetched settings into the session) and
//! `src/panels/voice-chain/slot-drag.ts` (owner keys, the chain a target
//! names, the edit a drop makes). Tested against
//! `shared-fixtures/plugin-sync.json`.

use crate::voice_chain::{add_slot_at, chain_of, move_slot_to_index};
use serde_json::{json, Map, Value};

/// One slot with whose chain it sits in.
#[derive(Debug, Clone, PartialEq)]
pub struct ChainSlot {
    pub target: Value,
    pub chain: Value,
    pub slot: Value,
}

/// A plug-in's events: `edits` are `(key, value)` pairs.
#[derive(Debug, Clone)]
pub struct PluginEvent {
    pub slot_id: String,
    pub edits: Vec<(String, Value)>,
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map_or(&[], |a| a.as_slice())
}

fn sid(slot: &Value) -> &str {
    slot.get("id").and_then(Value::as_str).unwrap_or("")
}

/// Every MIDI track's instrument as `(track id, slot)`.
pub fn instrument_slots(doc: &Value) -> Vec<(String, Value)> {
    arr(doc, "tracks")
        .iter()
        .filter_map(|t| {
            let i = t
                .get("instrument")
                .filter(|i| !i.is_null() && i.as_bool() != Some(false))?;
            Some((sid(t).to_string(), i.clone()))
        })
        .collect()
}

/// Whether a slot hosts a VST3 plug-in.
pub fn is_plugin_slot(s: &Value) -> bool {
    s.get("effect")
        .and_then(Value::as_str)
        .is_some_and(|e| e.starts_with("vst3:"))
}

/// Every chain slot in the session, with whose chain it is.
pub fn all_slots(doc: &Value) -> Vec<ChainSlot> {
    let mut out = Vec::new();
    let mut add = |target: Value, chain: Option<&Value>| {
        if let Some(chain) = chain.filter(|c| c.is_object()) {
            for slot in arr(chain, "slots") {
                out.push(ChainSlot {
                    target: target.clone(),
                    chain: chain.clone(),
                    slot: slot.clone(),
                });
            }
        }
    };
    for t in arr(doc, "tracks") {
        add(json!({ "kind": "track", "id": t["id"] }), t.get("effects"));
        for c in arr(t, "clips") {
            add(json!({ "kind": "clip", "id": c["id"] }), c.get("effects"));
        }
    }
    for b in arr(doc, "buses") {
        add(json!({ "kind": "bus", "id": b["id"] }), b.get("effects"));
    }
    add(
        json!({ "kind": "master" }),
        doc.get("master").and_then(|m| m.get("effects")),
    );
    out
}

/// Every plug-in slot: chain slots and instruments.
pub fn plugin_slots(doc: &Value) -> Vec<Value> {
    all_slots(doc)
        .into_iter()
        .map(|x| x.slot)
        .chain(instrument_slots(doc).into_iter().map(|x| x.1))
        .filter(is_plugin_slot)
        .collect()
}

fn key_of(t: &Value) -> String {
    match t.get("id").and_then(Value::as_str) {
        Some(id) => format!("{}:{id}", t["kind"].as_str().unwrap_or("")),
        None => t["kind"].as_str().unwrap_or("").to_string(),
    }
}

fn with_slots(chain: &Value, f: impl Fn(&Value) -> Value) -> Value {
    let mut c = chain.clone();
    if let Some(slots) = c.get_mut("slots").and_then(Value::as_array_mut) {
        for s in slots.iter_mut() {
            *s = f(s);
        }
    }
    c
}

/// The edits that bring plug-in events and fetched settings into the
/// session: one `set_effects` per chain touched, one `set_track` per MIDI
/// track whose instrument changed. `states` is `(slot id, settings)`.
pub fn sync_ops(doc: &Value, events: &[PluginEvent], states: &[(String, String)]) -> Vec<Value> {
    let slots = all_slots(doc);
    let instruments = instrument_slots(doc);
    // Insertion order matters: a JS Map iterates in the order keys were first set.
    let mut by_target: Vec<(String, Value, Value)> = Vec::new();
    let mut changed_instruments: Vec<(String, Value)> = Vec::new();
    let mut touch = |slot_id: &str, change: &dyn Fn(&Value) -> Value| {
        if let Some((track, slot)) = instruments.iter().find(|x| sid(&x.1) == slot_id) {
            match changed_instruments.iter_mut().find(|x| &x.0 == track) {
                Some(cur) => cur.1 = change(&cur.1),
                None => changed_instruments.push((track.clone(), change(slot))),
            }
            return;
        }
        let Some(found) = slots.iter().find(|x| sid(&x.slot) == slot_id) else {
            return;
        };
        let k = key_of(&found.target);
        let pos = by_target.iter().position(|x| x.0 == k);
        let (target, chain) = match pos {
            Some(i) => (by_target[i].1.clone(), by_target[i].2.clone()),
            None => (found.target.clone(), found.chain.clone()),
        };
        let next = with_slots(&chain, |s| {
            if sid(s) == slot_id {
                change(s)
            } else {
                s.clone()
            }
        });
        match pos {
            Some(i) => by_target[i].2 = next,
            None => by_target.push((k, target, next)),
        }
    };
    for e in events {
        if !e.edits.is_empty() {
            touch(&e.slot_id, &|s| {
                let mut s = s.clone();
                let mut params = s
                    .get("params")
                    .and_then(Value::as_object)
                    .cloned()
                    .unwrap_or_default();
                for (key, v) in &e.edits {
                    params.insert(key.clone(), v.clone());
                }
                s["params"] = Value::Object(params);
                s["preset"] = Value::Null;
                s
            });
        }
    }
    for (slot_id, data) in states {
        touch(slot_id, &|s| {
            if s.get("plugin_state").and_then(Value::as_str) == Some(data.as_str()) {
                s.clone()
            } else {
                let mut s = s.clone();
                s["plugin_state"] = json!(data);
                s
            }
        });
    }
    let mut ops = Vec::new();
    for (k, target, chain) in &by_target {
        if let Some(before) = slots
            .iter()
            .find(|x| &key_of(&x.target) == k)
            .map(|x| &x.chain)
        {
            if before == chain {
                continue;
            }
        }
        ops.push(json!({ "op": "set_effects", "target": target, "chain": chain }));
    }
    for (track, slot) in &changed_instruments {
        if let Some(before) = instruments.iter().find(|x| &x.0 == track).map(|x| &x.1) {
            if before == slot {
                continue;
            }
        }
        ops.push(json!({ "op": "set_track", "id": track, "changes": { "instrument": slot } }));
    }
    ops
}

/// One string per chain, for the `data-slot-owner` attribute.
pub fn owner_key(t: &Value) -> String {
    if t["kind"] == "master" {
        "master".to_string()
    } else {
        format!(
            "{}:{}",
            t["kind"].as_str().unwrap_or(""),
            t["id"].as_str().unwrap_or("")
        )
    }
}

/// The target an owner key names, if it is one.
pub fn parse_owner(key: &str) -> Option<Value> {
    if key == "master" {
        return Some(json!({ "kind": "master" }));
    }
    let mut it = key.splitn(2, ':');
    let kind = it.next()?;
    let id = it
        .next()
        .and_then(|r| r.split(':').next())
        .filter(|i| !i.is_empty())?;
    matches!(kind, "track" | "bus" | "clip").then(|| json!({ "kind": kind, "id": id }))
}

/// The chain a target names in `doc`.
pub fn chain_at(doc: &Value, t: &Value) -> Option<Value> {
    let by_id = |list: &str| arr(doc, list).iter().find(|x| x["id"] == t["id"]).cloned();
    match t["kind"].as_str()? {
        "master" => Some(chain_of(doc.get("master").and_then(|m| m.get("effects")))),
        "track" => by_id("tracks").map(|x| chain_of(x.get("effects"))),
        "bus" => by_id("buses").map(|x| chain_of(x.get("effects"))),
        _ => arr(doc, "tracks")
            .iter()
            .flat_map(|tr| arr(tr, "clips"))
            .find(|c| c["id"] == t["id"])
            .map(|c| chain_of(c.get("effects"))),
    }
}

/// The edit a drop makes: within the same chain the effect moves; onto
/// another chain a copy (a new id, the same settings) goes into that slot.
/// None when nothing changes or the target chain is full.
pub fn drop_edit(
    doc: &Value,
    from: &Value,
    slot: &Value,
    owner: &str,
    index: f64,
    new_id: &str,
) -> Option<(String, Value)> {
    let target = parse_owner(owner)?;
    let chain = chain_at(doc, &target)?;
    if owner_key(from) == owner {
        let next = move_slot_to_index(&chain, sid(slot), index);
        if next["slots"] == chain["slots"] {
            return None;
        }
        return Some((
            "Move Effect".into(),
            json!({ "op": "set_effects", "target": target, "chain": next }),
        ));
    }
    let mut copy = slot.clone();
    copy["id"] = json!(new_id);
    let own_sidechain = target["kind"] == "track" && slot.get("sidechain") == target.get("id");
    copy["sidechain"] = if own_sidechain {
        Value::Null
    } else {
        slot.get("sidechain")
            .cloned()
            .filter(|s| !s.is_null())
            .unwrap_or(Value::Null)
    };
    let next = add_slot_at(&chain, &copy, index)?;
    Some((
        "Copy Effect".into(),
        json!({ "op": "set_effects", "target": target, "chain": next }),
    ))
}

/// The effect target of a header row: a track or bus by id, or the master.
pub fn row_target(id: &str, bus: bool) -> Value {
    if id == "master" {
        json!({ "kind": "master" })
    } else {
        let mut m = Map::new();
        m.insert("kind".into(), json!(if bus { "bus" } else { "track" }));
        m.insert("id".into(), json!(id));
        Value::Object(m)
    }
}
