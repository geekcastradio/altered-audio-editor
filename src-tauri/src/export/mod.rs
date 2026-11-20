// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Writing audio out of a session: mixdowns now; metadata and chapters
//! later.

pub mod aac;
#[cfg(feature = "pure-aac")]
pub mod aac_pure;
pub mod adpcm;
pub mod analysis;
pub mod chapters;
pub mod convert;
pub mod cover;
pub mod edl;
pub mod encode;
pub mod interchange;
pub mod loudness;
pub mod m4a_tags;
pub mod mixdown;
pub mod mp4mux;
pub mod otio;
pub mod region;
pub mod tags;
pub mod timecode;
pub mod verify;
pub mod wav_multi;
