## Altered Audio Editor 0.22.0

Control surfaces that listen back.

### Controllers
- **Motorized faders follow the screen.** Edit > Controllers has a new option, "Send feedback to the devices". With it on, a control surface in its Mackie mode gets the fader positions from the session: when you move a fader with the mouse, undo a change, page to the next eight tracks or open another session, the motorized faders go there. A fader you are touching is left alone, and when you let go it stays where you put it instead of snapping back.
- **Lights.** Mute, solo and arm lights follow the tracks, and the pan rings show each track's pan.
- **Off by default, and only for devices you tick.** Nothing is sent to a device unless the option is on, and only short fader, note and knob messages go out. Leave it off for keyboards and anything that is not a control surface.
- **Needs the Mackie layout.** Feedback uses the Mackie Control message layout, which is what "Use Mackie Control Layout" sets up. Faders with other assignments are not driven.
- **Not yet:** faders do not move to follow automation while a session plays.
