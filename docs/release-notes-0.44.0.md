## Altered Audio Editor 0.44.0

### Under the hood
- The next step of the move to a Rust interface. The core now has one handle (`AppCore`) that logic uses to reach whatever interface it runs under, starting with the event sink. Commands will move behind it a few at a time. No change in what you see.
- Release builds can now be started from GitHub without tagging first. No change for users.
