// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Video pictures for the reference lane and the video panel. The decoding is
//! in `os-video/` (the decoders built into the OS); this file only keeps the
//! open readers and hands pictures to the page as raw bytes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use altered_os_video::{VideoInfo, VideoReader};
use serde::Serialize;
use tauri::async_runtime::spawn_blocking;
use tauri::ipc::Response;
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

/// What the page needs to know about a video file.
#[derive(Debug, Clone, Serialize)]
pub struct VideoMeta {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration: f64,
    pub has_audio: bool,
}

impl From<&VideoInfo> for VideoMeta {
    fn from(i: &VideoInfo) -> Self {
        Self {
            width: i.width,
            height: i.height,
            fps: i.fps,
            duration: i.duration,
            has_audio: i.has_audio,
        }
    }
}

/// Answer to `video_open`: a handle for later frame requests, and the facts.
#[derive(Debug, Clone, Serialize)]
pub struct VideoOpened {
    pub id: u32,
    pub meta: VideoMeta,
}

/// The open readers, by handle.
#[derive(Default)]
pub struct Videos {
    next: Mutex<u32>,
    open: Mutex<HashMap<u32, VideoReader>>,
}

/// Whether this build can show video pictures, and the reason when it can't.
#[tauri::command]
pub fn video_support() -> Result<(), String> {
    if altered_os_video::AVAILABLE {
        Ok(())
    } else {
        Err(altered_os_video::UNAVAILABLE.to_string())
    }
}

/// Shows the Open Video dialog. Resolves to None on cancel.
#[tauri::command]
pub async fn video_pick(app: AppHandle) -> Result<Option<PathBuf>, String> {
    let parent: Option<WebviewWindow> = app.get_webview_window("main");
    spawn_blocking(move || {
        let mut dialog = app
            .dialog()
            .file()
            .set_title("Open Video Reference")
            .add_filter("Video files", &["mp4", "m4v", "mov", "wmv", "avi", "mkv"])
            .add_filter("All files", &["*"]);
        if let Some(w) = parent.as_ref() {
            dialog = dialog.set_parent(w);
        }
        dialog
            .blocking_pick_file()
            .and_then(|f| f.as_path().map(Path::to_path_buf))
    })
    .await
    .map_err(|e| e.to_string())
}

/// Looks at a file without keeping it open: its size, rate, length, and
/// whether it carries sound. Err when it has no picture or can't be read.
#[tauri::command]
pub async fn video_probe(path: String) -> Result<VideoMeta, String> {
    spawn_blocking(move || altered_os_video::probe(Path::new(&path)).map(|i| VideoMeta::from(&i)))
        .await
        .map_err(|e| e.to_string())?
}

/// Opens a video for frame requests.
#[tauri::command]
pub async fn video_open(path: String, app: AppHandle) -> Result<VideoOpened, String> {
    spawn_blocking(move || -> Result<VideoOpened, String> {
        let reader = VideoReader::open(Path::new(&path))?;
        let meta = VideoMeta::from(reader.info());
        let state = app.state::<Videos>();
        let id = {
            let mut n = state.next.lock().map_err(|_| "Video: busy.".to_string())?;
            *n += 1;
            *n
        };
        state
            .open
            .lock()
            .map_err(|_| "Video: busy.".to_string())?
            .insert(id, reader);
        Ok(VideoOpened { id, meta })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// The picture shown at `seconds`, no wider than `max_width`. The bytes are
/// 16 header bytes (width u32, height u32, time f64, little-endian) and then
/// width x height RGBA.
#[tauri::command]
pub async fn video_frame(
    id: u32,
    seconds: f64,
    max_width: u32,
    app: AppHandle,
) -> Result<Response, String> {
    spawn_blocking(move || -> Result<Response, String> {
        let state = app.state::<Videos>();
        let map = state.open.lock().map_err(|_| "Video: busy.".to_string())?;
        let reader = map.get(&id).ok_or("Video: that file is not open.")?;
        let f = reader.frame_at(seconds, max_width.clamp(16, 4096))?;
        drop(map);
        let mut out = Vec::with_capacity(16 + f.rgba.len());
        out.extend_from_slice(&f.width.to_le_bytes());
        out.extend_from_slice(&f.height.to_le_bytes());
        out.extend_from_slice(&f.time.to_le_bytes());
        out.extend_from_slice(&f.rgba);
        Ok(Response::new(out))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Lets go of a video.
#[tauri::command]
pub fn video_close(id: u32, app: AppHandle) {
    if let Ok(mut map) = app.state::<Videos>().open.lock() {
        map.remove(&id);
    }
}
