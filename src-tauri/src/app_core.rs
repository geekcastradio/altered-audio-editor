// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The handle the core hands to any interface.
//!
//! Phase 0 of the move to a native interface: logic that today reaches for
//! the Tauri window moves behind this handle one piece at a time. A command
//! then becomes a thin wrapper that builds nothing and decides nothing. The
//! handle starts with the event sink; shared state joins it as commands move.

use std::sync::Arc;

use serde::Serialize;

use crate::events::{EventSink, EventSinkExt};

/// What core logic may use to reach the interface it runs under.
#[derive(Clone)]
pub struct AppCore {
    sink: Arc<dyn EventSink>,
}

impl AppCore {
    pub fn new(sink: Arc<dyn EventSink>) -> Self {
        Self { sink }
    }

    /// The place events go.
    pub fn sink(&self) -> &dyn EventSink {
        self.sink.as_ref()
    }

    /// Sends one named event. Errors mean the interface is gone; callers ignore them.
    pub fn emit<T: Serialize>(&self, name: &str, payload: T) -> Result<(), String> {
        self.sink.emit_event(name, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::MemorySink;

    #[test]
    fn emits_through_the_sink() {
        let mem = Arc::new(MemorySink::default());
        let core = AppCore::new(mem.clone());
        core.emit("export-progress", 3).unwrap();
        let got = mem.0.lock().unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, "export-progress");
    }

    #[test]
    fn clones_share_one_sink() {
        let mem = Arc::new(MemorySink::default());
        let a = AppCore::new(mem.clone());
        let b = a.clone();
        a.emit("a", 1).unwrap();
        b.emit("b", 2).unwrap();
        assert_eq!(mem.0.lock().unwrap().len(), 2);
    }
}
