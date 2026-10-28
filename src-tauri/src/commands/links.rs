// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Opens Help menu web pages in the user's browser, only when the user
//! picks one. The app makes no network requests of its own.
//!
//! Only https links on the allowed hosts are opened, so nothing in the
//! app can be used to open arbitrary sites or local files.

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

/// Hosts the app may open: the GCRN site, the store page for the
/// project lead's book, Altered Audio: Podcasting Demystified, the
/// support page, the project's GitHub page and its Discord invite.
const ALLOWED_HOSTS: &[&str] = &[
    "thegcrn.com",
    "www.thegcrn.com",
    "www.barnesandnoble.com",
    "paypal.me",
    "github.com",
    "discord.gg",
];

/// Checks a link against the allow list.
pub fn is_allowed(link: &str) -> bool {
    match url::Url::parse(link) {
        Ok(u) => u.scheme() == "https" && u.host_str().is_some_and(|h| ALLOWED_HOSTS.contains(&h)),
        Err(_) => false,
    }
}

#[tauri::command]
pub fn open_link(app: AppHandle, url: String) -> Result<(), String> {
    if !is_allowed(&url) {
        return Err(format!("Link not allowed: {url}"));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::is_allowed;

    #[test]
    fn allows_only_https_gcrn_links() {
        assert!(is_allowed("https://thegcrn.com/"));
        assert!(is_allowed("https://www.thegcrn.com/plugins/"));
        assert!(!is_allowed("http://thegcrn.com/"));
        assert!(!is_allowed("https://thegcrn.com.evil.example/"));
        assert!(!is_allowed("https://example.com/"));
        assert!(is_allowed(
            "https://www.barnesandnoble.com/w/altered-audio-steve-phillips/1147888459"
        ));
        assert!(!is_allowed("https://barnesandnoble.com.evil.example/"));
        assert!(is_allowed("https://paypal.me/scp21"));
        assert!(!is_allowed("http://paypal.me/scp21"));
        assert!(!is_allowed("https://paypal.me.evil.example/scp21"));
        assert!(is_allowed(
            "https://github.com/geekcastradio/altered-audio-editor/issues/new/choose"
        ));
        assert!(is_allowed("https://discord.gg/yAhcfmfevd"));
        assert!(!is_allowed("https://github.com.evil.example/"));
        assert!(!is_allowed("file:///C:/Windows/System32/calc.exe"));
        assert!(!is_allowed("not a url"));
    }
}
