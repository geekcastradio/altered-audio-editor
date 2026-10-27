# Altered Audio Editor 0.5.2

Fixes for recording with input monitoring, and a progress bar when a Voice Chain is applied to a file.

## Fixed
- **Input monitoring with ASIO.** Monitoring now plays on the input's own ASIO driver, so you can hear yourself while recording. Before, it tried the output picked in Settings, which could belong to a different driver.
- **Effects while monitoring.** A monitored track is heard through the built-in effects in its Voice Chain (EQ, compressor, limiter and the rest). VST3 plug-ins are not included, because their delay would be heard. The recording is always the clean input. Effect changes made while recording are heard the next time monitoring starts.

## New
- **Hear the input in the Waveform editor** (Settings > Playback and Recording). Off by default; use headphones. The file's Voice Chain is heard too.
- **Progress when applying a Voice Chain to a file.** It shows how far along it is, about how long is left, and a Cancel button that leaves the file as it was.

## Test checklist
- Settings: choose ASIO, turn on Monitor Input for an armed track, start recording: you hear yourself
- Same with the System driver: still works
- Add a Parametric EQ with an obvious boost to the track, turn on Monitor Input, and hear it while recording; open the recording afterwards and confirm the file itself is unprocessed
- Add a plug-in to the chain: it is not heard while monitoring, and nothing else changes
- Waveform editor: turn on the new setting, record, and hear the input; turn it off and it is silent
- Apply a slow chain to a file: the progress bar moves and shows the time left; Cancel leaves the file unchanged and Undo is not needed
