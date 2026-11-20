## Altered Audio Editor 0.46.0

### Fixes
- Edit > Settings > Privacy was empty, and its text and update options showed on the Automation page instead. The Privacy page now shows them. A test now checks that every Settings page has something drawing it. (Reported by Steve on 0.42.0.)

### Under the hood
- More of the move to a Rust interface. The core now works out its settings and backup folders once at start-up and keeps them in the `AppCore` handle. Shortcuts, preferences and saved layouts are the first commands to use it instead of asking the Tauri window for the folder. No change in what you see.
