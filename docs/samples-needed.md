# Samples needed from you (and how to make them)

Several features cannot be tuned or tested without real recordings. Each item says what to record, how long, and where to put it. Real recordings stay out of the public repository: keep them in your private samples folder (`C:\Users\scp21\GCRN\samples`) and tell me the file names.

You do not need studio quality. Normal podcast recordings are better than clean ones, because the problems we are tuning for are real ones.

## 1. Double-ender (for Sync Tracks drift correction)

What it is: two people each record their own side locally while on a call, so the two files drift slightly apart over time.

1. Record a call of 15 to 30 minutes with one other person. Each of you records locally (for example a local WAV or FLAC recorder on each side).
2. Both of you clap once at the start and once at the end, close to the mic.
3. Save both files, unedited. Note the sample rate each side used if you know it.
4. Put them in a folder named `double-ender` with names like `host.wav` and `guest.wav`.

Why: Sync Tracks lines up the start. Drift correction needs two files that really did drift, with a known clap at each end.

## 2. Close Dead Space and Isolated Clips (tuning)

1. Record 10 to 15 minutes of normal talking with natural pauses, breaths and a few "um" gaps. One voice is enough; two is better.
2. Include a few short stray sounds on purpose: a cough, a chair creak, a mouse click, a short laugh between sentences.
3. Save it unedited as `talk-pauses.wav`.
4. In a text file next to it (`talk-pauses.txt`) write down roughly where the stray sounds are (minute:second), so I can check what the tool finds against what you know is there.

## 3. Takes polish

1. Record the same voiceover line five times in a row in the Takes panel flow you would normally use.
2. Save the session (`.altaudio`) and the recorded files together in a folder named `takes`.
3. Write down what felt clumsy while doing it (which click, what you expected). Notes matter more than the files here.

## 4. Issue #149 (sample clip)

Open the issue; it names the file type and situation it needs. Send a short clip (under a minute is fine) that shows the problem. If you do not have one, say so on the issue and I will mark it as waiting.

## How to hand them over

Copy the folders into your samples folder, then tell me the folder names in chat or add the Windows path to the issue. I cannot see your computer unless the desktop app is connected with that folder added. If it is, I can read the files from there.

## 5. Audition sessions (for better `.sesx` import)

1. Make or pick two real sessions you already have from your Audition days: one with drawn clip volume lines (envelopes) and fades, one with effects on tracks and clips (EQ, compressor, noise reduction, limiter).
2. Copy each session folder with its media into a folder named `sesx-real`. Remove anything private.
3. For each one, write two lines: what it should sound like and what you remember being loud or quiet. This lets me check gain staging (an import that comes out 18 dB too loud is a known trap).

## 6. Audacity projects (for `.aup3` import)

1. Make three small projects in Audacity: one with a single track, one with three tracks and a label track, one with cut and moved clips and a silence gap.
2. Save each as `.aup3` into a folder named `aup3`. Keep each under about 20 MB.
3. Write down what you see in each (track names, label text), so I can check the import against it.
