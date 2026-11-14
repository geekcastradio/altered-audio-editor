# 0.27.0 hand test

Run on the installer build from the draft release. Tick each line. This list grows as 0.27.0 gets built.

## Dialogs (shared look)
Dialogs are being moved onto one shared shell a few at a time. In this build: Time Stretch, Fit Music to Length, Add Tracks, Generate Tone, Generate Noise, Insert Silence, Recording Options, File Tags, New Audio File, the one-line name box used for renaming, Pitch Tune and Curve, Match EQ, Auto Mic Mix, Sync Tracks, Export Markers and Chapters, and the clip color picker.

Time Stretch and Fit Music to Length
- [ ] Select one clip, Clip > Time Stretch. The dialog opens, the cursor is in New length, and the text is selected.
- [ ] Buttons read Cancel then Apply, left to right, with Apply in the accent color. Remove Stretch (only when the clip is stretched) sits at the far left.
- [ ] Enter in the length box applies. Escape closes. Clicking outside the dialog closes it.
- [ ] Press the mouse inside the length box, drag out past the dialog edge and release. The dialog stays open.
- [ ] With no time selection, Fit to Time Selection is dimmed and a line under the quick buttons says why.
- [ ] Clip > Fit Music to Length. Same look as Time Stretch: same field height, same button shape, buttons read Cancel, Find Joins, Apply.
- [ ] In Fit Music, Enter in the length box runs Find Joins.

The rest
- [ ] Multitrack > Add Track > Add Tracks: Cancel then Add. Enter in the Name box adds. Escape closes.
- [ ] Generate Tone, Generate Noise and Insert Silence: Cancel then the action. Enter applies when the length is valid and does nothing when it is not.
- [ ] Recording Options: Cancel then OK. The two number boxes are the same size, and the punch-in boxes dim when punch-in is off.
- [ ] File > Edit File Tags: the first field is focused. The Comment box lines up with its label. Cancel then Save Tags.
- [ ] If you open File Tags from another dialog (Publish, for example), Tags sits on top.
- [ ] File > New Audio File: the Name is selected, Enter creates the file, Escape cancels.
- [ ] Rename a track: the name is selected, Enter confirms, Escape cancels, the OK button is dim while the name is empty.

Measuring dialogs and the rest
- [ ] Match EQ, Auto Mic Mix and Sync Tracks: the buttons read Close, then Measure (or Match). After a result, the accent button is the action that uses it: Add EQ to Track, Write Volume Lanes, Move Track. Close is no longer the colored one.
- [ ] While one of them is measuring, Escape and a click outside do nothing, and the buttons are replaced by the "Measuring" line.
- [ ] Pitch Tune and Curve: the two tabs switch, the sliders line up in three columns, and Cancel and Apply are dim while it works.
- [ ] Export Markers and Chapters: Close then Save. The preview box scrolls and stays monospaced.
- [ ] Clip color (right-click a clip, Clip Color): the first swatch has the focus ring, the six swatches and the Other color box line up, Use Track Color is on the left and Cancel on the right.

Behaviour that should hold in all of them
- [ ] With two dialogs open, Escape closes only the top one.
- [ ] Resize the window small. A dialog scrolls instead of running off the screen.
