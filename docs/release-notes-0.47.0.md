## Altered Audio Editor 0.47.0

### Under the hood
- More of the move to a Rust interface. Settings backups (export, restore, snapshot, options) and session templates (list, save, delete) now get their folders and the app version from the `AppCore` handle instead of asking the Tauri window. No change in what you see.
