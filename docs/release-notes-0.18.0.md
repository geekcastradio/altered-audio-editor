## Altered Audio Editor 0.18.0

Opening one file opens it as itself, fades cover the whole file, and sessions bring their markers across.

### Opening files
- **One audio file opens in the Waveform editor.** Open or drop a single audio file and you get that file, not a one-track multitrack. This includes dropping a file from your file manager on an empty session. Two or more files still build a multitrack.

### Editing
- **Linear fades.** Amplitude now has Fade In (Linear) and Fade Out (Linear) beside the two existing fades.
- **Fades cover the whole file when nothing is selected.** Before, a fade with no selection did nothing.
- **The timeline scrolls sideways while you drag.** Drag a time selection or a box past the edge of the lanes and the view follows, faster the further you go.

### Libraries
- **Drop files onto a bin.** Drag files from your file manager onto a bin's name. They are added to the library and filed in that bin, and the drop area names the bin while you hover.
- **Tag several items at once.** Select two or more library items and add or remove tags on all of them in one step. It is a single undo.

### Sessions
- **Markers come across when a .sesx lists them in its marker list.** Each keeps its name, position and length, and the import notice says how many came across. Markers that another editor saves in a session's own metadata are not read yet.

### QuickPass and bug reports
- **QuickPass shows its progress count** while it works through a file.
- **Report a Bug opens a form with your version and system already filled in.**
