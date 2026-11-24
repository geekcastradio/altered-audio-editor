# Altered Audio Editor 0.68.0

Sessions from another editor keep their unread effects for the trip back out, FLAC export gets compression levels, and three import labels stop naming another product.

## New

- **Skipped effects are kept, not lost.** When you open a `.sesx` session, effects we have no equivalent for (a noise remover, a leveler, a reverb) still do not play, and the import notice still lists them by name. Their original settings are now stored unread inside the session, and File > Export > Multitrack Session writes them back into their slots. Saving as `.altaudio` and reopening keeps them too.
- **FLAC compression levels, 0 to 8.** Pick one in the export dialog or in Batch Convert. The default is 5, which is what FLAC exports used before, so nothing changes unless you choose. Every level is lossless: the samples are identical. Higher levels make the file a little smaller and take longer to write. On speech the gap between levels is modest.

## Fixes

- **Three import labels no longer name another editor.** They now read "Import shortcuts...", "Import chains from a .xml file..." and a plain description in the "no chains found" notice.

## Known limitations

Altered Audio Editor is young. Windows is the platform we test. Please keep a backup copy of any session that matters.

- **Linux and macOS are experimental.** They are built from the same source but have had little testing. The macOS app is not notarized: the first time, right-click it and choose Open. Cmd is not yet used in place of Ctrl on macOS.
- **ASIO has not been tested on real hardware.** If an ASIO driver cannot open, the system driver plays instead. Reports from people with ASIO interfaces are welcome.
- **VST3 plug-ins, MIDI tracks, buses and 3D Space have had little hardware testing.** A plug-in that crashes is kept in its own process, so the editor should carry on, but tell us which plug-in it was. A session that uses a VST3 plug-in may need the plug-in picked again on another operating system.
- **AAC and M4A export off Windows uses a built-in encoder** that has been tested against other decoders but not by ear on Linux or macOS. Windows uses its own system encoder. Monkey's Audio and Windows Media formats are not planned.
- **Hardware controllers** map faders, knobs, buttons and the transport, and motor faders, button lights and pan rings follow the screen (Mackie Control layout, not tried on a real surface). Scribble strips and jog wheels are not built.
- **Opening other programs' sessions** brings in tracks, clips, markers and the effects we have an equivalent for. Effects we do not have are left out of the sound and listed in the import notice, and their original settings are kept so a `.sesx` export writes them back.
- **Transcription, filler word removal, text-based editing, Music Remover and publishing from inside the app** are planned for after 1.0. Nothing in the app uses the network without you turning it on.
- **Some tests are by hand only.** Recording behavior, live meters and the look of the window are checked by a person, not by the automatic tests.
