## Altered Audio Editor 0.59.0

### Changed
- Behind the scenes, more of the interface's rules now have a copy in the Rust core, checked against the interface's own with shared test files: the output format rules (which bit depths a format offers, the format a file name suggests, the lines that describe a file, size estimates), the pitch edit choices and report sentence, what the welcome dialog shows on a fresh install or after an update, and the bug report text with its user-name scrub. Nothing looks different.
