// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Every type survives a trip to JSON and back unchanged, in memory
//! and on disk. Header checks reject files this build cannot read.

mod common;

use std::fmt::Debug;

use altered_audio_editor_lib::library::{Library, Template};
use altered_audio_editor_lib::session::document::{self, from_json, load, save, to_json};
use altered_audio_editor_lib::session::validate::validate;
use altered_audio_editor_lib::session::{
    Document, DocumentError, EffectChain, ForeignEffect, Session, Workspace,
};
use pretty_assertions::assert_eq;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Round trips a plain value through serde_json.
fn roundtrip_value<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let text = serde_json::to_string_pretty(value).unwrap();
    let back: T = serde_json::from_str(&text).unwrap();
    assert_eq!(&back, value);
}

/// Round trips a document through the versioned loader, in memory and
/// on disk.
fn roundtrip_document<T>(doc: &T)
where
    T: Document + PartialEq + Debug,
{
    let text = to_json(doc).unwrap();
    assert!(text.ends_with('\n'));
    let back: T = from_json(&text).unwrap();
    assert_eq!(&back, doc);

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("doc.json");
    save(doc, &path).unwrap();
    // Saving twice must replace the file, including on Windows.
    save(doc, &path).unwrap();
    let loaded: T = load(&path).unwrap();
    assert_eq!(&loaded, doc);

    let leftovers: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(leftovers.len(), 1, "temp file left behind: {leftovers:?}");
}

#[test]
fn parts_roundtrip() {
    roundtrip_value(&common::track());
    roundtrip_value(&common::bus());
    roundtrip_value(&common::clip());
    roundtrip_value(&common::envelope());
    roundtrip_value(&common::marker());
    roundtrip_value(&common::effect_chain());
    roundtrip_value(&common::library_item());
}

#[test]
fn session_roundtrips() {
    roundtrip_document(&common::session());
}

#[test]
fn new_session_roundtrips() {
    roundtrip_document(&Session::new("Empty", 44_100));
}

#[test]
fn library_roundtrips() {
    roundtrip_document(&common::library());
}

#[test]
fn template_roundtrips() {
    roundtrip_document(&common::template());
}

#[test]
fn workspace_roundtrips() {
    roundtrip_document(&common::workspace());
}

#[test]
fn host_profile_roundtrips() {
    roundtrip_document(&common::host_profile());
}

#[test]
fn keymap_roundtrips() {
    roundtrip_document(&common::keymap());
}

#[test]
fn preferences_roundtrip() {
    roundtrip_document(&common::preferences());
}

#[test]
fn sample_session_is_valid() {
    assert_eq!(validate(&common::session()), vec![]);
}

#[test]
fn every_document_writes_its_header() {
    fn header<T: Document>(doc: &T) -> (String, u64) {
        let v: serde_json::Value = serde_json::from_str(&to_json(doc).unwrap()).unwrap();
        (
            v["format"].as_str().unwrap().to_string(),
            v["schema_version"].as_u64().unwrap(),
        )
    }
    assert_eq!(header(&common::session()), ("altaudio.session".into(), 1));
    assert_eq!(header(&common::library()), ("altaudio.library".into(), 1));
    assert_eq!(header(&common::template()), ("altaudio.template".into(), 1));
    assert_eq!(
        header(&common::workspace()),
        ("altaudio.workspace".into(), 1)
    );
    assert_eq!(
        header(&common::host_profile()),
        ("altaudio.host_profile".into(), 1)
    );
}

/// Fields a newer version wrote must survive a load and save in this one.
#[test]
fn unknown_session_fields_survive_a_load_and_save() {
    let mut v = serde_json::to_value(common::session()).unwrap();
    v["from_the_future"] = serde_json::json!({ "level": 3, "tags": ["a", "b"] });
    v["tracks"][0]["track_note"] = serde_json::json!("keep me");
    v["tracks"][0]["clips"][0]["clip_flag"] = serde_json::json!(true);
    let text = serde_json::to_string(&v).unwrap();

    let session: Session = from_json(&text).unwrap();
    assert_eq!(session.extra["from_the_future"]["level"], 3);

    let back: serde_json::Value = serde_json::from_str(&to_json(&session).unwrap()).unwrap();
    assert_eq!(back["from_the_future"], v["from_the_future"]);
    assert_eq!(back["tracks"][0]["track_note"], "keep me");
    assert_eq!(back["tracks"][0]["clips"][0]["clip_flag"], true);
    // And a second pass changes nothing.
    let again: Session = from_json(&to_json(&session).unwrap()).unwrap();
    assert_eq!(again, session);
}

/// A session with nothing extra writes no extra keys at all.
#[test]
fn a_plain_session_writes_no_extra_keys() {
    let v = serde_json::to_value(common::session()).unwrap();
    assert!(v.get("extra").is_none());
    assert!(v["tracks"][0].get("extra").is_none());
}

fn session_value() -> serde_json::Value {
    serde_json::to_value(common::session()).unwrap()
}

#[test]
fn rejects_wrong_format() {
    let err = document::from_value::<Library>(session_value()).unwrap_err();
    assert!(matches!(err, DocumentError::WrongFormat { .. }), "{err}");
}

#[test]
fn rejects_newer_version() {
    let mut v = session_value();
    v["schema_version"] = 99.into();
    let err = document::from_value::<Session>(v).unwrap_err();
    assert!(
        matches!(err, DocumentError::TooNew { found: 99, .. }),
        "{err}"
    );
    assert!(err.to_string().contains("Update Altered Audio Editor"));
}

#[test]
fn rejects_version_zero() {
    let mut v = session_value();
    v["schema_version"] = 0.into();
    let err = document::from_value::<Session>(v).unwrap_err();
    assert!(matches!(err, DocumentError::UnknownVersion { .. }), "{err}");
}

#[test]
fn rejects_missing_header() {
    let mut v = session_value();
    v.as_object_mut().unwrap().remove("schema_version");
    let err = document::from_value::<Session>(v).unwrap_err();
    assert!(matches!(
        err,
        DocumentError::MissingHeader("schema_version")
    ));
}

#[test]
fn rejects_invalid_json() {
    let err = from_json::<Session>("{ not json").unwrap_err();
    assert!(matches!(err, DocumentError::Json(_)));
}

#[test]
fn template_checks_its_nested_session() {
    let mut v = serde_json::to_value(common::template()).unwrap();
    v["session"]["schema_version"] = 99.into();
    let err = document::from_value::<Template>(v).unwrap_err();
    assert!(matches!(err, DocumentError::TooNew { .. }), "{err}");
}

#[test]
fn missing_file_reports_path() {
    let err = load::<Workspace>(std::path::Path::new("does/not/exist.alworkspace")).unwrap_err();
    assert!(err.to_string().contains("exist.alworkspace"), "{err}");
}

#[test]
fn kept_foreign_effects_round_trip_and_old_chains_still_open() {
    // A chain saved before `foreign` existed has no such key.
    let old = r#"{"enabled":true,"input_gain_db":0.0,"output_gain_db":0.0,"mix":1.0,"slots":[]}"#;
    let chain: EffectChain = serde_json::from_str(old).unwrap();
    assert!(chain.foreign.is_empty());
    // With none kept, nothing new is written, so old readers see the same file.
    assert!(!serde_json::to_string(&chain).unwrap().contains("foreign"));
    let mut kept = chain;
    kept.foreign.push(ForeignEffect {
        name: "DeNoise".into(),
        enabled: false,
        position: 3,
        xml: r#"<component componentID="x" name="DeNoise"/>"#.into(),
    });
    let back: EffectChain = serde_json::from_str(&serde_json::to_string(&kept).unwrap()).unwrap();
    assert_eq!(back, kept);
}
