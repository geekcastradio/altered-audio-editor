# Altered Audio Editor 0.33.0

A buffer size choice for the audio driver, marked experimental.

## What's new

- **Buffer size (experimental).** Settings > Audio Devices has a new Buffer size choice, from 64 to 2048 frames, with Driver default as the start. A smaller buffer lowers the delay when you monitor a mic and can crackle on a slow computer. A larger one is steadier. It applies to playback, recording and previews from the next time they start.

## Good to know

- It is a request. Many drivers keep their own buffer, and a driver that does not report its sizes is left alone. ASIO drivers may use the size from their own control panel.
- If a size stops sound from opening, the error says to set it back to Driver default.
- It has been tried on very few devices. Tell us what you used and what happened.
