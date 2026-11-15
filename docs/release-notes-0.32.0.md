## 0.32.0

### New

- Takes and comping: Clip > Make Takes from Selected Clips turns clips stacked over one passage of a track into a set of takes. Only the chosen take plays; the others stay on the track, drawn dim. The new Takes panel lists the sets. Use a take over the time selection (or the whole stretch) to comp: the editor cuts the takes at the edges and switches the playing piece, one undo step each. Flatten Takes drops the silent pieces and turns the rest back into ordinary clips. Delete a take from the panel.
- Settings > Multitrack: Take lanes on or off. Off hides the commands and the panel's tools; sessions that have takes still play the chosen takes.
- Takes that are not playing are left out of mixdowns and of .sesx export.

### Known

- Recording with Loop on does not make one take per lap yet. Record again over the same stretch, then select the clips and make takes.
- A session saved with takes opens in older builds with every take playing at once.
