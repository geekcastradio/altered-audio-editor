# Schools and districts

This page is for teachers and the people who manage school computers. It states only what the app does today. Where something is not true or not tested, it says so.

## In short

- **No accounts, no sign-in, no ads, no telemetry.** Students never create or use an account.
- **Works fully offline.** Every feature works without a network connection.
- **Nothing about students or their audio is collected or sent.** Audio, sessions and settings stay in folders on the computer. See [PRIVACY.md](../PRIVACY.md).
- **Free and open source** under GPLv3. No per-seat licence and no license check.

## Installing on many computers

- The release page has an **MSI package** for Group Policy and similar tools, an installer for single computers, and a portable zip that runs from a folder or a USB drive without installing.
- Each release lists SHA-256 checksums (`SHA256SUMS.txt`) so a copy can be verified before it is deployed.
- The app is **not code signed yet**. Managed computers that only allow signed software will need an exception until it is.
- Windows 10 and 11 (64-bit) is the supported and tested system. Linux and macOS builds are experimental.

## Network use

The only time the app uses the network is the update check. It is started by the person (Help > Check for Updates) or turned on in Edit > Settings > Privacy, where it is **off by default**. It reads one small file from this project's GitHub releases and sends nothing about the person or their audio. A school can leave it off and update by deploying a new package. The Help menu opens web pages in the browser only when a link is picked.

## Student work

Sessions and recordings are saved where the student chooses, for example a school network folder or a USB drive. The app keeps autosaves and a waveform cache in the user's own app folders. Nothing leaves the computer unless the student exports or copies it.

## Questions we cannot answer for you

This project is not a vendor and does not sign data-privacy agreements, because it never receives student data. If your district requires a form anyway, tell us what it needs. Whether the app meets your own policies is for your district to decide; the facts above are what we can offer.
