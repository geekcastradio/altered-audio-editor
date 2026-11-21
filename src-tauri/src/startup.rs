// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! How long each stage of starting the program takes.
//!
//! Each stage is stamped with the time since the process began and written to
//! `startup-timing.log` in the settings folder, replacing the one from the
//! last launch. The file stays on this computer; nothing is sent anywhere. It
//! exists so a slow launch can be read stage by stage before anyone guesses.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

struct State {
    began: Instant,
    marks: Vec<(u128, String)>,
    file: Option<PathBuf>,
}

fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(State {
            began: Instant::now(),
            marks: Vec::new(),
            file: None,
        })
    })
}

/// Text of the log: one line per stage, in the order they happened.
pub fn render(marks: &[(u128, String)]) -> String {
    let mut out = String::from("Startup timing (milliseconds since the process began)\n");
    let mut last = 0u128;
    for (at, stage) in marks {
        out.push_str(&format!(
            "{:>7} ms  +{:<6} {}\n",
            at,
            at.saturating_sub(last),
            stage
        ));
        last = *at;
    }
    out
}

/// The log as text, for Help > Copy Bug Report Details.
pub fn text() -> String {
    let s = state().lock().unwrap_or_else(|e| e.into_inner());
    render(&s.marks)
}

/// Starts the clock. Call first thing in `run`.
pub fn begin() {
    let _ = state();
    mark("process started");
}

/// Stamps a stage. Safe to call from any thread, and cheap.
pub fn mark(stage: &str) {
    let mut s = state().lock().unwrap_or_else(|e| e.into_inner());
    let at = s.began.elapsed().as_millis();
    s.marks.push((at, stage.to_string()));
    if let Some(file) = &s.file {
        let _ = std::fs::write(file, render(&s.marks));
    }
}

/// Says where the log goes and writes what was stamped so far.
pub fn set_folder(dir: &Path) {
    let mut s = state().lock().unwrap_or_else(|e| e.into_inner());
    let file = dir.join("startup-timing.log");
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(&file, render(&s.marks));
    s.file = Some(file);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_has_the_header_and_the_stages_so_far() {
        mark("test stage for text");
        let t = text();
        assert!(t.starts_with("Startup timing"));
        assert!(t.contains("test stage for text"));
    }

    #[test]
    fn render_lists_each_stage_with_its_gap() {
        let text = render(&[(0, "a".to_string()), (120, "b".to_string())]);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("0 ms") && lines[1].ends_with(" a"));
        assert!(
            lines[2].contains("120 ms") && lines[2].contains("+120") && lines[2].ends_with(" b")
        );
    }

    #[test]
    fn marks_are_written_once_a_folder_is_set() {
        let dir = std::env::temp_dir().join(format!("aae-startup-{}", std::process::id()));
        mark("before folder");
        set_folder(&dir);
        mark("after folder");
        let text = std::fs::read_to_string(dir.join("startup-timing.log")).unwrap();
        assert!(text.contains("before folder"));
        assert!(text.contains("after folder"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
