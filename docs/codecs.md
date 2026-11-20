# Codec inventory

Every audio and video codec and container the program reads or writes, who supplies it, and under what license. Written for contributors, packagers and anyone who has to answer "what is in this build, and who owns it?"

Last checked against 0.41.0 plus the 0.42.0 work. If you add, remove or switch a codec, change this page in the same pull request. Third-party licenses are also listed in `ATTRIBUTION.md`.

## At a glance

- No codec library is downloaded at run time, and none is installed alongside the program.
- Everything below is either **compiled into the program** (a Rust crate or LAME built from source), or **provided by the operating system** (Windows Media Foundation, macOS AVFoundation). The program never ships FFmpeg, FDK-AAC, libx264 or a similar library.
- Patent-encumbered formats (AAC, H.264, HEVC) are handled by the operating system's own decoders and encoder by default. The one exception is the optional `pure-aac` build, described below.

## Audio in (decoding)

| Format | Supplied by | License | Notes |
| --- | --- | --- | --- |
| WAV, BWF, RF64, W64 (PCM, float, IMA ADPCM) | Symphonia, plus our own long-file reader (`src-tauri/src/codecs/longwav.rs`, `wavlong.rs`) | MPL-2.0 (Symphonia); our code GPL-3.0-or-later | The long-file reader handles files over 4 GB. |
| AIFF, AIFC | Symphonia | MPL-2.0 | |
| FLAC | Symphonia | MPL-2.0 | |
| MP3 | Symphonia | MPL-2.0 | |
| AAC, M4A, MP4 audio | Symphonia | MPL-2.0 | Decoding is pure Rust, on every system. |
| ALAC | Symphonia | MPL-2.0 | |
| Ogg Vorbis | Symphonia | MPL-2.0 | |
| Opus (`.opus`, Ogg Opus) | `opus-decoder` crate, patched copy in `src-tauri/vendor/opus-decoder` | MIT or Apache-2.0 | Pure Rust. The Ogg container is read by Symphonia. |
| Matroska audio (`.mkv`) | Symphonia | MPL-2.0 | |
| Tags and cover art (ID3v2, MP4 `ilst`) | Symphonia; our own M4A tag writer (`export/m4a_tags.rs`) | MPL-2.0; GPL-3.0-or-later | |

## Audio out (encoding)

| Format | Supplied by | License | Notes |
| --- | --- | --- | --- |
| WAV (8, 16, 24, 32-bit, float), RF64 over 4 GB | Our own writer (`codecs/wavlong.rs`) | GPL-3.0-or-later | |
| WAV, 4-bit IMA ADPCM | Our own writer (`export/adpcm.rs`), from the public IMA recommendation | GPL-3.0-or-later | |
| FLAC | `flacenc` | Apache-2.0 | Pure Rust. |
| MP3 | LAME 3.100 through `mp3lame-sys` and `mp3lame-encoder`, built from source and linked in | LAME: LGPL-2.0-or-later. The two Rust crates declare LGPL-3.0. | Linking LGPL code into a GPL-3.0-or-later program is allowed; the source for the whole program is public, which satisfies the relinking terms. |
| AAC in M4A, raw AAC (ADTS `.aac`) | **Default build:** the Windows system encoder (Media Foundation), through `src-tauri/os-aac`. Windows only. | Nothing bundled | On other systems the Export dialog says AAC is unavailable and offers MP3 or FLAC. |
| AAC in M4A, raw AAC (ADTS `.aac`) | **`pure-aac` build:** `filmcraft-aac` (a clean-room encoder from ISO/IEC 14496-3, no `unsafe`), through `src-tauri/src/export/aac_pure.rs`. The M4A container is written by our own code. | `filmcraft-aac`: MIT OR Apache-2.0 | Off by default. Works on every system. See below. |
| Chapter and marker data (WAV cue and labl, FLAC cue sheet and tags, ID3 CHAP, MP4 `chpl`) | Our own writers | GPL-3.0-or-later | |

### The `pure-aac` feature

Build with `cargo build --features pure-aac` (or `tauri build -- --features pure-aac`).

- Same Export dialog and same limits (44.1 or 48 kHz at 96 to 192 kbps) as the system encoder, so the dialog does not change.
- Writes M4A with an edit list that trims the encoder's start delay, so a file plays the length that went in.
- Mono and stereo only for M4A today.
- Verified against the program's own post-export check (opens, every sample decodes, length matches) and decoded by an independent decoder (FFmpeg) in the tests.
- **Patents.** AAC is covered by patents held by a licensing pool. A software encoder shipped inside the program is a different position from the operating system's encoder, whose maker already carries the license. Get a short legal opinion before turning this on in a public release. Until then it stays an opt-in build flag, and the installers on the Releases page are built without it.

## Video (pictures only)

| What | Supplied by | License | Notes |
| --- | --- | --- | --- |
| Video preview (H.264, usually HEVC, others as the system allows) | Windows Media Foundation, through `src-tauri/os-video`; macOS AVFoundation behind the `apple-video` feature (not yet built on a Mac) | Nothing bundled | One frame at a time. Linux has no system decoder, so there is no video preview there. |
| Putting a mix back into a video (picture copied, not re-encoded) | `mp4` crate | MIT | H.264 in `.mp4`, `.m4v`, `.mov` only. The sound is the exported M4A. |

## Not used

- No FFmpeg, libav, GStreamer or libmpv.
- No FDK-AAC or any crate derived from it (license incompatible with GPL-3.0-or-later).
- No VST2 (the SDK is no longer licensed). VST3 plug-ins are hosted in a separate process; see `src-tauri/plughost`.
- Speech models (RNNoise through `nnnoiseless`, DeepFilterNet3 through `deep-filter-rt`) are not codecs. They are listed in `ATTRIBUTION.md` with their licenses.

## Open items

- Legal opinion on AAC, H.264 and HEVC before any bundled software codec is on by default.
- Whether to replace the operating system's video decoders with a pure-Rust set. A plan is written (opt-in, behind a feature, with fallback to the system decoder); nothing is built.
- `symphonia`'s MPL-2.0 license applies to its own files only. We use it unmodified, apart from the debug-profile override in `Cargo.toml`, which changes build settings and not code.
