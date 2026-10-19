# Attribution

Code ported or adapted from other projects is credited here. Each entry lists the project, the source file, its license, and what was used. Required license notices are kept in the adapted files.

No third-party code has been ported yet.

| Project | Source file | License | What was used | Our file |
| --- | --- | --- | --- | --- |

## Published formulas

| Source | Use | Our file |
| --- | --- | --- |
| Robert Bristow-Johnson, "Cookbook formulae for audio EQ biquad filter coefficients" (published freely for use) | Coefficient formulas for the low-pass, high-pass, band-pass, notch, peak and shelf filters behind Parametric EQ, Hum Cut and De-Esser. Formulas only; the filter code is our own. | `src-tauri/src/dsp/biquad.rs` |
| Sophocles J. Orfanidis, "Lecture Notes on Elliptic Filter Design" (Rutgers University, freely published) | The textbook elliptic design method: Landen's transformation for the Jacobi elliptic functions, the degree equation, and the pole and zero formulas. Method only; the code is our own. | `src-tauri/src/dsp/elliptic.rs` |
| MP4 file format (ISO/IEC 14496-12 box layout, as publicly documented) | The box layout for writing M4A tags (`moov/udta/meta/ilst`). Our own writer. | `src-tauri/src/export/m4a_tags.rs` |

## Bundled assets

| Asset | Source | License | Use |
| --- | --- | --- | --- |
| Source Sans 3 (regular, semibold; Latin) | npm package `@fontsource/source-sans-3` | SIL Open Font License 1.1 | Interface font, bundled into the app build. The license travels with the package. |

## Libraries with notable licenses

Most dependencies are MIT or Apache-2.0 and are listed in `Cargo.lock` and `package-lock.json`. These are called out because of their license or what they do:

| Library | License | Use |
| --- | --- | --- |
| LAME 3.100 (via `mp3lame-sys` and `mp3lame-encoder`) | LGPL-2.0-or-later (LAME); MIT (bindings) | MP3 encoding. Built from source and linked into the app; LGPL is compatible with the app's GPL-3.0-or-later. |
| `flacenc` | Apache-2.0 | FLAC encoding. |
| `opus-decoder` | MIT or Apache-2.0 | Opus decoding. |
| `zip` | MIT | Reading zip downloads of recordings. |
| Symphonia | MPL-2.0 | Decoding WAV, AIFF, FLAC, MP3, AAC, ALAC, Ogg Vorbis and ADPCM. |
| `windows` (Windows only) | MIT or Apache-2.0 | Calls the AAC encoder built into Windows (Media Foundation) for M4A export. No AAC encoder is bundled; the operating system provides it. |

