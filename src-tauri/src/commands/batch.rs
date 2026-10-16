// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Batch Convert: converts a list of files one after another on a worker
//! thread, reporting each file's progress as `batch-progress` events. A
//! file that fails is reported and the rest carry on.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Emitter, State};

use crate::export::convert::{convert_file, Converted};
use crate::export::encode::OutputFormat;

/// Set to stop the running batch after the current file.
#[derive(Default)]
pub struct BatchCancel(pub Arc<AtomicBool>);

#[derive(Debug, Clone, Deserialize)]
pub struct BatchJob {
    pub id: String,
    pub source: PathBuf,
    pub dest: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchProgress {
    id: String,
    /// "running", "done", "failed" or "stopped".
    status: &'static str,
    /// 0 to 1 while running.
    progress: f64,
    error: Option<String>,
    dest: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummary {
    pub done: usize,
    pub failed: usize,
    pub stopped: bool,
}

#[tauri::command]
pub async fn batch_convert(
    app: AppHandle,
    cancel: State<'_, BatchCancel>,
    jobs: Vec<BatchJob>,
    format: OutputFormat,
    overwrite: bool,
) -> Result<BatchSummary, String> {
    let flag = cancel.0.clone();
    flag.store(false, Ordering::Relaxed);
    spawn_blocking(move || {
        let mut summary = BatchSummary {
            done: 0,
            failed: 0,
            stopped: false,
        };
        for job in jobs {
            if flag.load(Ordering::Relaxed) {
                summary.stopped = true;
                break;
            }
            let send = |status: &'static str, progress: f64, error: Option<String>| {
                let _ = app.emit(
                    "batch-progress",
                    BatchProgress {
                        id: job.id.clone(),
                        status,
                        progress,
                        error,
                        dest: job.dest.clone(),
                    },
                );
            };
            send("running", 0.0, None);
            let mut last = 0.0;
            let r = convert_file(&job.source, &job.dest, &format, overwrite, &flag, |d, t| {
                let p = if t > 0 { d as f64 / t as f64 } else { 0.0 };
                if p - last >= 0.02 {
                    last = p;
                    send("running", p, None);
                }
            });
            match r {
                Ok(Converted::Done { .. }) => {
                    summary.done += 1;
                    send("done", 1.0, None);
                }
                Ok(Converted::Cancelled) => {
                    summary.stopped = true;
                    send("stopped", 0.0, None);
                    break;
                }
                Err(e) => {
                    summary.failed += 1;
                    send("failed", 0.0, Some(e));
                }
            }
        }
        summary
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_cancel(cancel: State<'_, BatchCancel>) {
    cancel.0.store(true, Ordering::Relaxed);
}
