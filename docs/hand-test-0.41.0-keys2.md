# Hand test 0.41.0, part 2: Ctrl+1 to Ctrl+5, A, Alt+Shift+comma and period

1. In a session, select a clip and copy it with Ctrl+C. Press Ctrl+2 to switch to clipboard 2. Copy a different clip. Press Ctrl+1: Paste (Ctrl+V) should paste the first copy. Ctrl+2: the second. Edit > Set Current Clipboard should tick the slot you picked.
2. Type in a text box (a track name or a dialog field) and press Ctrl+1 to Ctrl+5: they must not change the clipboard slot while you are typing.
3. Play a long session with the view zoomed in. Press A: Follow Playhead should turn off (the toolbar button un-lights and the view stops following). Press A again: it lights and the view follows. View > Follow Playhead should show a tick that matches the toolbar button.
4. Type in a text box and press A: it must type the letter, never toggle Follow Playhead.
5. Multitrack: select a clip that is not locked. Press Alt+Shift+period: it moves right five steps (five times what Alt+period moves it). Alt+Shift+comma moves it left five steps. Edit > Undo should read "Nudge Right 5 Steps" and put it back in one step.
6. With no clip selected, or a locked clip selected, Clip > Nudge > Nudge Left 5 Steps is greyed out with a reason.
7. In the Waveform Editor those two nudge keys do nothing (like Alt+comma and Alt+period).
8. Edit > Keyboard Shortcuts lists the four new commands. Import your Audition shortcut export: the five clipboard keys, A and the two nudge keys come in.

If Alt+Shift+comma or period does nothing on your keyboard, tell me which keyboard layout you use. The keys are bound as Alt+Shift+< and Alt+Shift+> as well, which is what a US keyboard sends.
