# Review guide for Gemini Code Assist

Review pull requests against `AGENTS.md` in the repository root. Flag in particular:

- Any name, icon, color or wording copied from another commercial product.
- Any network request, telemetry or account code.
- A new dependency without a licence check and an `ATTRIBUTION.md` entry.
- A new keyboard shortcut.
- A state change that is not undoable.
- A missing GPLv3 header, a missing test, or a Rust port without a shared fixture that the TypeScript side also reads.
- Em dashes or long sentences in user-facing text.
