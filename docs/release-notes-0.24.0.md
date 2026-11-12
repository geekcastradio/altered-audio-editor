# Altered Audio Editor 0.24.0

Multi-mic shows. One mic per person, mixed for you.

## What's new

- **Auto Mic Mix.** Multitrack > Auto Mic Mix and Talk Time. Pick the mic tracks, and it finds who is talking and writes a volume lane on each mic that turns it down while someone else speaks. Each mic is judged against its own quiet level, so a mic that hears the others a little does not hold the mix open. When nobody talks, every mic stays open so the room sound stays even. The start of a word is not cut: mics open fast and close slowly, with a short hold.
- **Talk Time.** The same measurement says how long each voice spoke and what share of the conversation that was.
- **Plain volume lanes.** The result is ordinary automation on each track's volume lane. Edit it by hand, or press Undo once to put every lane back. Other lanes on the track are left alone.
- **Long shows.** The audio is read a minute at a time, so a two-hour recording does not have to fit in memory.

## Good to know

- Talk time is by level. Laughing and crosstalk count as talking.
- It works best when each person has their own mic. Two people on one mic is one voice to it.
- Run it before the Voice Chain's gates if you use them; it reads the raw audio, without effects or faders.
