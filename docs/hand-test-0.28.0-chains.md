# Hand test: Run Process Chain (0.28.0)

Chain files are plain JSON. `docs/chains/episode-cleanup.json` is an example. Two steps exist today: `multitrack.auto-mic-mix` and `multitrack.close-dead-space`. Settings are the same as the menu commands, written with underscores (`reduce_db`, `pause_seconds`).

1. Open a multitrack session with two or more mic tracks and some silence between speakers.
2. Multitrack > Run Process Chain... Pick `episode-cleanup.json`.
3. Both steps run. The notice lists what each did. Check the volume lanes and the shortened pauses.
4. Edit > Undo once puts everything back (lanes and positions). Redo applies both again.
5. Edit the file: change a step to `"action": "nope"`. Run it: the notice names step 1 and nothing changes.
6. Set `"pause_seconds": 9`: refused with the allowed range, nothing changes.
7. Lock a clip, run the chain with close-dead-space `"on_error": "stop"`: the chain stops and the mic mix is taken back. With `"on_error": "skip"` the mix stays and the notice says the step was skipped.
8. A session with one track: the mic mix step reports "needs two or more tracks" and the chain carries on.
