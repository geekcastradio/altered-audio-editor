// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Process chains: a named list of editing steps saved as plain JSON. Each step names an action
//! and gives typed parameters. The same action registry produces a JSON Schema per action, so the
//! chain file and a future tool endpoint (MCP) describe actions in one way.
//!
//! This module checks a chain against the registry. Running a step belongs to the app, which maps
//! an action name to the command the menu already uses. Every step is one undo step and a chain
//! is one more around them.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

pub const CHAIN_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chain {
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    /// Action id from the registry, such as `multitrack.close-dead-space`.
    pub action: String,
    #[serde(default)]
    pub params: Map<String, Value>,
    /// Stop the chain (the default) or carry on when this step fails.
    #[serde(default)]
    pub on_error: OnError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OnError {
    #[default]
    Stop,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Number,
    Integer,
    Bool,
    Text,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: &'static str,
    pub kind: Kind,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub default: Option<Value>,
    pub help: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub id: &'static str,
    pub title: &'static str,
    /// True when the step changes nothing until applied and has a preview.
    pub destructive: bool,
    pub params: Vec<Param>,
}

fn num(name: &'static str, min: f64, max: f64, default: f64, help: &'static str) -> Param {
    Param {
        name,
        kind: Kind::Number,
        min: Some(min),
        max: Some(max),
        default: Some(json!(default)),
        help,
    }
}

fn text(name: &'static str, default: &str, help: &'static str) -> Param {
    Param {
        name,
        kind: Kind::Text,
        min: None,
        max: None,
        default: Some(json!(default)),
        help,
    }
}

/// The actions a chain can call. A draft list: ids and parameters are matched to the real
/// commands when the runner is built.
pub fn registry() -> Vec<Action> {
    vec![
        Action {
            id: "multitrack.close-dead-space",
            title: "Close Dead Space",
            destructive: true,
            params: vec![num(
                "pause_seconds",
                0.0,
                2.0,
                0.5,
                "Pause left where every track was silent",
            )],
        },
        Action {
            id: "multitrack.auto-mic-mix",
            title: "Auto Mic Mix",
            destructive: true,
            params: vec![
                num(
                    "open_db",
                    0.0,
                    24.0,
                    8.0,
                    "How far under a track must be to count as quiet",
                ),
                num(
                    "margin_db",
                    0.0,
                    24.0,
                    6.0,
                    "Tracks within this of the loudest stay open",
                ),
                num(
                    "reduce_db",
                    0.0,
                    36.0,
                    12.0,
                    "How far quiet tracks are turned down",
                ),
                num("fade_ms", 10.0, 1000.0, 150.0, "Fade back up"),
                num("attack_ms", 1.0, 200.0, 15.0, "Fade down"),
                num("hold_ms", 0.0, 1000.0, 250.0, "Hold open after speech"),
            ],
        },
        Action {
            id: "multitrack.loudness-target",
            title: "Loudness Target",
            destructive: true,
            params: vec![
                text(
                    "target",
                    "podcast-stereo",
                    "Which target: podcast-stereo, podcast-mono, spotify, broadcast or custom",
                ),
                num(
                    "tolerance_db",
                    0.0,
                    3.0,
                    0.5,
                    "A mix this close to the target is left alone",
                ),
            ],
        },
        Action {
            id: "multitrack.voice-chain-preset",
            title: "Voice Chain Preset",
            destructive: true,
            params: vec![
                text(
                    "preset",
                    "Podcast Voice",
                    "Name of a built-in or saved Voice Chain preset",
                ),
                text(
                    "tracks",
                    "all",
                    "A track's name, or all for every track that has audio",
                ),
            ],
        },
        Action {
            id: "multitrack.quickpass",
            title: "QuickPass",
            destructive: true,
            params: vec![
                text(
                    "preset",
                    "Clean Up Podcast Interview",
                    "Name of a QuickPass preset that removes or turns down what it finds",
                ),
                text(
                    "ripple",
                    "none",
                    "After a removal: none keeps every track in step, track closes the gap on that track, all closes it everywhere",
                ),
            ],
        },
    ]
}

#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    /// Zero-based step index, or `None` for the chain as a whole.
    pub step: Option<usize>,
    pub message: String,
}

fn check_param(spec: &Param, value: &Value) -> Option<String> {
    match spec.kind {
        Kind::Bool => (!value.is_boolean()).then(|| format!("{} must be true or false", spec.name)),
        Kind::Text => (!value.is_string()).then(|| format!("{} must be text", spec.name)),
        Kind::Number | Kind::Integer => {
            let Some(n) = value.as_f64() else {
                return Some(format!("{} must be a number", spec.name));
            };
            if spec.kind == Kind::Integer && n.fract() != 0.0 {
                return Some(format!("{} must be a whole number", spec.name));
            }
            if spec.min.is_some_and(|m| n < m) || spec.max.is_some_and(|m| n > m) {
                return Some(format!(
                    "{} must be between {} and {}",
                    spec.name,
                    spec.min.unwrap_or(f64::NEG_INFINITY),
                    spec.max.unwrap_or(f64::INFINITY)
                ));
            }
            None
        }
    }
}

/// Everything wrong with a chain, empty when it can run.
pub fn validate(chain: &Chain) -> Vec<Problem> {
    let reg = registry();
    let mut out = Vec::new();
    if chain.version == 0 || chain.version > CHAIN_VERSION {
        out.push(Problem {
            step: None,
            message: format!(
                "This chain is version {}; this app reads up to {}",
                chain.version, CHAIN_VERSION
            ),
        });
    }
    if chain.name.trim().is_empty() {
        out.push(Problem {
            step: None,
            message: "The chain needs a name".into(),
        });
    }
    if chain.steps.is_empty() {
        out.push(Problem {
            step: None,
            message: "The chain has no steps".into(),
        });
    }
    for (i, step) in chain.steps.iter().enumerate() {
        let Some(action) = reg.iter().find(|a| a.id == step.action) else {
            out.push(Problem {
                step: Some(i),
                message: format!("Unknown action {}", step.action),
            });
            continue;
        };
        for key in step.params.keys() {
            if !action.params.iter().any(|p| p.name == key) {
                out.push(Problem {
                    step: Some(i),
                    message: format!("{} has no setting called {key}", action.title),
                });
            }
        }
        for spec in &action.params {
            if let Some(value) = step.params.get(spec.name) {
                if let Some(message) = check_param(spec, value) {
                    out.push(Problem {
                        step: Some(i),
                        message,
                    });
                }
            }
        }
    }
    out
}

/// A step's parameters with the defaults filled in.
pub fn resolved(step: &Step) -> Option<Map<String, Value>> {
    let reg = registry();
    let action = reg.iter().find(|a| a.id == step.action)?;
    let mut out = Map::new();
    for spec in &action.params {
        let value = step
            .params
            .get(spec.name)
            .cloned()
            .or_else(|| spec.default.clone())?;
        out.insert(spec.name.to_string(), value);
    }
    Some(out)
}

/// A JSON Schema for each action, the form a tool endpoint publishes.
pub fn action_schemas() -> Value {
    let list: Vec<Value> = registry()
        .iter()
        .map(|a| {
            let mut props = Map::new();
            for p in &a.params {
                let mut s = Map::new();
                s.insert(
                    "type".into(),
                    json!(match p.kind {
                        Kind::Number => "number",
                        Kind::Integer => "integer",
                        Kind::Bool => "boolean",
                        Kind::Text => "string",
                    }),
                );
                if let Some(m) = p.min {
                    s.insert("minimum".into(), json!(m));
                }
                if let Some(m) = p.max {
                    s.insert("maximum".into(), json!(m));
                }
                if let Some(d) = &p.default {
                    s.insert("default".into(), d.clone());
                }
                s.insert("description".into(), json!(p.help));
                props.insert(p.name.into(), Value::Object(s));
            }
            json!({
                "name": a.id,
                "description": a.title,
                "inputSchema": { "type": "object", "properties": props, "additionalProperties": false }
            })
        })
        .collect();
    Value::Array(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chain(json_text: &str) -> Chain {
        serde_json::from_str(json_text).unwrap()
    }

    const GOOD: &str = r#"{
        "version": 1,
        "name": "Episode cleanup",
        "steps": [
            { "action": "multitrack.auto-mic-mix", "params": { "reduce_db": 10 } },
            { "action": "multitrack.close-dead-space", "params": { "pause_seconds": 0.4 }, "on_error": "skip" }
        ]
    }"#;

    #[test]
    fn a_good_chain_has_no_problems_and_fills_defaults() {
        let c = chain(GOOD);
        assert!(validate(&c).is_empty(), "{:?}", validate(&c));
        let r = resolved(&c.steps[0]).unwrap();
        assert_eq!(r["reduce_db"], json!(10));
        assert_eq!(r["open_db"], json!(8.0));
        assert_eq!(c.steps[1].on_error, OnError::Skip);
        assert_eq!(c.steps[0].on_error, OnError::Stop);
    }

    #[test]
    fn the_preset_steps_take_names_and_have_defaults() {
        let c = chain(
            r#"{ "version": 1, "name": "x", "steps": [
                { "action": "multitrack.voice-chain-preset", "params": { "tracks": "Host" } },
                { "action": "multitrack.quickpass" }
            ] }"#,
        );
        assert!(validate(&c).is_empty(), "{:?}", validate(&c));
        let a = resolved(&c.steps[0]).unwrap();
        assert_eq!(a["preset"], json!("Podcast Voice"));
        assert_eq!(a["tracks"], json!("Host"));
        let b = resolved(&c.steps[1]).unwrap();
        assert_eq!(b["preset"], json!("Clean Up Podcast Interview"));
        assert_eq!(b["ripple"], json!("none"));
    }

    #[test]
    fn the_loudness_step_is_in_the_registry_with_its_defaults() {
        let c = chain(
            r#"{ "version": 1, "name": "x", "steps": [
                { "action": "multitrack.loudness-target" },
                { "action": "multitrack.loudness-target", "params": { "target": 5 } }
            ] }"#,
        );
        let r = resolved(&c.steps[0]).unwrap();
        assert_eq!(r["target"], json!("podcast-stereo"));
        assert_eq!(r["tolerance_db"], json!(0.5));
        let m: Vec<String> = validate(&c).into_iter().map(|p| p.message).collect();
        assert_eq!(m, ["target must be text"]);
    }

    #[test]
    fn a_chain_round_trips_through_json() {
        let c = chain(GOOD);
        let back: Chain = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn mistakes_are_named_per_step() {
        let c = chain(
            r#"{ "version": 1, "name": "x", "steps": [
                { "action": "nope" },
                { "action": "multitrack.close-dead-space", "params": { "pause_seconds": 9 } },
                { "action": "multitrack.close-dead-space", "params": { "pause_seconds": "long", "extra": 1 } },
                { "action": "multitrack.auto-mic-mix", "params": { "fade_ms": true } }
            ] }"#,
        );
        let problems = validate(&c);
        let text: Vec<String> = problems
            .iter()
            .map(|p| format!("{:?}: {}", p.step, p.message))
            .collect();
        let joined = text.join("\n");
        assert!(joined.contains("Some(0): Unknown action nope"), "{joined}");
        assert!(
            joined.contains("Some(1): pause_seconds must be between 0 and 2"),
            "{joined}"
        );
        assert!(
            joined.contains("Some(2): pause_seconds must be a number"),
            "{joined}"
        );
        assert!(joined.contains("no setting called extra"), "{joined}");
        assert!(
            joined.contains("Some(3): fade_ms must be a number"),
            "{joined}"
        );
    }

    #[test]
    fn empty_nameless_and_newer_chains_are_refused() {
        let c = chain(r#"{ "version": 2, "name": " ", "steps": [] }"#);
        let m: Vec<String> = validate(&c).into_iter().map(|p| p.message).collect();
        assert_eq!(m.len(), 3, "{m:?}");
    }

    #[test]
    fn the_registry_publishes_one_schema_per_action() {
        let s = action_schemas();
        let list = s.as_array().unwrap();
        assert_eq!(list.len(), registry().len());
        let mix = list
            .iter()
            .find(|a| a["name"] == "multitrack.auto-mic-mix")
            .unwrap();
        assert_eq!(
            mix["inputSchema"]["properties"]["reduce_db"]["maximum"],
            json!(36.0)
        );
        assert_eq!(mix["inputSchema"]["additionalProperties"], json!(false));
        // Ids are unique and every default passes its own check.
        let mut ids: Vec<_> = registry().iter().map(|a| a.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), registry().len());
        for a in registry() {
            for p in &a.params {
                if let Some(d) = &p.default {
                    assert!(check_param(p, d).is_none(), "{} {}", a.id, p.name);
                }
            }
        }
    }
}
