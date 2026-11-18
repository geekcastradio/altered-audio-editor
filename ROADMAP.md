# Roadmap

What is planned, what is not, and where outside help is most useful. The project is local-first: no accounts, no telemetry, and no network use beyond the update check. See [CONTRIBUTING.md](CONTRIBUTING.md) for how to take part and [docs/backlog.md](docs/backlog.md) for the detailed plan.

## Where the project is

Altered Audio Editor is a free, open-source audio editor for podcasters and voiceover work. It follows Adobe Audition's layout, shortcuts and workflow closely, and adds tools for podcast work such as multi-mic cleanup, process chains and batch loudness. It is not at 1.0 yet, and releases come as features and fixes are ready.

## The way to 1.0

There is no date for 1.0. It is the stable, no-network release, and it comes when the program is solid, not when a calendar says so.

1. **Close the gaps.** Anything the reference editor has natively and we lack is built or, if it doesn't belong, removed. A 1.0 has no greyed-out "not built yet" menu items.
2. **Fix bugs and performance problems** as people report them, with real files and real machines.
3. **Soak.** Fixes only, until there are two clean weeks with no crash or freeze reports.
4. **Lock the session file format**, with a test fixture from every earlier version.
5. **Final checks:** a clean install and an upgrade on Windows, a smoke test on Linux and Mac, and a read through the User Guide.

## After 1.0

- **1.1: transcripts.** Local transcription and a script panel for editing by text, with reviewed filler-word cleanup (a person keeps or cuts each item, never automatic), all on your own computer.
- **1.2: automation.** A local-only endpoint, off by default, so scripts and assistants can run process chains and read session information, with a visible on/off switch and a log of what was done. Every change is a labeled, undoable step. If anything here ever needs the network, it will be opt-in and off by default, and these documents will say so.
- **Only if people ask:** hosting CLAP plug-ins.

## Not planned

- The built-in effects as separate plug-ins, or a plug-in store.
- Cloud or guest recording. For a remote guest, record locally on both ends and sync the files (see the User Guide).
- Audio Units, LADSPA, LV2 and ARA hosting.
- VST2 plug-ins (not supported by design).

## How to help

- **Test on hardware.** A Mac, other Linux distributions, ASIO audio interfaces, and a Mackie-style controller are the gaps. Tell us what happened with your device and driver.
- **Share recordings you are free to share.** A double-ender (two local recordings of the same conversation) helps with drift correction, and two-mic recordings help with bleed and mic mixing.
- **Report bugs** with Help > Report a Bug, and attach the file or clip that shows the problem.
- **Take an issue.** Look for the `good first issue` and `help wanted` labels on the issue tracker.
