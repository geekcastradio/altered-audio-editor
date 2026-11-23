# Hand test for 0.65.0

Things to try on Windows. Each line says what to do and what should happen.

## Bounce to New Track
1. Open a session with two tracks that each have clips. Click a track name to select it. Multitrack > Bounce to New Track > Selected Track. A new track appears with one clip that spans the first clip start to the last clip end, and it sounds like the track. Undo removes it in one step.
2. Select an empty track. The same menu item is greyed out with the reason "Select a track that has clips."
3. Select two clips, then drag a time selection across only part of them. Bounce to New Track > Selected Clips in Time Selection. The new clip covers only the time selection and holds only those clips.
4. With no time selection, the same item is greyed out with a reason.
5. Right-click an empty part of a lane: the same two entries are there and work.

## Voice Chain preset favorites
1. Open the Voice Chain panel. The star beside the Presets row is greyed out until you pick a preset.
2. Pick a preset, then click the star. It lights up. Open the Presets menu: a Favorites entry is at the top and lists that preset.
3. Star a second preset. Favorites lists both, in the order you starred them. Click one: it applies like any other pick.
4. Click the star again on the lit preset: it is removed from Favorites. When none are starred, the Favorites entry is gone.
5. Close and reopen the app: the favorites are still there.
