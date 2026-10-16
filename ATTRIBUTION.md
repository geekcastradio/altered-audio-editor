# Attribution

Code ported or adapted from other projects is credited here. Each entry lists the project, the source file, its license, and what was used. Required license notices are kept in the adapted files.

No third-party code has been ported yet.

| Project | Source file | License | What was used | Our file |
| --- | --- | --- | --- | --- |

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

