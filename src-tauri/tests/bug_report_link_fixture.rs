// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The Rust bug report link must agree with `shared-fixtures/bug-report-link.json`,
//! the same file the TypeScript tests read. Strings are compared exactly.

use altered_audio_editor_lib::bug_report_link::*;
use serde_json::Value;

fn fixture() -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../shared-fixtures/bug-report-link.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn os_from_matches() {
    for c in fixture()["osFrom"].as_array().unwrap() {
        let got = os_from(c["ua"].as_str().unwrap()).map(ReportOs::label);
        assert_eq!(got, c["want"].as_str(), "{c}");
    }
}

#[test]
fn report_url_matches() {
    for c in fixture()["reportUrl"].as_array().unwrap() {
        let got = report_url(c["version"].as_str(), c["ua"].as_str().unwrap());
        assert_eq!(got, c["want"].as_str().unwrap(), "{c}");
    }
}
