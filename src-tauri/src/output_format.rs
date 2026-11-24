// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Output format rules for export and Batch Convert: which settings a format
//! allows, reading saved settings back, the text that says what a file will be,
//! the size estimate and the per-format memory. The Rust copy of the pure parts
//! of `src/app/output-format.ts`; both are tested against
//! `shared-fixtures/output-format.json`. Menu labels, help text and the
//! sample-type presets stay in TypeScript.

use crate::js_compat::{js_number_to_string, js_round};
use serde_json::Value;

pub const AAC_BITRATES: [u32; 4] = [96, 128, 160, 192];
pub const OUTPUT_RATES: [u32; 9] = [
    8_000, 11_025, 16_000, 22_050, 32_000, 44_100, 48_000, 88_200, 96_000,
];
pub const MP3_BITRATES: [u32; 12] = [32, 48, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
pub const MP3_MAX_RATE: f64 = 48_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Wav,
    Flac,
    Mp3,
    M4a,
    Aac,
}

pub const KINDS: [Kind; 5] = [Kind::Wav, Kind::Flac, Kind::Mp3, Kind::M4a, Kind::Aac];

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Mp3 => "mp3",
            Self::M4a => "m4a",
            Self::Aac => "aac",
        }
    }
    fn index(self) -> usize {
        KINDS.iter().position(|k| *k == self).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherKind {
    Rectangular,
    Triangular,
    Gaussian,
}

impl DitherKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rectangular => "rectangular",
            Self::Triangular => "triangular",
            Self::Gaussian => "gaussian",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shaping {
    None,
    Light,
    Strong,
}

impl Shaping {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Light => "light",
            Self::Strong => "strong",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resample {
    Draft,
    Good,
    Best,
    Highest,
}

impl Resample {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Good => "good",
            Self::Best => "best",
            Self::Highest => "highest",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputFormat {
    pub kind: Kind,
    pub bit_depth: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub mp3_vbr: bool,
    pub mp3_bitrate: u32,
    pub mp3_quality: u32,
    pub flac_level: u32,
    pub aac_bitrate: u32,
    pub dither: bool,
    pub dither_kind: DitherKind,
    pub dither_amount: f64,
    pub noise_shaping: Shaping,
    pub resample_quality: Resample,
    pub mix_left: f64,
    pub mix_right: f64,
}

pub const DEFAULT_FORMAT: OutputFormat = OutputFormat {
    kind: Kind::Wav,
    bit_depth: None,
    sample_rate: None,
    channels: None,
    mp3_vbr: false,
    mp3_bitrate: 192,
    mp3_quality: 2,
    flac_level: 5,
    aac_bitrate: 128,
    dither: true,
    dither_kind: DitherKind::Triangular,
    dither_amount: 1.0,
    noise_shaping: Shaping::None,
    resample_quality: Resample::Best,
    mix_left: 0.5,
    mix_right: 0.5,
};

/// What the source file is like (rate, channels, bit depth).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Src {
    pub sample_rate: f64,
    pub channels: f64,
    pub bit_depth: f64,
}

/// Whether this platform can write the format. Every format works everywhere: Windows
/// uses its own AAC encoder, the other systems a built-in one.
pub fn format_available(_kind: Kind, _windows: bool) -> bool {
    true
}

/// The bit depths a format offers, with their menu labels.
pub fn depths_for(kind: Kind) -> Vec<(u32, &'static str)> {
    if matches!(kind, Kind::Mp3 | Kind::M4a | Kind::Aac) {
        return Vec::new();
    }
    let d = [(16, "16-bit"), (24, "24-bit")];
    if kind == Kind::Wav {
        vec![
            (4, "4-bit ADPCM (smallest, lossy)"),
            (8, "8-bit"),
            d[0],
            d[1],
            (32, "32-bit float"),
        ]
    } else {
        d.to_vec()
    }
}

/// The format a file name's extension suggests. A name with no dot is read as
/// its own extension.
pub fn kind_of_path(path: &str) -> Option<Kind> {
    let start = path.rfind('.').map_or(0, |i| i + 1);
    match path[start..].to_lowercase().as_str() {
        "wav" | "wave" => Some(Kind::Wav),
        "flac" => Some(Kind::Flac),
        "mp3" => Some(Kind::Mp3),
        "m4a" => Some(Kind::M4a),
        "aac" => Some(Kind::Aac),
        _ => None,
    }
}

fn unit(v: Option<&Value>) -> f64 {
    match v.and_then(Value::as_f64) {
        Some(x) if x.is_finite() => 1.0_f64.min(0.0_f64.max(x)),
        _ => 0.5,
    }
}

fn listed(v: Option<&Value>, list: &[u32]) -> Option<u32> {
    let x = v.and_then(Value::as_f64)?;
    list.iter().copied().find(|c| f64::from(*c) == x)
}

/// Reads saved settings back: anything missing or not allowed falls back to the
/// default.
pub fn sanitize_format(raw: &Value) -> OutputFormat {
    let s = |k: &str| raw.get(k).and_then(Value::as_str);
    let kind = match s("kind") {
        Some("flac") => Kind::Flac,
        Some("mp3") => Kind::Mp3,
        Some("m4a") => Kind::M4a,
        Some("aac") => Kind::Aac,
        _ => Kind::Wav,
    };
    let depths: Vec<u32> = depths_for(kind).iter().map(|d| d.0).collect();
    let channels = raw.get("channels").and_then(Value::as_f64);
    OutputFormat {
        kind,
        bit_depth: listed(raw.get("bitDepth"), &depths),
        sample_rate: listed(raw.get("sampleRate"), &OUTPUT_RATES),
        channels: match channels {
            Some(1.0) => Some(1),
            Some(2.0) => Some(2),
            _ => None,
        },
        mp3_vbr: s("mp3Mode") == Some("vbr"),
        mp3_bitrate: listed(raw.get("mp3Bitrate"), &MP3_BITRATES).unwrap_or(192),
        mp3_quality: match raw.get("mp3Quality").and_then(Value::as_f64) {
            Some(q) => 9.0_f64.min(0.0_f64.max(js_round(q))) as u32,
            None => 2,
        },
        flac_level: match raw.get("flacLevel").and_then(Value::as_f64) {
            Some(q) if q.is_finite() => 8.0_f64.min(0.0_f64.max(js_round(q))) as u32,
            _ => 5,
        },
        aac_bitrate: listed(raw.get("aacBitrate"), &AAC_BITRATES).unwrap_or(128),
        dither: raw.get("dither") != Some(&Value::Bool(false)),
        dither_kind: match s("ditherKind") {
            Some("rectangular") => DitherKind::Rectangular,
            Some("gaussian") => DitherKind::Gaussian,
            _ => DitherKind::Triangular,
        },
        dither_amount: match raw.get("ditherAmount").and_then(Value::as_f64) {
            Some(a) if a.is_finite() => 2.0_f64.min(0.25_f64.max(a)),
            _ => 1.0,
        },
        noise_shaping: match s("noiseShaping") {
            Some("light") => Shaping::Light,
            Some("strong") => Shaping::Strong,
            _ => Shaping::None,
        },
        resample_quality: match s("resampleQuality") {
            Some("draft") => Resample::Draft,
            Some("good") => Resample::Good,
            Some("highest") => Resample::Highest,
            _ => Resample::Best,
        },
        mix_left: unit(raw.get("mixLeft")),
        mix_right: unit(raw.get("mixRight")),
    }
}

/// What the dither settings add to the sample type text, when anything.
pub fn dither_note(f: &OutputFormat) -> Option<String> {
    if !f.dither {
        return Some("no dither".to_string());
    }
    let mut bits = Vec::new();
    if f.dither_kind != DitherKind::Triangular {
        bits.push(format!("{} dither", f.dither_kind.as_str()));
    }
    if f.dither_amount != 1.0 {
        bits.push(format!("dither ×{}", js_number_to_string(f.dither_amount)));
    }
    if f.noise_shaping != Shaping::None {
        bits.push(format!("{} noise shaping", f.noise_shaping.as_str()));
    }
    (!bits.is_empty()).then(|| bits.join(", "))
}

/// The rate and channel count the file will really have.
fn effective(f: &OutputFormat, src: Src) -> (f64, f64) {
    let mut rate = f.sample_rate.map_or(src.sample_rate, f64::from);
    let mut ch = f.channels.map_or(src.channels, f64::from);
    if f.kind == Kind::Mp3 {
        ch = ch.min(2.0);
        if rate > MP3_MAX_RATE {
            rate = MP3_MAX_RATE;
        }
    }
    if f.kind == Kind::M4a || f.kind == Kind::Aac {
        ch = ch.min(2.0);
        rate = if rate > 44_100.0 { 48_000.0 } else { 44_100.0 };
    }
    (rate, ch)
}

/// The bit depth a lossless file gets: 4 and 8 only when chosen outright,
/// otherwise the source rounds up to 16, 24 or 32 the way the core does.
fn lossless_bits(f: &OutputFormat, src: Src) -> u32 {
    let want = f.bit_depth.map_or(src.bit_depth, f64::from);
    if f.kind == Kind::Wav {
        if let Some(b @ (4 | 8)) = f.bit_depth {
            return b;
        }
    }
    if f.kind == Kind::Flac {
        if want <= 16.0 {
            16
        } else {
            24
        }
    } else if want <= 16.0 {
        16
    } else if want <= 24.0 {
        24
    } else {
        32
    }
}

fn channel_word(ch: f64, many: &str) -> String {
    if ch == 1.0 {
        "mono".to_string()
    } else if ch == 2.0 {
        "stereo".to_string()
    } else {
        format!("{} {many}", js_number_to_string(ch))
    }
}

/// One line saying what the exported file will be.
pub fn describe_format(f: &OutputFormat, src: Src) -> String {
    let (rate, ch) = effective(f, src);
    let chs = channel_word(ch, "ch");
    let hz = format!("{} Hz", js_number_to_string(rate));
    match f.kind {
        Kind::M4a => return format!("AAC {} kbps, {hz} {chs}", f.aac_bitrate),
        Kind::Aac => return format!("AAC (raw) {} kbps, {hz} {chs}", f.aac_bitrate),
        Kind::Mp3 => {
            return if f.mp3_vbr {
                format!("MP3 variable, quality {}, {hz} {chs}", f.mp3_quality)
            } else {
                format!("MP3 {} kbps constant, {hz} {chs}", f.mp3_bitrate)
            };
        }
        _ => {}
    }
    let bits = lossless_bits(f, src);
    let depth = match bits {
        32 => "32-bit float".to_string(),
        4 => "4-bit IMA ADPCM".to_string(),
        b => format!("{b}-bit"),
    };
    format!(
        "{} {depth}, {hz} {chs}",
        if f.kind == Kind::Wav { "WAV" } else { "FLAC" }
    )
}

/// Rough size in bytes of `seconds` of audio in this format.
pub fn estimate_bytes(f: &OutputFormat, seconds: f64, src: Src) -> f64 {
    let rate = f
        .sample_rate
        .map_or(src.sample_rate, f64::from)
        .min(if f.kind == Kind::Mp3 {
            MP3_MAX_RATE
        } else {
            f64::INFINITY
        });
    let ch = f.channels.map_or(src.channels, f64::from);
    if f.kind == Kind::M4a || f.kind == Kind::Aac {
        return (f64::from(f.aac_bitrate) * 1000.0 * seconds) / 8.0;
    }
    if f.kind == Kind::Mp3 {
        // Variable quality 0..9 runs roughly 245 down to 65 kbps.
        let kbps = if f.mp3_vbr {
            245.0 - f64::from(f.mp3_quality) * 20.0
        } else {
            f64::from(f.mp3_bitrate)
        };
        return (kbps * 1000.0 * seconds) / 8.0;
    }
    let want = f.bit_depth.map_or(src.bit_depth, f64::from);
    if f.kind == Kind::Wav && f.bit_depth == Some(4) {
        return rate * ch * 0.5 * seconds;
    }
    if f.kind == Kind::Wav && f.bit_depth == Some(8) {
        return rate * ch * seconds;
    }
    let bytes = if f.kind == Kind::Flac {
        (if want <= 16.0 { 2.0 } else { 3.0 }) * 0.6
    } else if want <= 16.0 {
        2.0
    } else if want <= 24.0 {
        3.0
    } else {
        4.0
    };
    rate * ch * bytes * seconds
}

/// The "sample type" box: rate, channels and, for lossless formats, bit depth.
pub fn describe_sample_type(f: &OutputFormat, src: Src) -> String {
    let (rate, ch) = effective(f, src);
    let mut parts = vec![
        format!("{} Hz", js_number_to_string(rate)),
        channel_word(ch, "channels"),
    ];
    if !depths_for(f.kind).is_empty() {
        let bits = lossless_bits(f, src);
        parts.push(match bits {
            32 => "32-bit float".to_string(),
            4 => "4-bit ADPCM".to_string(),
            b => format!("{b}-bit"),
        });
        let note =
            if bits < 32 && bits != 4 && (f.bit_depth.is_some() || src.bit_depth > f64::from(bits))
            {
                dither_note(f)
            } else {
                None
            };
        if let Some(n) = note {
            parts.push(n);
        }
    }
    parts.join(", ")
}

/// The "format settings" box: what the format itself lets you set, or None.
pub fn describe_settings(f: &OutputFormat) -> Option<String> {
    match f.kind {
        Kind::Mp3 if f.mp3_vbr => Some(format!("MP3, variable, quality {}", f.mp3_quality)),
        Kind::Mp3 => Some(format!("MP3, {} kbps, constant", f.mp3_bitrate)),
        Kind::M4a => Some(format!("M4A, {} kbps", f.aac_bitrate)),
        Kind::Aac => Some(format!("Raw AAC, {} kbps", f.aac_bitrate)),
        _ => None,
    }
}

/// Settings remembered for each format, indexed like `KINDS`.
pub type Memory = [Option<OutputFormat>; 5];

/// Keeps only valid remembered formats.
pub fn sanitize_memory(raw: &Value) -> Memory {
    let mut out: Memory = Default::default();
    for kind in KINDS {
        if let Some(v) = raw.get(kind.as_str()) {
            if v.is_object() || v.is_array() {
                out[kind.index()] = Some(OutputFormat {
                    kind,
                    ..sanitize_format(v)
                });
            }
        }
    }
    out
}

/// Switching format: stash the one being left, bring back what the new one had
/// (or a clean start).
pub fn switch_format(memory: &Memory, from: &OutputFormat, to: Kind) -> (Memory, OutputFormat) {
    let mut next = memory.clone();
    next[from.kind.index()] = Some(from.clone());
    let format = next[to.index()].clone().unwrap_or(OutputFormat {
        kind: to,
        ..DEFAULT_FORMAT
    });
    (next, format)
}
