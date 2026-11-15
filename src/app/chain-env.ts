// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * The app's side of the process chain steps: the measurements and saved
 * presets a step may need. Kept apart from the step logic so that logic
 * runs in plain tests with nothing from the app behind it.
 */

import { effectsCatalog, loudnessCheck, quickpassLevels } from "./core";
import type { ChainEnv } from "./chain-actions";
import { sanitizePresets } from "../panels/voice-chain/chain";
import { prefs } from "../stores/prefs";

/** Where the Voice Chain panel keeps the user's saved presets. */
const KEY_PRESETS = "voiceChain.presets";

/** Same window the QuickPass panel measures in. */
const WINDOW_MS = 10;

export function chainEnv(): ChainEnv {
  return {
    measure: () => loudnessCheck({}),
    levels: (ids) => quickpassLevels(ids, WINDOW_MS, false),
    presets: async () => ({
      saved: sanitizePresets(prefs.raw(KEY_PRESETS)),
      known: new Set((await effectsCatalog()).map((x) => x.id)),
    }),
  };
}
