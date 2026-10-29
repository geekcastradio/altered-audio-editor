// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

import { writable } from "svelte/store";

/** True while the editor shows its "Nothing open" start screen. Record and the transport read it. */
export const startScreen = writable(false);
