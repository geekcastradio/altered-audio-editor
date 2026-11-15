# Hand test: Reduce Bleed (0.30.0)

Needs a multitrack session with two mics that each hear the other speaker, ideally a real two-mic recording with one clip per mic. Save the session first (the cleaned copies go in a "Bleed Reduced" folder beside it).

1. Multitrack > Reduce Bleed... Pick one mic to clean and the other as the sound to take out. Run.
2. The notice says how many clips were cleaned and by up to how many dB. The clips now play the cleaned copies; the originals in your session folder are untouched.
3. Solo the cleaned track and listen where only the other person talks: their voice should be much quieter. Where the wearer talks, the voice should sound natural, not thin, watery or gated.
4. Double talk: both people at once should keep the wearer's voice at full level.
5. Edit > Undo once brings every original clip back. Redo puts the cleaned ones back.
6. Run it on a track with a locked clip, and on a clip that is looped or stretched: they are left alone and the notice says so.
7. Pick the same track twice: the dialog refuses.
8. Run it on two tracks that do not overlap: the notice says almost no bleed was found and nothing changes.
9. Order: Run Reduce Bleed before Auto Mic Mix. Running it after still works but cleans against the gated mix.

Tell me what you hear: the settings (how long a room the filter follows, how fast it learns, how loud bleed can be) are set from synthetic tests and need your real recordings to tune.
