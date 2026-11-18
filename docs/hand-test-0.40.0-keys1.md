# Hand test 0.40.0, part 6: K, L, Ctrl+L and Ctrl+Shift+Space

1. Multitrack: with a session open, press L. It should play. Press L again while playing: nothing should happen (it does not stop).
2. Press K while playing: it should stop, with the playhead handled the way the Stop button does (follows Settings > Return to Start on Stop).
3. Press Space to play, then Ctrl+Shift+Space: it should pause where it is, like Space. Press Ctrl+Shift+Space again: it should play on.
4. Open a file in the Waveform Editor and select a stretch. Press Ctrl+L: the Loop button in the transport bar should light and play should repeat the selection. Press Ctrl+L again: it turns off.
5. In the Multitrack, Multitrack > Loop Playback should be greyed out with a reason (it works in the Waveform Editor). Ctrl+L there should do nothing.
6. While recording, press L and K: L does nothing, K ends the recording the way Stop does.
7. Type in a text box (a track name, a dialog field) and press K, L, Ctrl+L: they must type or do nothing, never act on the transport.
8. Edit > Keyboard Shortcuts: the four new keys appear. Import an Audition shortcut export that has J, K, L and Ctrl+L: K, L, Ctrl+L and the pause key come in; J has nothing to map to and is left out.
9. Help > Keyboard Shortcuts and the User Guide list the new keys.

Not built: J (play backward). Audition's shuttle also changes speed with repeated presses; here L only plays. Tell me if you miss that.
