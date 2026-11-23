// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The panel list, the built-in workspaces and the rules for placing a panel. The Rust
//! copy of the helpers in `src/app/docking/registry.ts`. The panel tables themselves are
//! `data/docking.json`, written from the TypeScript lists; a Vitest check fails when the two
//! differ, so a change to the lists means writing the file again. Tested against
//! `shared-fixtures/docking.json`.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Where to put a panel: next to a panel, inside its tab group, or at an edge of the window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Placement {
    Beside {
        reference: String,
        direction: String,
    },
    Edge {
        edge: String,
    },
}

/// One panel the app can show.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelDef {
    pub id: String,
    pub title: String,
    #[serde(rename = "inDefault")]
    pub in_default: bool,
    pub home: Vec<Placement>,
    pub pending: Option<String>,
}

/// One step of building a layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutStep {
    pub id: String,
    pub placement: Option<Placement>,
    #[serde(default)]
    pub inactive: Option<bool>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
}

/// A workspace that ships with the app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<LayoutStep>,
    pub version: u32,
}

/// A panel's smallest size in pixels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MinSize {
    pub w: f64,
    pub h: f64,
}

/// Everything in `data/docking.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Registry {
    pub panels: Vec<PanelDef>,
    pub layout: Vec<LayoutStep>,
    #[serde(rename = "layoutVersion")]
    pub layout_version: u32,
    pub presets: Vec<Preset>,
    #[serde(rename = "minSize")]
    pub min_size: HashMap<String, MinSize>,
}

/// The registry, read once.
pub fn registry() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| {
        serde_json::from_str(include_str!("../data/docking.json"))
            .expect("data/docking.json is valid")
    })
}

/// The panel with this id, if there is one.
pub fn panel_def(id: &str) -> Option<&'static PanelDef> {
    registry().panels.iter().find(|p| p.id == id)
}

/// Picks the first home placement that is possible with the panels currently open.
pub fn resolve_home(id: &str, open: &HashSet<String>) -> Placement {
    if let Some(def) = panel_def(id) {
        for place in &def.home {
            match place {
                Placement::Edge { .. } => return place.clone(),
                Placement::Beside { reference, .. } => {
                    if reference != id && open.contains(reference) {
                        return place.clone();
                    }
                }
            }
        }
    }
    Placement::Edge {
        edge: "right".into(),
    }
}

/// The built-in workspace with this id.
pub fn preset_by_id(id: &str) -> Option<&'static Preset> {
    registry().presets.iter().find(|p| p.id == id)
}

/// A file-safe id for a new workspace name that does not clash with `taken`.
pub fn workspace_id(name: &str, taken: &HashSet<String>) -> String {
    let lower = name.to_lowercase();
    let mut dashed = String::new();
    let mut in_run = false;
    for c in lower.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            dashed.push(c);
            in_run = false;
        } else if !in_run {
            dashed.push('-');
            in_run = true;
        }
    }
    let trimmed: String = dashed.trim_matches('-').chars().take(48).collect();
    let base = if trimmed.is_empty() {
        "workspace".to_string()
    } else {
        trimmed
    };
    let mut id = base.clone();
    let mut n = 2;
    while taken.contains(&id) {
        id = format!("{base}-{n}");
        n += 1;
    }
    id
}

/// The size limit for a group of panels: the largest of each.
pub fn group_min(ids: &[&str]) -> (f64, f64) {
    ids.iter().fold((0.0, 0.0), |(w, h), id| {
        let m = registry().min_size.get(*id);
        (w.max(m.map_or(0.0, |m| m.w)), h.max(m.map_or(0.0, |m| m.h)))
    })
}
