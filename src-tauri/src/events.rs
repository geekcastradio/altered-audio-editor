// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! How the core tells the interface something happened (progress bars,
//! files handed over by the system, scan results).
//!
//! The core only knows [`EventSink`]. Today the one implementation is the
//! Tauri window; a native interface can supply its own without any command
//! changing. Payloads travel as JSON values so the trait stays object safe.

use serde::Serialize;

/// Where events go.
pub trait EventSink: Send + Sync {
    /// Delivers one named event. Errors mean the interface is gone; callers ignore them.
    fn emit_json(&self, name: &str, payload: serde_json::Value) -> Result<(), String>;
}

/// Sends any serializable payload to an [`EventSink`].
pub trait EventSinkExt {
    fn emit_event<T: Serialize>(&self, name: &str, payload: T) -> Result<(), String>;
}

impl<S: EventSink + ?Sized> EventSinkExt for S {
    fn emit_event<T: Serialize>(&self, name: &str, payload: T) -> Result<(), String> {
        let value = serde_json::to_value(payload).map_err(|e| e.to_string())?;
        self.emit_json(name, value)
    }
}

impl EventSink for tauri::AppHandle {
    fn emit_json(&self, name: &str, payload: serde_json::Value) -> Result<(), String> {
        tauri::Emitter::emit(self, name, payload).map_err(|e| e.to_string())
    }
}

/// Collects events in memory. For tests, and for a headless run.
#[derive(Default)]
pub struct MemorySink(pub std::sync::Mutex<Vec<(String, serde_json::Value)>>);

impl EventSink for MemorySink {
    fn emit_json(&self, name: &str, payload: serde_json::Value) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .push((name.to_string(), payload));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Progress {
        done: u64,
        total: u64,
    }

    #[test]
    fn a_memory_sink_keeps_events_in_order() {
        let sink = MemorySink::default();
        sink.emit_event("export-progress", Progress { done: 1, total: 4 })
            .unwrap();
        sink.emit_event("export-progress", Progress { done: 2, total: 4 })
            .unwrap();
        let got = sink.0.lock().unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].0, "export-progress");
        assert_eq!(got[1].1["done"], 2);
    }

    #[test]
    fn works_through_a_trait_object() {
        let sink: Box<dyn EventSink> = Box::new(MemorySink::default());
        assert!(sink.emit_event("x", 5).is_ok());
    }
}
