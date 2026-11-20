## Altered Audio Editor 0.42.0

### Effects
- Fill Single Click has a texture option in Settings > Effects: Natural (the default, as before) or Smooth. Smooth fills the gap with a clean interpolation and adds no noise. The undo entry says "Fill Single Click (Smooth)" when it is used.

### Under the hood
- The VST3 hosting code now documents why every unsafe block is sound, checks pointers handed to it by plugins, and fails the build if a new unsafe block comes without a safety comment.
- New, optional `pure-aac` build feature: AAC export through a pure-Rust encoder (`filmcraft-aac`, MIT or Apache-2.0) instead of the operating system's encoder. It is off in release builds. CI now builds and tests it on Linux.

### Licensing and docs
- `docs/codecs.md` lists every codec the app reads and writes, where each one comes from, and its license.
- ATTRIBUTION.md corrected: the LAME bindings are LGPL-3.0 (not MIT), and a few more third-party licenses are listed.
