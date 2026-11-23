// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Effect envelopes on a clip: which effect settings can move over time, the lane shown,
//! its range, points, resting value and name. The Rust copy of the effect parts of
//! `src/panels/editor/envelope.ts`, which read the effect catalog; here the catalog is
//! passed in. Clips are plain JSON. Tested against `shared-fixtures/envelope-fx.json`.

use crate::envelope::{range_of, EnvKind, EnvRange};
use crate::mix_routing::truthy;
use serde_json::{json, Value};
use std::collections::HashMap;

/// One effect setting on a clip that can have an envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct FxLane {
    pub slot: String,
    pub param: String,
}

/// The effect catalog by id. A later entry with the same id replaces an earlier one.
pub struct Catalog(HashMap<String, Value>);

impl Catalog {
    pub fn new(list: &[Value]) -> Self {
        let mut m = HashMap::new();
        for e in list {
            if let Some(id) = e["id"].as_str() {
                m.insert(id.to_string(), e.clone());
            }
        }
        Catalog(m)
    }
}

/// The control behind a lane.
#[derive(Debug, Clone, PartialEq)]
pub struct FxParam {
    pub effect: String,
    pub param: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub value: f64,
}

fn slots(clip: &Value) -> &[Value] {
    clip["effects"]["slots"]
        .as_array()
        .map_or(&[], Vec::as_slice)
}

fn envelopes(clip: &Value) -> &[Value] {
    clip["envelopes"].as_array().map_or(&[], Vec::as_slice)
}

fn params_of(info: &Value) -> &[Value] {
    info["params"].as_array().map_or(&[], Vec::as_slice)
}

/// The control behind `lane` on `clip`, if that effect is still in its chain.
pub fn fx_param(cat: &Catalog, clip: &Value, lane: Option<&FxLane>) -> Option<FxParam> {
    let lane = lane?;
    let slot = slots(clip)
        .iter()
        .find(|s| s["id"].as_str() == Some(lane.slot.as_str()))?;
    let info = cat.0.get(slot["effect"].as_str()?)?;
    let param = params_of(info)
        .iter()
        .find(|p| p["key"].as_str() == Some(lane.param.as_str()) && !truthy(&p["hidden"]))?;
    let set = &slot["params"][lane.param.as_str()];
    let value = if set.is_null() {
        param["default"].as_f64().unwrap_or(f64::NAN)
    } else {
        set.as_f64().unwrap_or(f64::NAN)
    };
    Some(FxParam {
        effect: info["name"].as_str().unwrap_or("").to_string(),
        param: param["name"].as_str().unwrap_or("").to_string(),
        min: param["min"].as_f64().unwrap_or(f64::NAN),
        max: param["max"].as_f64().unwrap_or(f64::NAN),
        step: param["step"].as_f64().unwrap_or(f64::NAN),
        value,
    })
}

/// Effect settings of a clip's chain that can move over time, with their menu labels.
pub fn fx_choices(cat: &Catalog, clip: &Value) -> Vec<(FxLane, String)> {
    let mut out = Vec::new();
    for slot in slots(clip) {
        let Some(info) = slot["effect"].as_str().and_then(|e| cat.0.get(e)) else {
            continue;
        };
        for p in params_of(info) {
            if truthy(&p["hidden"]) || truthy(&p["retimes"]) {
                continue;
            }
            out.push((
                FxLane {
                    slot: slot["id"].as_str().unwrap_or("").to_string(),
                    param: p["key"].as_str().unwrap_or("").to_string(),
                },
                format!(
                    "{}: {}",
                    info["name"].as_str().unwrap_or(""),
                    p["name"].as_str().unwrap_or("")
                ),
            ));
        }
    }
    out
}

/// The effect envelope shown on `clip`: the one picked for it, else the first one it has.
pub fn fx_lane_of(cat: &Catalog, clip: &Value, picked: Option<&FxLane>) -> Option<FxLane> {
    if let Some(p) = picked {
        if fx_param(cat, clip, Some(p)).is_some() {
            return Some(p.clone());
        }
    }
    envelopes(clip)
        .iter()
        .find(|e| e["target"]["type"].as_str() == Some("effect_param") && truthy(&e["enabled"]))
        .map(|e| FxLane {
            slot: e["target"]["slot_id"].as_str().unwrap_or("").to_string(),
            param: e["target"]["param"].as_str().unwrap_or("").to_string(),
        })
}

/// Range of an envelope; effect envelopes take the control's range (high at the top).
pub fn range_for(
    kind: EnvKind,
    cat: &Catalog,
    clip: Option<&Value>,
    lane: Option<&FxLane>,
) -> EnvRange {
    if kind != EnvKind::Fx {
        return range_of(kind);
    }
    match clip.and_then(|c| fx_param(cat, c, lane)) {
        None => EnvRange {
            top: 1.0,
            bottom: 0.0,
            step: 0.01,
        },
        Some(fp) => EnvRange {
            top: fp.max,
            bottom: fp.min,
            step: if fp.step > 0.0 { fp.step } else { 0.01 },
        },
    }
}

/// The points of the enabled envelope of this kind (or of this effect lane).
pub fn points_of(clip: &Value, kind: EnvKind, lane: Option<&FxLane>) -> Vec<Value> {
    let found = if kind == EnvKind::Fx {
        let Some(l) = lane else {
            return Vec::new();
        };
        envelopes(clip).iter().find(|e| {
            e["target"]["type"].as_str() == Some("effect_param")
                && e["target"]["slot_id"].as_str() == Some(l.slot.as_str())
                && e["target"]["param"].as_str() == Some(l.param.as_str())
                && truthy(&e["enabled"])
        })
    } else {
        let name = if kind == EnvKind::Volume {
            "volume"
        } else {
            "pan"
        };
        envelopes(clip)
            .iter()
            .find(|e| e["target"]["type"].as_str() == Some(name) && truthy(&e["enabled"]))
    };
    found
        .and_then(|e| e["points"].as_array())
        .cloned()
        .unwrap_or_default()
}

/// The value a line with no points sits at: 0 dB, centered, or the effect's own setting.
pub fn rest_value(
    kind: EnvKind,
    cat: &Catalog,
    clip: Option<&Value>,
    lane: Option<&FxLane>,
) -> f64 {
    match (kind, clip) {
        (EnvKind::Fx, Some(c)) => fx_param(cat, c, lane).map_or(0.0, |fp| fp.value),
        _ => 0.0,
    }
}

/// The clip change that sets an envelope's points. An effect envelope needs its lane.
pub fn env_change(
    clip: &Value,
    kind: EnvKind,
    points: &[Value],
    lane: Option<&FxLane>,
) -> Option<Value> {
    let id = &clip["id"];
    Some(match kind {
        EnvKind::Volume => json!({ "id": id, "volume": points }),
        EnvKind::Pan => json!({ "id": id, "pan": points }),
        EnvKind::Fx => {
            let l = lane?;
            json!({ "id": id, "effect_points": [{ "slot_id": l.slot, "param": l.param, "points": points }] })
        }
    })
}

/// Name of an envelope for menus and undo.
pub fn env_name(
    kind: EnvKind,
    cat: &Catalog,
    clip: Option<&Value>,
    lane: Option<&FxLane>,
) -> String {
    match kind {
        EnvKind::Volume => "Volume".into(),
        EnvKind::Pan => "Pan".into(),
        EnvKind::Fx => match clip.and_then(|c| fx_param(cat, c, lane)) {
            Some(fp) => format!("{} {}", fp.effect, fp.param),
            None => "Effect".into(),
        },
    }
}
