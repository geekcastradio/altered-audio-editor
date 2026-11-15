# Altered Audio Editor 0.29.0

Process chains grow from two steps to five, so one saved file can clean up, tone and level an episode.

## What's new

- **Loudness Target step.** A chain can now finish by setting the Master so the mix lands on a loudness target (podcast, louder platforms, broadcast or your custom one). It measures the mix after the earlier steps have run, uses the same rule as an export, and stops short rather than push peaks past the target's limit, saying so in the notice. A mix already close enough is left alone.
- **Voice Chain Preset step.** A chain can give a track a Voice Chain preset by name, built in or one you saved, for one track by its name or for every track with audio. Put one step per track to give each voice its own chain.
- **QuickPass step.** A chain can run a QuickPass preset that removes or turns down what it finds, such as Clean Up Podcast Interview. Choose whether the gaps stay open so every track keeps its place, or close on the track or on all tracks.
- **A second example chain.** docs/chains/episode-cleanup-and-level.json mixes the mics, closes dead space and levels the result in one run. Like every chain, it is one undo.

## Good to know

- A chain is refused before anything changes if a step is unknown, has a setting out of range, or names a preset or track that does not exist. The notice says which step.
- A Voice Chain Preset step replaces the track's chain. Single-effect presets add their effect to the end, so running the same chain twice adds it twice.
- The QuickPass step runs the silence, noise and audio scans with a preset that removes or turns down. Presets that repair, split or mark still need the QuickPass panel.
- Sessions now have room for take information, which a later release uses for takes and comping. Nothing sets it yet, and your sessions open and save as before.
