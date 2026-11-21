// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Help > Report a Problem opens the project's bug form in the browser with the
//! app version and operating system already filled in. That is all it puts in
//! the link: no errors, no file names, no paths. This is the Rust copy of
//! `src/app/bug-report-link.ts`; both are tested against
//! `shared-fixtures/bug-report-link.json`.

const NEW_ISSUE: &str = "https://github.com/geekcastradio/altered-audio-editor/issues/new";

/// The operating systems the bug form names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportOs {
    Windows,
    MacOs,
    Linux,
}

impl ReportOs {
    pub fn label(self) -> &'static str {
        match self {
            ReportOs::Windows => "Windows",
            ReportOs::MacOs => "macOS",
            ReportOs::Linux => "Linux",
        }
    }
}

/// The operating system named in the browser engine's user agent, or `None`
/// when it is not one of the three. Matching ignores case.
pub fn os_from(user_agent: &str) -> Option<ReportOs> {
    let ua = user_agent.to_ascii_lowercase();
    if ua.contains("windows") {
        Some(ReportOs::Windows)
    } else if ua.contains("macintosh") || ua.contains("mac os x") {
        Some(ReportOs::MacOs)
    } else if ua.contains("linux") || ua.contains("x11") {
        Some(ReportOs::Linux)
    } else {
        None
    }
}

/// A version is filled in only when it looks like one: 1 to 40 characters from
/// letters, digits, `.`, `+` and `-`. A path or a sentence is left out.
fn looks_like_version(v: &str) -> bool {
    (1..=40).contains(&v.len())
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'+' | b'-'))
}

/// Form encoding, as a browser writes a query value: letters, digits and
/// `*-._` stay, a space becomes `+`, every other byte becomes `%XX`.
fn form_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'*' | b'-' | b'.' | b'_' => {
                out.push(char::from(b))
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// The bug form's link, with the app version and operating system filled in
/// where known.
pub fn report_url(app_version: Option<&str>, user_agent: &str) -> String {
    let mut q = vec![("template", "bug_report.yml".to_string())];
    if let Some(v) = app_version.filter(|v| looks_like_version(v)) {
        q.push(("app_version", v.to_string()));
    }
    if let Some(os) = os_from(user_agent) {
        q.push(("os", os.label().to_string()));
    }
    let query: Vec<String> = q
        .iter()
        .map(|(k, v)| format!("{}={}", form_encode(k), form_encode(v)))
        .collect();
    format!("{NEW_ISSUE}?{}", query.join("&"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_is_never_put_in_the_link() {
        let url = report_url(Some("C:\\Users\\steve"), "Macintosh");
        assert!(!url.contains("steve") && !url.contains("app_version"));
    }

    #[test]
    fn a_plus_in_a_version_is_encoded() {
        assert!(report_url(Some("1.2.3+build.5"), "").contains("app_version=1.2.3%2Bbuild.5"));
    }

    #[test]
    fn nothing_known_leaves_only_the_template() {
        assert_eq!(
            report_url(None, "something else"),
            format!("{NEW_ISSUE}?template=bug_report.yml")
        );
    }
}
