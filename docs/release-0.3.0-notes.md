# Altered Audio Editor 0.3.0

A big one, with a lot of it for people who make music. None of it shows up unless you ask: your podcast and voiceover layouts look exactly as they did in 0.2.0.

## New: Music Mode

Turn it on in View > Music Mode, or open the Music workspace from Window > Workspace. It adds:

- MIDI tracks with a piano roll and a VST3 instrument per track
- The Amp Model effect (loads Neural Amp Modeler captures)
- A metronome, a tempo strip and Bars and Beats time

## Also in this release

- Spectral EQ
- Buses, sends and sidechain keys
- VST3 plug-ins in Voice Chains, each in its own process so a bad plug-in cannot take the editor down
- Phase Wheel and Properties panels
- 3D Space panel (Window menu) with spots, stereo pair handles, snapping, faders and automation
- Voice Chain redesign: knobs, a menu per slot, effect families
- Help > Check for Updates
- New zoom shortcuts
- Thanks to Broman (nwh212) for the music and plug-in work

## Known limits

- MIDI, Amp Model, real VST3 plug-ins, buses and 3D Space have had little hardware testing. Keep backups.
- Sessions saved in 0.3.0 can hold new data that 0.2.0 and older drop when they save.
- A session that uses a VST3 plug-in may need the plug-in picked again on another operating system.
- Windows installer is unsigned: SmartScreen shows a warning (More info, then Run anyway). Linux and macOS builds are experimental.
