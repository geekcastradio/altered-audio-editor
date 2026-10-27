# Altered Audio Editor 0.5.1

A fix for ASIO devices that sometimes do not show up.

## Fixed
- **ASIO devices missing in Settings.** If the Play through and input lists showed only "System default" while ASIO was chosen (reported after a plug-in failed to load, and when the app was started right after an update), the app now asks the driver again a few times before giving up, and Settings tries twice more by itself. A new **Refresh Devices** button in Settings > Audio Devices asks again on demand, and a note explains what is happening if the lists stay empty.

## Good to know
- The cause is an ASIO driver being open in one program at a time. We could not reproduce the problem, so if the lists are still empty after Refresh Devices, please tell us what else was running.

## Test checklist
- Choose ASIO in Settings > Audio Devices: your interface's devices appear
- With ASIO chosen, start a recording in another app that holds the interface, open Settings, and confirm the note and Refresh Devices appear; close the other app, press Refresh Devices, and the lists fill
- Switch back to System: lists appear immediately, as before
