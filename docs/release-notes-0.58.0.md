## Altered Audio Editor 0.58.0

### Changed
- Behind the scenes, more of the interface's rules now have a copy in the Rust core, checked against the interface's own with shared test files: the timeline view (zoom, scrolling, the ruler, time and timecode text, the overview strip, track heights, peak levels), the Check Loudness targets, verdicts and export prediction, and the recording rules (punch-in, pre-roll and where a finished take lands). Nothing looks different.
