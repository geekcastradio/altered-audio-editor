#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Copyright (c) The GeekCast Radio Network, LLC.
# This file is part of Altered Audio Editor.
#
# Builds the macOS app and disk image, trying again when the disk image step fails.
# The disk image tool on the hosted Mac runners sometimes fails once and then works
# (a leftover mounted image or a busy volume). The Rust code is already built by the
# time the image step runs, so a second try only repeats the bundling and takes seconds.
#
# Usage: scripts/build-mac.sh [extra arguments for tauri build]
set -u

clean_up() {
  # Unmount images left behind by a failed attempt and remove half-written ones.
  for vol in /Volumes/dmg.* "/Volumes/Altered Audio Editor"*; do
    [ -e "$vol" ] && hdiutil detach -force "$vol" >/dev/null 2>&1 || true
  done
  find src-tauri/target -name 'rw.*.dmg' -delete 2>/dev/null || true
}

for attempt in 1 2 3; do
  clean_up
  if npm run tauri build -- --bundles app,dmg "$@"; then
    exit 0
  fi
  echo "::warning::Mac build attempt $attempt of 3 did not finish; trying again."
  sleep 15
done
exit 1
