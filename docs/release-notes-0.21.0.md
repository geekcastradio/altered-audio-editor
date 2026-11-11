## Altered Audio Editor 0.21.0

True-peak limiting.

### Limiter
- **New True peak switch on the Limiter.** Turn it on and the Limiter holds the peaks between samples under the ceiling, the ones a converter, a phone speaker or an MP3 or AAC encoder can reconstruct, not only the stored samples. It looks a little further ahead to do this (the Voice Chain accounts for the extra delay). With it on, the two channels always share one gain so the stereo image stays put, and the look-ahead stays between 0.5 and 5 ms. With it off the Limiter works exactly as before.

### Loudness Match
- **Limit the peaks a match would push over.** When a match would take a track over its peak limit, a new option adds a True Peak limiter at that limit to the track, instead of leaving the row unticked. A clip's whole track gets the limiter, and it can be changed or removed in the track's Voice Chain. The check after applying reads clips without their effects, so those rows can still show the warning.
