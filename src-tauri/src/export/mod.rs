// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Writing audio out of a session: mixdowns now; metadata and chapters
//! later.

pub mod aac;
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
pub mod otio;
pub mod region;
pub mod tags;
pub mod timecode;
pub mod verify;
pub mod wav_multi;
