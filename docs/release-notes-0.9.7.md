# Altered Audio Editor 0.9.7

Two bug fixes from testers (issues #27 and #28).

- **VST3 settings no longer snap back to default** (#27). Plug-ins used in the Waveform editor are now kept in step like the ones in the Multitrack session: what you set in a plug-in's own window is read back into the chain, so moving the playhead, editing another effect or pressing Apply uses your settings. While a plug-in window is open (and nothing is playing) its settings are read every couple of seconds, because many plug-ins never announce that they changed. Apply to Selection and Apply to Whole File read the settings first. Changing the multitrack session no longer stops plug-ins that belong to an open audio file.
- **New audio files show up in Files** (#28). File > New Audio File, Paste to New File and Mix Down to New File now list the file at the top of the Files panel (a note marks it New until saved), along with other audio files open in the Waveform editor. Double-click shows it, the x or Delete closes it (asking first if there are unsaved edits).
