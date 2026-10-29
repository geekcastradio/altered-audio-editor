// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Cover art inside an MP3's ID3v2.3 tag (the APIC frame): read it,
//! replace it, remove it. Every other frame, chapters included, is kept.
//! Other formats are left alone; this editor only writes MP3 cover art.

use std::fs;
use std::path::Path;

use super::chapters::{empty_id3, existing_id3, frame, id3_frames, replace, with_frames};

/// Pictures larger than this are refused; a tag is not the place for them.
pub const COVER_MAX: usize = 8 * 1024 * 1024;

/// The image type of `bytes` by its first bytes: JPEG or PNG only.
pub fn image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg")
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        Some("image/png")
    } else {
        None
    }
}

/// An APIC frame body: Latin-1 text encoding, the MIME type, "front cover", no description.
fn apic_body(mime: &str, image: &[u8]) -> Vec<u8> {
    let mut body = vec![0u8];
    body.extend_from_slice(mime.as_bytes());
    body.push(0);
    body.push(3);
    body.push(0);
    body.extend_from_slice(image);
    body
}

/// The picture inside an APIC frame body: its MIME type and bytes.
fn parse_apic(body: &[u8]) -> Option<(String, Vec<u8>)> {
    let enc = *body.first()?;
    let rest = body.get(1..)?;
    let mime_end = rest.iter().position(|&b| b == 0)?;
    let mime = String::from_utf8_lossy(&rest[..mime_end]).into_owned();
    // After the MIME type: a picture type byte, then a description ended by one
    // zero byte (Latin-1, UTF-8) or two zero bytes at an even offset (UTF-16).
    let after = rest.get(mime_end + 2..)?;
    let wide = enc == 1 || enc == 2;
    let skip = if wide {
        let mut i = 0;
        loop {
            let pair = after.get(i..i + 2)?;
            if pair == [0, 0] {
                break i + 2;
            }
            i += 2;
        }
    } else {
        after.iter().position(|&b| b == 0)? + 1
    };
    Some((mime, after.get(skip..)?.to_vec()))
}

/// The cover art in an MP3 file, if it has any: its MIME type and bytes.
pub fn read_cover(path: &Path) -> Option<(String, Vec<u8>)> {
    if !is_mp3(path) {
        return None;
    }
    let data = fs::read(path).ok()?;
    let n = existing_id3(&data);
    id3_frames(&data[..n])
        .into_iter()
        .filter(|(id, _)| id == b"APIC")
        .find_map(|(_, body)| parse_apic(&body))
        .filter(|(_, img)| !img.is_empty())
}

fn is_mp3(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("mp3"))
}

/// Replaces the cover art of an MP3 with `image` (JPEG or PNG), or removes it when `image`
/// is None. False when the file is not an MP3.
pub fn write_cover(path: &Path, image: Option<&[u8]>) -> std::io::Result<bool> {
    if !is_mp3(path) {
        return Ok(false);
    }
    let mut new_frame = Vec::new();
    if let Some(img) = image {
        let Some(mime) = image_mime(img) else {
            return Err(std::io::Error::other("The picture must be a JPEG or PNG."));
        };
        if img.len() > COVER_MAX {
            return Err(std::io::Error::other("The picture is larger than 8 MB."));
        }
        new_frame = frame(b"APIC", &apic_body(mime, img));
    }
    let data = fs::read(path)?;
    let n = existing_id3(&data);
    let mut frames = new_frame;
    for (id, body) in id3_frames(&data[..n]) {
        if &id != b"APIC" {
            frames.extend(frame(&id, &body));
        }
    }
    let mut out = with_frames(&empty_id3(), &frames);
    out.extend_from_slice(&data[n..]);
    replace(path, &out)?;
    Ok(true)
}

/// Standard base64, for handing a picture to the page as a data URL.
pub fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for c in bytes.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if c.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::tags::{write_tags, Tags};

    /// Whether the file's tag holds a frame with this id (the stand-in audio is too short for the decoder to read tags).
    fn has(p: &Path, id: &[u8; 4]) -> bool {
        let data = fs::read(p).unwrap();
        id3_frames(&data[..existing_id3(&data)])
            .iter()
            .any(|(i, _)| i == id)
    }

    fn jpeg() -> Vec<u8> {
        let mut v = vec![0xff, 0xd8, 0xff, 0xe0];
        v.extend((0..200u16).map(|i| (i % 251) as u8));
        v
    }

    fn mp3_file(dir: &Path) -> std::path::PathBuf {
        let p = dir.join("a.mp3");
        // A few bytes after the tag stand in for audio; the tag code never decodes them.
        fs::write(&p, [0xffu8, 0xfb, 0x90, 0x00, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        p
    }

    #[test]
    fn base64_matches_known_values() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn only_jpeg_and_png_are_pictures() {
        assert_eq!(image_mime(&jpeg()), Some("image/jpeg"));
        assert_eq!(
            image_mime(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0]),
            Some("image/png")
        );
        assert_eq!(image_mime(b"GIF89a"), None);
    }

    #[test]
    fn a_cover_goes_in_reads_back_replaces_and_comes_out_keeping_tags_and_audio() {
        let dir = tempfile::tempdir().unwrap();
        let p = mp3_file(dir.path());
        write_tags(
            &p,
            &Tags {
                title: Some("Episode 1".into()),
                ..Tags::default()
            },
        )
        .unwrap();
        assert!(read_cover(&p).is_none());

        let img = jpeg();
        assert!(write_cover(&p, Some(&img)).unwrap());
        let (mime, back) = read_cover(&p).unwrap();
        assert_eq!(mime, "image/jpeg");
        assert_eq!(back, img);
        assert!(has(&p, b"TIT2"));

        // Writing tags afterwards keeps the picture.
        write_tags(
            &p,
            &Tags {
                artist: Some("GCRN".into()),
                ..Tags::default()
            },
        )
        .unwrap();
        assert_eq!(read_cover(&p).unwrap().1, img);

        // A second picture replaces the first; there is never more than one.
        let mut other = img.clone();
        other.push(9);
        write_cover(&p, Some(&other)).unwrap();
        let data = fs::read(&p).unwrap();
        let n = existing_id3(&data);
        assert_eq!(
            id3_frames(&data[..n])
                .iter()
                .filter(|(id, _)| id == b"APIC")
                .count(),
            1
        );
        assert_eq!(read_cover(&p).unwrap().1, other);

        // Removing it leaves the text tags and the audio bytes.
        write_cover(&p, None).unwrap();
        assert!(read_cover(&p).is_none());
        assert!(has(&p, b"TIT2") && has(&p, b"TPE1"));
        let data = fs::read(&p).unwrap();
        assert!(data.ends_with(&[0xff, 0xfb, 0x90, 0x00, 1, 2, 3, 4, 5, 6, 7, 8]));
    }

    #[test]
    fn bad_pictures_and_other_formats_are_refused_without_touching_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = mp3_file(dir.path());
        let before = fs::read(&p).unwrap();
        assert!(write_cover(&p, Some(b"not a picture")).is_err());
        assert_eq!(fs::read(&p).unwrap(), before);
        let wav = dir.path().join("a.wav");
        fs::write(&wav, b"RIFF").unwrap();
        assert!(!write_cover(&wav, Some(&jpeg())).unwrap());
        assert!(read_cover(&wav).is_none());
    }

    #[test]
    fn a_description_in_utf16_is_skipped() {
        let mut body = vec![1u8];
        body.extend_from_slice(b"image/png\0");
        body.push(3);
        body.extend_from_slice(&[0xff, 0xfe, b'h', 0, b'i', 0, 0, 0]);
        body.extend_from_slice(&[1, 2, 3]);
        assert_eq!(
            parse_apic(&body),
            Some(("image/png".to_string(), vec![1, 2, 3]))
        );
    }
}
