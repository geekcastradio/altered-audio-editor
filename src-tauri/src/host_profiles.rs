// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Host profiles in Rust: the chain a profile puts on a voice track and the
//! edit that applies it. Mirrors `src/app/host-profiles.ts`; tested against
//! `shared-fixtures/sound-host.json`.

use crate::voice_chain::{builtin_presets, preset_chain};
use serde_json::{json, Value};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
pub struct HostProfile {
    pub id: &'static str,
    pub name: &'static str,
    pub note: &'static str,
    pub gate_db: f64,
    pub guest: bool,
}

pub const HOST_PROFILES: [HostProfile; 3] = [
    HostProfile {
        id: "close-mic",
        name: "Close Mic",
        note: "A strong voice right on the mic.",
        gate_db: -14.0,
        guest: false,
    },
    HostProfile {
        id: "standard-mic",
        name: "Standard Mic",
        note: "A typical speaker at normal mic distance.",
        gate_db: -20.0,
        guest: false,
    },
    HostProfile {
        id: "remote-guest",
        name: "Remote Guest",
        note: "Someone calling in over the internet or phone.",
        gate_db: -30.0,
        guest: true,
    },
];

impl HostProfile {
    pub fn role(&self) -> &'static str {
        if self.guest {
            "guest"
        } else {
            "standard"
        }
    }
}

pub fn profile_by_id(id: Option<&str>) -> Option<HostProfile> {
    let id = id?;
    HOST_PROFILES.iter().find(|p| p.id == id).copied()
}

/// The chain a profile puts on the track.
pub fn profile_chain(p: &HostProfile, new_id: &mut dyn FnMut() -> String) -> Value {
    let name = if p.guest {
        "Remote Guest Cleanup"
    } else {
        "Podcast Voice"
    };
    let presets = builtin_presets();
    let preset = presets
        .iter()
        .find(|x| x["name"] == name)
        .unwrap_or_else(|| panic!("Missing preset {name}"));
    // Profiles only use built-in effects, so every effect counts as known.
    let known: HashSet<String> = preset["chain"]["slots"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s["effect"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let mut base = preset_chain(preset, &known, new_id);
    let tag = format!("{} profile", p.name);
    let mut slots: Vec<Value> = base["slots"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|s| s["effect"] != "altaudio.breath_gate")
        .collect();
    if p.guest {
        // Phone band: the low-pass keeps call-codec fizz out above 7 kHz.
        if let Some(eq) = slots
            .iter_mut()
            .find(|s| s["effect"] == "altaudio.parametric_eq")
        {
            if let Some(params) = eq["params"].as_object_mut() {
                params.insert("lp_on".into(), json!(1));
                params.insert("lp_freq".into(), json!(7000));
            }
        }
    }
    slots.push(json!({
        "id": new_id(),
        "effect": "altaudio.breath_gate",
        "enabled": true,
        "params": { "threshold": p.gate_db, "attack": 2, "hold": 50, "release": 200, "range": -30 },
        "preset": tag,
    }));
    for s in &mut slots {
        s["preset"] = json!(tag);
    }
    base["slots"] = Value::Array(slots);
    base
}

/// The edit that applies `p` to a track, or clears the profile when `p` is None.
pub fn profile_op(
    track_id: &str,
    p: Option<&HostProfile>,
    new_id: &mut dyn FnMut() -> String,
) -> Value {
    let Some(p) = p else {
        return json!({ "op": "set_track", "id": track_id, "changes": { "host_profile_id": "", "role": "standard" } });
    };
    json!({
        "op": "batch",
        "ops": [
            { "op": "set_track", "id": track_id, "changes": { "host_profile_id": p.id, "role": p.role() } },
            { "op": "set_effects", "target": { "kind": "track", "id": track_id }, "chain": profile_chain(p, new_id) },
        ],
    })
}
