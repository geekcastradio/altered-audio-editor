// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! What a key press does: the Rust copy of `decide` in `src/keymap/dispatch.ts` and
//! `ownsArrowKeys` in `src/keymap/owns-arrows.ts`. The parts that read the page (is the
//! target a text field) stay in the interface; this takes what they found. Tested
//! against `shared-fixtures/key-dispatch.json`.

use crate::chord::{chord_from_event, KeyEvent};

/// Actions that keep firing while the key is held.
const REPEATABLE: [&str; 8] = [
    "edit.undo",
    "edit.redo",
    "view.zoom-in",
    "view.zoom-out",
    "transport.step-left",
    "transport.step-right",
    "transport.page-left",
    "transport.page-right",
];

/// Webview keys that would reload, print or search the page. Blocked in release builds.
const BROWSER_KEYS: [&str; 9] = [
    "F5",
    "Ctrl+R",
    "Ctrl+Shift+R",
    "Ctrl+P",
    "Ctrl+G",
    "Ctrl+Shift+G",
    "Ctrl+U",
    "F3",
    "F7",
];

/// Roles whose widgets use the arrow and Page keys themselves.
const ARROW_ROLES: [&str; 15] = [
    "listbox",
    "option",
    "slider",
    "spinbutton",
    "tab",
    "tablist",
    "menu",
    "menuitem",
    "menubar",
    "tree",
    "treeitem",
    "grid",
    "gridcell",
    "radio",
    "radiogroup",
];

/// What to do with a key press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// The action to run, if any.
    pub action: Option<String>,
    /// Stop the webview's own handling of this key.
    pub prevent: bool,
}

/// What the interface knows about where the key was pressed.
#[derive(Debug, Clone, Copy, Default)]
pub struct Context {
    pub in_text_field: bool,
    pub modal_open: bool,
    pub release: bool,
}

fn nothing(prevent: bool) -> Decision {
    Decision {
        action: None,
        prevent,
    }
}

/// Decides what a key press does. `action_for` looks a chord up in the keymap.
pub fn decide(
    e: &KeyEvent,
    repeat: bool,
    default_prevented: bool,
    ctx: Context,
    action_for: &dyn Fn(&str) -> Option<String>,
) -> Decision {
    if default_prevented {
        return nothing(false);
    }
    let Some(chord) = chord_from_event(e) else {
        return nothing(false);
    };
    let browser_key = ctx.release && BROWSER_KEYS.contains(&chord.as_str());
    if ctx.in_text_field {
        return nothing(browser_key);
    }
    // Outside text fields Ctrl+A never selects the page's text, even under a dialog.
    if ctx.modal_open {
        return nothing(browser_key || chord == "Ctrl+A");
    }
    let Some(action) = action_for(&chord) else {
        return nothing(browser_key);
    };
    if repeat && !REPEATABLE.contains(&action.as_str()) {
        return nothing(true);
    }
    Decision {
        action: Some(action),
        prevent: true,
    }
}

/// True when the focused control uses the bare arrow and Page keys itself (a slider,
/// a drop-down, a list, a tab row), so the playhead keys must leave it alone.
/// `tag` is None when there is no element.
pub fn owns_arrow_keys(tag: Option<&str>, input_type: Option<&str>, role: Option<&str>) -> bool {
    let Some(tag) = tag else {
        return false;
    };
    let tag = tag.to_uppercase();
    if tag == "SELECT" {
        return true;
    }
    if tag == "INPUT" && matches!(input_type, Some("range") | Some("radio")) {
        return true;
    }
    matches!(role, Some(r) if ARROW_ROLES.contains(&r))
}
