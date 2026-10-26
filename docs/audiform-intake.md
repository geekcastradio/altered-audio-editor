# Audiform intake

Audiform (github.com/nwh212/Audiform) is nwh212's fork of Altered Audio Editor. It is GPL v3 or later, credits this project, and offers its changes under our CLA so they can come back. Nothing is merged automatically. This page is how we decide.

## Rules

1. **Features, not looks.** We take features. We do not take changes to our layouts, menus, panel names, colors, icons, logo or wording. Audiform's own name, brand and credits stay in Audiform.
2. **One feature at a time.** Each feature comes in on its own branch, built from our `main`, with CI green and `npm run check` passing (headers, privacy, private terms).
3. **Scope test.** It must help podcasting, voiceover or work Adobe Audition users expect, or sit behind Music Mode. The Default, Podcast and Voiceover layouts must not change because of it.
4. **Hands-on test.** Anything that touches audio devices, plug-ins or channel layouts is tested on real hardware before a release.
5. **Rights.** The author has agreed to `CLA.md` for it. Anything by someone else waits until they agree.
6. **No whole-branch merges.** Audiform rewrote the copyright header in about 350 files, so merging its branch would conflict everywhere and would pull in its name. We cherry-pick feature commits, then put our headers and wording back (`Audiform`, `FX Chain` and `Based on` lines come out).

## Verdicts on Audiform's commits (to 2026-10-26)

| Commit | What it is | Verdict |
| --- | --- | --- |
| 79f288b | Rename, new icon and logo, credits, reports and releases point to Audiform, settings carry-over | Skip. Identity of the fork. |
| 932e944 | Shift+Space records. Removes an old rule that pulled the view back to the playhead | Take. Shift+Space is Audition's Record shortcut. The old rule was a real bug. |
| 259ad4c | Renames Voice Chain to FX Chain, slim track grab bar | Skip. Our naming and look. |
| e67fd0f | FX panel owner colors, dashed empty slots, mono icon, "any row shows its chain" | Skip the looks. The "any row shows its chain" fix may be worth porting by hand later. |
| c860b62 | Source Channels: multichannel files keep every channel; per-clip left/right amounts with presets | Take, with one change: the Properties section shows only for files of more than two channels. |
| 608384c | Multichannel files in the Waveform editor: every channel, edits and saves keep all channels | Take. Useful for multi-track recorder files. Test with a real multichannel WAV. |
| b65448d | Channel Routing settings (pick output channels), Speakers listen mode in 3D Space | Take Channel Routing settings first. Speakers mode later (needs a multi-output interface to test). |
| fab9bf3 | Multichannel clips play each channel from its own speaker in 3D Space | Later, with Speakers mode. |
| 870be64 | clippy `as_chunks` fixes | Skip. We already made these changes. |
| db8c494, 3a28d07 | README wording | Skip. |
| d8cf344 | New mark as app icon and logo | Skip. |
| test-build.yml | Windows test build on every push, as a download | Optional. Useful for testing without tagging a release. |

## How to bring one in

```
git remote add audiform https://github.com/nwh212/Audiform.git
git fetch audiform
git checkout -b intake-<feature> main
git cherry-pick -n <commit>        # -n: apply without committing, then fix
git grep -n "Audiform\|FX Chain\|Based on"   # put our wording back
npm run check; npm test
```

Then the usual cargo checks, a hands-on test, and a pull request or a merge into `main`.
