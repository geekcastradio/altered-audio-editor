## Altered Audio Editor 0.57.0

### Changed
- Behind the scenes, more of the interface's rules now have a copy in the Rust core, checked against the interface's own with shared test files: the envelope line maths (levels, snapping, value text, adding, moving and slicing points), the dip that Silence Time Selection puts in a clip's volume line, the stretch percent text and the time-entry reader, the session tab rules, and the track visibility presets. Nothing looks different.
