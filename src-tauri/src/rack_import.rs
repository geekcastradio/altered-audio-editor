// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Reads the effect chains (racks) saved in another editor's effect presets
//! file and turns them into Voice Chain presets. A rack lists its effects by
//! name and keeps each effect's settings as numbers in that editor's own scale,
//! or as opaque binary. Only the settings we have worked out are read, so a
//! chain comes across with the same effects in the same order and our own
//! starting values, except where the scale is known (the limiter's ceiling).
//!
//! Mirrors `src/panels/voice-chain/audition-presets.ts`; the shared cases in
//! `shared-fixtures/rack-import.json` keep both sides the same.

use crate::js_compat::{js_round, js_string_to_number, js_trim};
use crate::sound_types::{clean_noise, made, Made};
use serde_json::{json, Map, Value};

/// One effect in a rack.
#[derive(Clone, Debug, PartialEq)]
pub struct RackSlot {
    pub name: String,
    /// The effect's identifier in the file, lower case. This is what we match on.
    pub id: String,
    pub on: bool,
    /// The settings as plain numbers, or None when they were kept as binary.
    pub values: Option<Vec<f64>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rack {
    pub name: String,
    pub slots: Vec<RackSlot>,
}

struct Node {
    tag: String,
    text: String,
    kids: Vec<usize>,
}

/// A small reader for the tags-and-text XML these files use. Attributes are
/// ignored. Anything that is not a tag is text on the open tag.
struct Doc {
    nodes: Vec<Node>,
}

fn is_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'.'
}

fn parse(xml: &str) -> Doc {
    let mut nodes = vec![Node {
        tag: "#root".into(),
        text: String::new(),
        kids: Vec::new(),
    }];
    let mut stack = vec![0usize];
    let b = xml.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'<' {
            let mut j = i + 1;
            let closing = b.get(j) == Some(&b'/');
            if closing {
                j += 1;
            }
            let mut k = j;
            while k < b.len() && is_name_byte(b[k]) {
                k += 1;
            }
            let end = if k > j {
                b[k..].iter().position(|&c| c == b'>').map(|p| k + p)
            } else {
                None
            };
            match end {
                Some(p) => {
                    if closing {
                        if stack.len() > 1 {
                            stack.pop();
                        }
                    } else {
                        let self_closing = p > k && b[p - 1] == b'/';
                        let id = nodes.len();
                        nodes.push(Node {
                            tag: xml[j..k].to_string(),
                            text: String::new(),
                            kids: Vec::new(),
                        });
                        let top = *stack.last().unwrap();
                        nodes[top].kids.push(id);
                        if !self_closing {
                            stack.push(id);
                        }
                    }
                    i = p + 1;
                }
                // Not a tag: the `<` is dropped and what follows reads as text.
                None => i += 1,
            }
        } else {
            let stop = b[i..]
                .iter()
                .position(|&c| c == b'<')
                .map_or(b.len(), |p| i + p);
            let top = *stack.last().unwrap();
            nodes[top].text.push_str(&xml[i..stop]);
            i = stop;
        }
    }
    Doc { nodes }
}

fn decode(s: &str) -> String {
    js_trim(
        &s.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&amp;", "&"),
    )
    .to_string()
}

impl Doc {
    fn tag(&self, n: usize) -> &str {
        &self.nodes[n].tag
    }

    fn kid(&self, n: usize, tag: &str) -> Option<usize> {
        self.nodes[n]
            .kids
            .iter()
            .copied()
            .find(|&k| self.tag(k) == tag)
    }

    /// A prop.list as key to value node, in file order. A repeated key keeps its
    /// first place and takes the last value.
    fn pairs(&self, list: Option<usize>) -> Vec<(String, usize)> {
        let mut out: Vec<(String, usize)> = Vec::new();
        let Some(list) = list else { return out };
        for &p in &self.nodes[list].kids {
            if self.tag(p) != "prop.pair" {
                continue;
            }
            let key = self.kid(p, "key");
            let val = self.nodes[p]
                .kids
                .iter()
                .copied()
                .find(|&k| self.tag(k) != "key");
            if let (Some(key), Some(val)) = (key, val) {
                let name = decode(&self.nodes[key].text);
                match out.iter_mut().find(|(k, _)| *k == name) {
                    Some(slot) => slot.1 = val,
                    None => out.push((name, val)),
                }
            }
        }
        out
    }
}

fn get(pairs: &[(String, usize)], key: &str) -> Option<usize> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| *v)
}

/// `^SlotNumber_(\d+)$`: the number as a float, like JavaScript's `Number`.
fn slot_number(key: &str) -> Option<f64> {
    let digits = key.strip_prefix("SlotNumber_")?;
    if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse::<f64>().ok()
}

fn walk(doc: &Doc, node: usize, slots: &mut Vec<(f64, RackSlot)>) {
    for (key, val) in doc.pairs(Some(node)) {
        let is_list = doc.tag(val) == "prop.list";
        match slot_number(&key) {
            Some(n) if is_list => {
                let f = doc.pairs(Some(val));
                let Some(dn) = get(&f, "DisplayName") else {
                    continue;
                };
                let id = get(&f, "EffectID")
                    .map(|n| decode(&doc.nodes[n].text))
                    .unwrap_or_default()
                    .to_lowercase();
                let on = get(&f, "EffectPower").is_none_or(|n| doc.tag(n) != "false");
                let param_list = get(&doc.pairs(get(&f, "EffectSettings")), "ParamList");
                let values = param_list.and_then(|arr| {
                    let nums: Vec<Option<f64>> = doc.nodes[arr]
                        .kids
                        .iter()
                        .filter(|&&k| matches!(doc.tag(k), "float" | "double"))
                        .map(|&k| js_string_to_number(&doc.nodes[k].text))
                        .collect();
                    nums.iter()
                        .all(|v| v.is_some_and(f64::is_finite))
                        .then(|| nums.into_iter().flatten().collect::<Vec<f64>>())
                });
                slots.push((
                    n,
                    RackSlot {
                        name: decode(&doc.nodes[dn].text),
                        id,
                        on,
                        values,
                    },
                ));
            }
            _ if is_list => walk(doc, val, slots),
            _ => {}
        }
    }
}

/// The racks in the file, without the editor's own "(Default)" and "(Last Applied Rack)".
pub fn read_racks(xml: &str) -> Vec<Rack> {
    let doc = parse(xml);
    let map = doc.kid(0, "prop.map");
    let top = doc.pairs(map.and_then(|m| doc.kid(m, "prop.list")));
    let racks = doc.pairs(get(&doc.pairs(get(&top, "EffectsRackPresetID")), "Presets"));
    let mut out = Vec::new();
    for (name, preset) in racks {
        if name == "(Default)" || name == "(Last Applied Rack)" {
            continue;
        }
        let mut slots: Vec<(f64, RackSlot)> = Vec::new();
        walk(&doc, preset, &mut slots);
        if !slots.is_empty() {
            // A stable sort, so equal numbers keep file order.
            slots.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            out.push(Rack {
                name,
                slots: slots.into_iter().map(|s| s.1).collect(),
            });
        }
    }
    out
}

fn plain(effect: &str) -> Made {
    made(effect, json!({}))
}

fn limiter_from(v: Option<&[f64]>) -> Made {
    // The limiter keeps its ceiling as 1 + dB/100 (0.999 is -0.1 dB, 0.94 is -6 dB).
    let db = match v {
        Some(v) if !v.is_empty() => js_round((v[0] - 1.0) * 1000.0) / 10.0,
        _ => -0.1,
    };
    made(
        "limiter",
        json!({ "ceiling": db.clamp(-24.0, 0.0), "lookahead": 5, "release": 100 }),
    )
}

/// The effects we have an equivalent for, by the identifier the file stores
/// (never by name, so no one else's feature names live in this source).
fn by_id(id: &str, v: Option<&[f64]>) -> Option<Made> {
    Some(match id {
        "57c2006b-d675-40bc-a1aa-f7764ef79959" | "e0b23f05-f1a7-4ef7-9b50-7ec3e3002058" => {
            limiter_from(v)
        }
        "3034b5c2-23a7-43b6-a1ef-9c4047f4368d" => plain("speech_leveler"),
        "c6762838-e451-4d22-9c90-b0eaa734605a" => clean_noise(10.0),
        "f76dad9c-77da-477a-9939-eefefc5c23bd" => plain("dynamics"),
        "3981e750-c8ae-40a2-889f-41a01c7efa03" => plain("multiband"),
        "8edcd326-33de-4a4d-985b-f1bf48ed1427" => plain("parametric_eq"),
        "4f327230-f04c-4c34-9ea5-a998b4459221" => plain("curve_eq"),
        "ee38cdc5-5914-4a75-b7dd-73a265e55c31" => plain("notch"),
        "ffbe710f-cd69-4139-ad26-65603616d9d4" => plain("deesser"),
        "f14fee7d-fe5f-4202-b790-f53209e56411" => plain("gain"),
        "f205d7b7-5e0b-421b-bc6d-5f27096ba192"
        | "5e85f068-b39d-4c63-9912-2030f6f0041e"
        | "c4ad21d8-9748-4d79-b4a3-d88ab7e4be34" => plain("echo"),
        "31d33f85-3288-489e-8390-d03c60d9ce6c" => plain("chorus"),
        "d30b6731-3e00-4a11-9650-e816f58a9d12" => plain("flanger"),
        "9b66ab13-4d1b-469a-88c6-b807fcf1df16" => plain("distortion"),
        "164b4e8a-7105-406b-b23b-1ef2cc4d8957" => plain("pitch_shift"),
        "433322b0-d25a-4cfa-a4f1-4a1ab2d7615b" => plain("click_repair"),
        "0a1016a3-74e6-4c93-8b6c-e058a1419bf1" => plain("center_remover"),
        "7354704a-d87c-11e0-b05b-763c4824019b" => plain("phase_curve"),
        _ => return None,
    })
}

/// A few effects carry a plug-in style id and are matched by display name.
fn by_name(name: &str) -> Option<Made> {
    (name == "Phaser").then(|| plain("phaser"))
}

/// A Voice Chain preset made from one rack.
#[derive(Clone, Debug)]
pub struct RackPreset {
    pub name: String,
    pub slots: Vec<Made>,
}

#[derive(Clone, Debug)]
pub struct RackImport {
    pub presets: Vec<RackPreset>,
    /// Effect names with no match here, with how many chains used them.
    pub missing: Vec<(String, usize)>,
    /// Chains in the file that came across.
    pub chains: usize,
}

/// Voice Chain presets from the racks in a file, named "<rack> (Audition)".
pub fn import_racks(xml: &str) -> RackImport {
    let mut missing: Vec<(String, usize)> = Vec::new();
    let mut presets = Vec::new();
    for rack in read_racks(xml) {
        let mut slots = Vec::new();
        for s in &rack.slots {
            let built = by_id(&s.id, s.values.as_deref()).or_else(|| by_name(&s.name));
            let Some(built) = built else {
                match missing.iter_mut().find(|(n, _)| *n == s.name) {
                    Some(m) => m.1 += 1,
                    None => missing.push((s.name.clone(), 1)),
                }
                continue;
            };
            // An effect switched off in the rack stays out, so the chain sounds as it did there.
            if s.on {
                slots.push(built);
            }
        }
        if !slots.is_empty() {
            presets.push(RackPreset {
                name: format!("{} (Audition)", rack.name),
                slots,
            });
        }
    }
    // Most used first; equal counts keep the order they were first seen in.
    missing.sort_by_key(|m| std::cmp::Reverse(m.1));
    RackImport {
        chains: presets.len(),
        presets,
        missing,
    }
}

/// The result in the same shape the TypeScript side returns, for comparing.
pub fn import_to_value(r: &RackImport) -> Value {
    let presets: Vec<Value> = r
        .presets
        .iter()
        .map(|p| {
            let slots: Vec<Value> = p
                .slots
                .iter()
                .map(|m| json!({ "effect": m.effect, "params": Value::Object(m.params.clone()) }))
                .collect();
            json!({
                "name": p.name,
                "group": "Voice chains",
                "chain": {
                    "enabled": true, "input_gain_db": 0, "output_gain_db": 0, "mix": 1,
                    "slots": slots,
                },
            })
        })
        .collect();
    let missing: Vec<Value> = r
        .missing
        .iter()
        .map(|(n, c)| json!({ "name": n, "count": c }))
        .collect();
    json!({ "presets": presets, "missing": missing, "chains": r.chains })
}

/// The racks in the shape the TypeScript side returns, for comparing.
pub fn racks_to_value(racks: &[Rack]) -> Value {
    Value::Array(
        racks
            .iter()
            .map(|r| {
                let slots: Vec<Value> = r
                    .slots
                    .iter()
                    .map(|s| {
                        let mut m = Map::new();
                        m.insert("name".into(), json!(s.name));
                        m.insert("id".into(), json!(s.id));
                        m.insert("on".into(), json!(s.on));
                        m.insert("values".into(), json!(s.values));
                        Value::Object(m)
                    })
                    .collect();
                json!({ "name": r.name, "slots": slots })
            })
            .collect(),
    )
}
