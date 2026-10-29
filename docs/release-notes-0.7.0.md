# Altered Audio Editor 0.7.0

Mixing helpers for podcasts: music that ducks under voices, an Audio FX Panel, a multiband compressor, and a richer Publish Package.

## Added
- **Duck Music Under Voice (Multitrack menu).** Pick light, medium or deep and the music track turns down while any voice track speaks, then comes back up. It is a normal sidechain Dynamics effect on the music track, so you can open and tune it like any other. Running it again replaces the old one.
- **Audio FX Panel (Window menu), version 1.** Four looks, each Off, Light, Medium or Strong: Restoration (noise), Level (leveler and compressor), Clarity (voice EQ and de-esser) and Polish (limiter). Preview it on the selection or the whole file, apply it, or remove it. It writes ordinary Voice Chain effects, so nothing is hidden. It tells you when the selected clip is not marked as dialogue. The strengths are starting points, not tuned on every voice.
- **Multiband Compressor.** Three bands with their own threshold, ratio and gain, so you can calm boomy lows or harsh highs without squashing the whole voice.
- **Publish Package additions.** A `chapters.vtt` file (WebVTT) when you have chapter markers, and a `midrolls.json` file with ad positions when you have ad slots.
- **Settings: Amp Model without Music Mode.** One checkbox offers the Amp Model in the Add effect menus outside the Music workspace.

## Fixed
- **Record with nothing open** now asks for a new audio file first and records into it, as other editors do. On the "Nothing open" screen every transport and zoom button is greyed out except Record.

## Known limits
- M4A export and its check run on Windows only. The automated test skips real AAC encoding on Windows (it crashed the test process once); please export an M4A by hand and report any trouble.
- With two voices talking at once, the music ducks by the sum of both. If that is too much, use a lighter setting.
- Untested on real hardware: buses and sends on real interfaces, VST3 instruments and effects, MIDI input, and Linux and macOS builds. Treat those as experimental and tell us what you find.
