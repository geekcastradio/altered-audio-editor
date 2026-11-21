# Hand test: 0.53.0

- Recording follow (the one thing the cloud could not test): open or create an empty file in the Waveform editor and record for a minute at a normal zoom. The view scrolls to keep the end of the new audio in sight. Repeat in a file that already has audio, and zoomed in close. Scroll by hand mid-take: the view holds where you put it. Stop and record again: it follows again.
- Matroska and WebM: open a `.mka` and a `.webm` file (any you have; a podcast saved from a video site works). They are accepted in File > Open, by drag and drop, and from a folder. Each shows a length close to the real one (a few hundredths of a second long at most), plays through to the end, and comes into a track with all of its audio. If one fails, note the codec inside it.
- Loudness Match, Check Loudness and the Analysis panel (Frequency, Statistics, Histogram, Problems): numbers and wording look the same as in 0.52.1.
- Edit > Select (clips to end of track, next clip in track, clips from the playhead): same results as before.
- Help > Report a Problem opens the bug form with the version and system filled in, and nothing else.
- Open a session, play it, edit a clip, save, close and reopen: nothing different from 0.52.1.
- Still wanted from 0.52.0: the startup timing list from a cold start (Help > Copy Bug Report Details), plus a stopwatch time and whether the wait is before or after the window appears.
