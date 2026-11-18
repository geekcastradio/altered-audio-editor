# Clean-room policy

Altered Audio Editor aims to work the way people expect an audio editor like Adobe Audition to work. It does that without using any Adobe code, assets or text. This page says how the project keeps that line.

## The rules

1. **Behavior, not code.** Features are written from how an editor behaves when you use it, from public documentation, and from open standards and published research. No Adobe source code, decompiled code or leaked material is used or looked at.
2. **No copied assets.** Icons, artwork, sounds, fonts, screenshots, sample files and wording in the program and in the User Guide are our own, or are under a license recorded in `THIRD-PARTY` files. Nothing is taken from another product.
3. **Our own words.** Menu names, command names and shortcuts follow common editing vocabulary and the shortcuts people already know, because names and key chords are facts about how to use a tool. Help text, error messages and the guide are written for this program.
4. **Files a person already has.** The Audition shortcut import reads a shortcut file that a person exports from their own installation. Session import (.sesx) reads session files that a person already has. Neither includes or ships any Adobe file, and neither needs Adobe software to run.
5. **Nothing proprietary in the repository.** Adobe installers, program files, plug-ins, presets, screenshots and documentation are not added to the repository, issues or pull requests. If a bug report needs a file, the reporter's own recording or session is fine.
6. **Third-party code stays labeled.** Every library we ship has its license recorded and checked before it is added, and the privacy gate keeps network libraries out.

## For contributors

- Describe what the program should do and how it should behave. Do not paste code, text or screenshots from another product.
- If you have seen another program's source code for a feature, tell a maintainer before working on that feature, so we can decide who should write it.
- Report anything in the repository that looks copied. We will check it and remove it if it is.

## Trademarks

Adobe and Audition are trademarks of their owners. They are used only to say what a feature is compatible with or similar to. This project is not made by, endorsed by or connected to Adobe.

## Questions

Open an issue or write to the maintainers at The GCRN.
