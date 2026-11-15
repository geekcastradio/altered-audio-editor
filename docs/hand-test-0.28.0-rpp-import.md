# Hand test: .rpp import (0.28.0)

1. File > Import Session (.rpp)... Pick a project with a few tracks and items. A new, unsaved session opens with the same track names, order, mute and solo.
2. Clip positions match the project (play both side by side at a few spots). Item fades, volume and start offsets match.
3. An item whose play rate is not 1 opens as a stretched clip. Check by ear that the length matches.
4. Markers and regions appear. A region has a length.
5. The notice names what was left out (plug-ins, sends, envelopes) and how many media files were not found.
6. Move the project to another folder without its media: clips are placed, media shows missing, relinking works.
7. Try a plain text file renamed to .rpp: a clear error, nothing changes in the open session.
8. File > Open Recent lists the project; picking it imports again.
