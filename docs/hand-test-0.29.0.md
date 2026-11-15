# Hand test: process chain steps (0.29.0)

Three steps join the two from 0.28.0. Settings use underscores in the chain file. `docs/chains/episode-cleanup-and-level.json` is an example.

## Loudness Target (`multitrack.loudness-target`)
Settings: `target` (podcast-stereo, podcast-mono, spotify, broadcast, custom) and `tolerance_db` (default 0.5).
1. Open a session whose mix is clearly quieter than -16 LUFS. Run a chain with only this step. The Master rises and the notice names the change. Check with Effects > Loudness that the mix now reads close to -16.
2. Run it again. It reports it is already close enough and changes nothing.
3. Use a very peaky mix. The Master rises only until the true peak reaches -1 dBTP, and the notice says the peak limit stopped it.
4. Undo once. The Master returns to where it was.

## Voice Chain Preset (`multitrack.voice-chain-preset`)
Settings: `preset` (a built-in or saved Voice Chain preset name) and `tracks` (`all` or a track name).
1. Run it with `"tracks": "all"`. Each track with audio gets the chain, tracks without clips do not. Undo removes them all in one step.
2. Run it with a track name in a different case. Only that track changes.
3. Misspell the track name or the preset. The chain is refused or the step stops, naming what it could not find, and nothing changes.
4. Save a preset of your own with a built-in's name. The chain uses yours.

## QuickPass (`multitrack.quickpass`)
Settings: `preset` (a QuickPass preset that removes or turns down) and `ripple` (none, track or all).
1. Open an episode with long silences. Run it with `ripple: "none"`: the silences are cleared and every track keeps its place. With `"track"` the gaps close on their own track.
2. Compare with the QuickPass panel using the same preset: the same stretches are found.
3. Name a preset that repairs, splits or marks. The step stops and says it needs the QuickPass panel.
4. Muted tracks, muted clips and locked clips are left alone.

## Whole chain
Run `episode-cleanup-and-level.json`. The notice lists every step. One undo takes it all back, one redo puts it all back.

## Session format
Open an older session and a new one. Both open, and saving writes nothing new until takes exist.
