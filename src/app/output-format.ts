// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

/**
 * Output file settings shared by Export Mixdown, Batch Convert and Save
 * As: format, sample rate, channels, bit depth and MP3 quality. Field
 * names match the core's `OutputFormat`, so a value goes over as is.
 */

export type FormatKind = "wav" | "flac" | "mp3" | "m4a" | "aac";
export type Mp3Mode = "cbr" | "vbr";
export type DitherKind = "rectangular" | "triangular" | "gaussian";
export type NoiseShaping = "none" | "light" | "strong";
export type ResampleQuality = "draft" | "good" | "best" | "highest";

export interface OutputFormat {
  kind: FormatKind;
  /** WAV 4 (IMA ADPCM), 8, 16, 24 or 32 (float); FLAC 16 or 24. Null keeps the source's. */
  bitDepth: number | null;
  /** Null keeps the source's rate. */
  sampleRate: number | null;
  /** Null keeps the source's channels. */
  channels: 1 | 2 | null;
  mp3Mode: Mp3Mode;
  mp3Bitrate: number;
  /** 0 (best) to 9 (smallest). */
  mp3Quality: number;
  /** AAC kbps: 96, 128, 160 or 192. */
  aacBitrate: number;
  /** Dither when writing fewer bits. On by default. */
  dither: boolean;
  /** The shape of the dither noise. Triangular is the standard. */
  ditherKind: DitherKind;
  /** Dither size in steps of the target depth, 0.25 to 2. One is standard. */
  ditherAmount: number;
  /** Moves rounding noise away from where the ear is most sensitive. */
  noiseShaping: NoiseShaping;
  /** How carefully a sample rate change filters. */
  resampleQuality: ResampleQuality;
  /** Stereo to mono: how much of each side goes in, 0 to 1. */
  mixLeft: number;
  mixRight: number;
}

export const FORMAT_KINDS: {
  value: FormatKind;
  label: string;
  lossy: boolean;
  group: "main" | "other";
}[] = [
  { value: "wav", label: "WAV", lossy: false, group: "main" },
  {
    value: "flac",
    label: "FLAC (lossless, smaller)",
    lossy: false,
    group: "main",
  },
  { value: "mp3", label: "MP3", lossy: true, group: "main" },
  { value: "m4a", label: "M4A (AAC)", lossy: true, group: "main" },
  { value: "aac", label: "AAC (raw .aac stream)", lossy: true, group: "other" },
];

/** What each format is for, shown behind the (i) next to the Format menu. */
export const FORMAT_INFO: Record<FormatKind, string> = {
  wav: "Uncompressed and exact. The best choice for a master you will keep editing, and what most editors and broadcast systems expect. Files are large.",
  flac: "Lossless compression: it sounds identical to WAV and takes about half the space. Good for archives and masters. Not every podcast host accepts it.",
  mp3: "The format podcast hosts and players all accept. Lossy, so keep a WAV or FLAC master. 128 kbps is plenty for speech, 192 for music.",
  m4a: "AAC in an M4A file. Sounds better than MP3 at the same size and works in Apple Podcasts and most players. Lossy. Chapters and artwork are supported.",
  aac: "A bare AAC stream with no wrapper (.aac). Few podcast tools want this: it has no tags or chapters. Pick M4A unless a system asks for .aac.",
};

/** AAC uses the encoder built into Windows; other systems get it later. */
export function formatAvailable(kind: FormatKind, platform: string): boolean {
  return (kind !== "m4a" && kind !== "aac") || platform === "windows";
}

export const AAC_BITRATES = [96, 128, 160, 192] as const;
/** Rates the system AAC encoder takes. */
export const AAC_RATES = [44_100, 48_000] as const;

export const OUTPUT_RATES = [
  8_000, 11_025, 16_000, 22_050, 32_000, 44_100, 48_000, 88_200, 96_000,
] as const;
export const MP3_BITRATES = [
  32, 48, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
] as const;
/** Highest rate an MP3 carries. */
export const MP3_MAX_RATE = 48_000;

export const DEFAULT_FORMAT: OutputFormat = {
  kind: "wav",
  bitDepth: null,
  sampleRate: null,
  channels: null,
  mp3Mode: "cbr",
  mp3Bitrate: 192,
  mp3Quality: 2,
  aacBitrate: 128,
  dither: true,
  ditherKind: "triangular",
  ditherAmount: 1,
  noiseShaping: "none",
  resampleQuality: "best",
  mixLeft: 0.5,
  mixRight: 0.5,
};

export const DITHER_CHOICES: { value: "none" | DitherKind; label: string }[] = [
  { value: "triangular", label: "Triangular (standard)" },
  { value: "rectangular", label: "Rectangular (quietest)" },
  { value: "gaussian", label: "Gaussian (most natural)" },
  { value: "none", label: "None" },
];
export const DITHER_AMOUNTS: { value: number; label: string }[] = [
  { value: 0.5, label: "Half (0.5 step)" },
  { value: 1, label: "Standard (1 step)" },
  { value: 1.5, label: "More (1.5 steps)" },
  { value: 2, label: "Most (2 steps)" },
];
export const SHAPING_CHOICES: { value: NoiseShaping; label: string }[] = [
  { value: "none", label: "None" },
  { value: "light", label: "Light" },
  { value: "strong", label: "Strong" },
];
export const RESAMPLE_CHOICES: { value: ResampleQuality; label: string }[] = [
  { value: "draft", label: "Draft (fastest)" },
  { value: "good", label: "Good" },
  { value: "best", label: "Best (default)" },
  { value: "highest", label: "Highest (slowest)" },
];

/** The one dither menu: off, or one of the noise shapes. */
export const ditherChoice = (f: OutputFormat): "none" | DitherKind =>
  f.dither ? f.ditherKind : "none";

export function withDither(
  f: OutputFormat,
  choice: "none" | DitherKind,
): OutputFormat {
  return choice === "none"
    ? { ...f, dither: false }
    : { ...f, dither: true, ditherKind: choice };
}

/** Common targets, applied to rate, channels and depth in one pick. */
export const SAMPLE_PRESETS: {
  id: string;
  label: string;
  patch: Pick<OutputFormat, "sampleRate" | "channels" | "bitDepth"> &
    Partial<OutputFormat>;
}[] = [
  {
    id: "cd",
    label: "CD audio (44.1 kHz, 16-bit stereo)",
    patch: {
      sampleRate: 44_100,
      channels: 2,
      bitDepth: 16,
      dither: true,
      ditherKind: "triangular",
      noiseShaping: "light",
    },
  },
  {
    id: "podcast-mono",
    label: "Podcast mono (44.1 kHz, 16-bit)",
    patch: { sampleRate: 44_100, channels: 1, bitDepth: 16 },
  },
  {
    id: "podcast-stereo",
    label: "Podcast stereo (44.1 kHz, 16-bit)",
    patch: { sampleRate: 44_100, channels: 2, bitDepth: 16 },
  },
  {
    id: "video",
    label: "Video and broadcast (48 kHz, 24-bit)",
    patch: { sampleRate: 48_000, channels: 2, bitDepth: 24 },
  },
  {
    id: "hires",
    label: "High resolution (96 kHz, 24-bit)",
    patch: { sampleRate: 96_000, channels: 2, bitDepth: 24 },
  },
  {
    id: "voice",
    label: "Voice only (22.05 kHz mono, 16-bit)",
    patch: { sampleRate: 22_050, channels: 1, bitDepth: 16 },
  },
  {
    id: "phone",
    label: "Telephone (8 kHz mono, 16-bit)",
    patch: { sampleRate: 8_000, channels: 1, bitDepth: 16 },
  },
];

/** Applies a preset. Dither choices a preset leaves out stay as they were. */
export function applyPreset(f: OutputFormat, id: string): OutputFormat {
  const p = SAMPLE_PRESETS.find((x) => x.id === id);
  return p ? { ...f, ...p.patch } : f;
}

/** A line about the dither when it is not the plain standard one, or null. */
export function ditherNote(f: OutputFormat): string | null {
  if (!f.dither) return "no dither";
  const bits: string[] = [];
  if (f.ditherKind !== "triangular") bits.push(`${f.ditherKind} dither`);
  if (f.ditherAmount !== 1) bits.push(`dither ×${f.ditherAmount}`);
  if (f.noiseShaping !== "none") bits.push(`${f.noiseShaping} noise shaping`);
  return bits.length > 0 ? bits.join(", ") : null;
}

/** Bit depths a format offers. MP3 has none. */
export function depthsFor(
  kind: FormatKind,
): { value: number; label: string }[] {
  if (kind === "mp3" || kind === "m4a" || kind === "aac") return [];
  const d = [
    { value: 16, label: "16-bit" },
    { value: 24, label: "24-bit" },
  ];
  return kind === "wav"
    ? [
        { value: 4, label: "4-bit ADPCM (smallest, lossy)" },
        { value: 8, label: "8-bit" },
        ...d,
        { value: 32, label: "32-bit float" },
      ]
    : d;
}

export const extensionOf = (kind: FormatKind): string => kind;

/** The format a file name saves as, or null if it isn't one we write. */
export function kindOfPath(path: string): FormatKind | null {
  const ext = path.slice(path.lastIndexOf(".") + 1).toLowerCase();
  if (ext === "wav" || ext === "wave") return "wav";
  if (ext === "flac") return "flac";
  if (ext === "mp3") return "mp3";
  if (ext === "m4a") return "m4a";
  if (ext === "aac") return "aac";
  return null;
}

/** Keeps only valid values, so a stale or hand-edited preference can't break an export. */
export function sanitizeFormat(raw: unknown): OutputFormat {
  const v = (raw && typeof raw === "object" ? raw : {}) as Partial<
    Record<keyof OutputFormat, unknown>
  >;
  const kind: FormatKind =
    v.kind === "flac" ||
    v.kind === "mp3" ||
    v.kind === "m4a" ||
    v.kind === "aac"
      ? v.kind
      : "wav";
  const depth =
    typeof v.bitDepth === "number" &&
    depthsFor(kind).some((d) => d.value === v.bitDepth)
      ? v.bitDepth
      : null;
  const rate =
    typeof v.sampleRate === "number" &&
    (OUTPUT_RATES as readonly number[]).includes(v.sampleRate)
      ? v.sampleRate
      : null;
  return {
    kind,
    bitDepth: depth,
    sampleRate: rate,
    channels: v.channels === 1 || v.channels === 2 ? v.channels : null,
    mp3Mode: v.mp3Mode === "vbr" ? "vbr" : "cbr",
    mp3Bitrate:
      typeof v.mp3Bitrate === "number" &&
      (MP3_BITRATES as readonly number[]).includes(v.mp3Bitrate)
        ? v.mp3Bitrate
        : 192,
    mp3Quality:
      typeof v.mp3Quality === "number"
        ? Math.min(9, Math.max(0, Math.round(v.mp3Quality)))
        : 2,
    aacBitrate:
      typeof v.aacBitrate === "number" &&
      (AAC_BITRATES as readonly number[]).includes(v.aacBitrate)
        ? v.aacBitrate
        : 128,
    dither: v.dither !== false,
    ditherKind:
      v.ditherKind === "rectangular" || v.ditherKind === "gaussian"
        ? v.ditherKind
        : "triangular",
    ditherAmount:
      typeof v.ditherAmount === "number" && Number.isFinite(v.ditherAmount)
        ? Math.min(2, Math.max(0.25, v.ditherAmount))
        : 1,
    noiseShaping:
      v.noiseShaping === "light" || v.noiseShaping === "strong"
        ? v.noiseShaping
        : "none",
    resampleQuality:
      v.resampleQuality === "draft" ||
      v.resampleQuality === "good" ||
      v.resampleQuality === "highest"
        ? v.resampleQuality
        : "best",
    mixLeft: unit(v.mixLeft),
    mixRight: unit(v.mixRight),
  };
}

const unit = (x: unknown) =>
  typeof x === "number" && Number.isFinite(x)
    ? Math.min(1, Math.max(0, x))
    : 0.5;

/** What the file will be, given the source it comes from. Matches the core's choices. */
export function describeFormat(
  f: OutputFormat,
  src: { sampleRate: number; channels: number; bitDepth: number },
): string {
  let rate = f.sampleRate ?? src.sampleRate;
  let ch = f.channels ?? src.channels;
  if (f.kind === "mp3") {
    ch = Math.min(ch, 2);
    if (rate > MP3_MAX_RATE) rate = MP3_MAX_RATE;
  }
  if (f.kind === "m4a" || f.kind === "aac") {
    ch = Math.min(ch, 2);
    rate = rate > 44_100 ? 48_000 : 44_100;
  }
  const chs = ch === 1 ? "mono" : ch === 2 ? "stereo" : `${ch} ch`;
  const hz = `${rate} Hz`;
  if (f.kind === "m4a") return `AAC ${f.aacBitrate} kbps, ${hz} ${chs}`;
  if (f.kind === "aac") return `AAC (raw) ${f.aacBitrate} kbps, ${hz} ${chs}`;
  if (f.kind === "mp3") {
    return f.mp3Mode === "cbr"
      ? `MP3 ${f.mp3Bitrate} kbps constant, ${hz} ${chs}`
      : `MP3 variable, quality ${f.mp3Quality}, ${hz} ${chs}`;
  }
  const want = f.bitDepth ?? src.bitDepth;
  // 4 and 8 are only ever chosen outright; a source's depth rounds up to 16, 24 or 32 like the core does.
  const picked =
    f.kind === "wav" && (f.bitDepth === 4 || f.bitDepth === 8)
      ? f.bitDepth
      : null;
  const bits =
    picked ??
    (f.kind === "flac"
      ? want <= 16
        ? 16
        : 24
      : want <= 16
        ? 16
        : want <= 24
          ? 24
          : 32);
  const depth =
    bits === 32
      ? "32-bit float"
      : bits === 4
        ? "4-bit IMA ADPCM"
        : `${bits}-bit`;
  return `${f.kind === "wav" ? "WAV" : "FLAC"} ${depth}, ${hz} ${chs}`;
}

/** Rough size in bytes of `seconds` of audio in this format. */
export function estimateBytes(
  f: OutputFormat,
  seconds: number,
  src: { sampleRate: number; channels: number; bitDepth: number },
): number {
  const rate = Math.min(
    f.sampleRate ?? src.sampleRate,
    f.kind === "mp3" ? MP3_MAX_RATE : Infinity,
  );
  const ch = f.channels ?? src.channels;
  if (f.kind === "m4a" || f.kind === "aac")
    return (f.aacBitrate * 1000 * seconds) / 8;
  if (f.kind === "mp3") {
    // Variable quality 0..9 runs roughly 245 down to 65 kbps.
    const kbps = f.mp3Mode === "cbr" ? f.mp3Bitrate : 245 - f.mp3Quality * 20;
    return (kbps * 1000 * seconds) / 8;
  }
  const want = f.bitDepth ?? src.bitDepth;
  if (f.kind === "wav" && f.bitDepth === 4) return rate * ch * 0.5 * seconds;
  if (f.kind === "wav" && f.bitDepth === 8) return rate * ch * seconds;
  const bytes =
    f.kind === "flac"
      ? (want <= 16 ? 2 : 3) * 0.6
      : want <= 16
        ? 2
        : want <= 24
          ? 3
          : 4;
  return rate * ch * bytes * seconds;
}

type Src = { sampleRate: number; channels: number; bitDepth: number };

/** The "sample type" box: rate, channels and (for lossless formats) bit depth, as the file will have them. */
export function describeSampleType(f: OutputFormat, src: Src): string {
  let rate = f.sampleRate ?? src.sampleRate;
  let ch = f.channels ?? src.channels;
  if (f.kind === "mp3") {
    ch = Math.min(ch, 2);
    if (rate > MP3_MAX_RATE) rate = MP3_MAX_RATE;
  }
  if (f.kind === "m4a" || f.kind === "aac") {
    ch = Math.min(ch, 2);
    rate = rate > 44_100 ? 48_000 : 44_100;
  }
  const chs = ch === 1 ? "mono" : ch === 2 ? "stereo" : `${ch} channels`;
  const parts = [`${rate} Hz`, chs];
  if (depthsFor(f.kind).length > 0) {
    const want = f.bitDepth ?? src.bitDepth;
    const picked =
      f.kind === "wav" && (f.bitDepth === 4 || f.bitDepth === 8)
        ? f.bitDepth
        : null;
    const bits =
      picked ??
      (f.kind === "flac"
        ? want <= 16
          ? 16
          : 24
        : want <= 16
          ? 16
          : want <= 24
            ? 24
            : 32);
    parts.push(
      bits === 32 ? "32-bit float" : bits === 4 ? "4-bit ADPCM" : `${bits}-bit`,
    );
    const note =
      bits < 32 && bits !== 4 && (f.bitDepth !== null || src.bitDepth > bits)
        ? ditherNote(f)
        : null;
    if (note) parts.push(note);
  }
  return parts.join(", ");
}

/** The "format settings" box: what the format itself lets you set, or null when it has nothing. */
export function describeSettings(f: OutputFormat): string | null {
  if (f.kind === "mp3")
    return f.mp3Mode === "cbr"
      ? `MP3, ${f.mp3Bitrate} kbps, constant`
      : `MP3, variable, quality ${f.mp3Quality}`;
  if (f.kind === "m4a") return `M4A, ${f.aacBitrate} kbps`;
  if (f.kind === "aac") return `Raw AAC, ${f.aacBitrate} kbps`;
  return null;
}

/** Settings remembered for each format, so an MP3 for the web and a WAV master each keep their own. */
export type FormatMemory = Partial<Record<FormatKind, OutputFormat>>;

/** Keeps only valid remembered formats. */
export function sanitizeMemory(raw: unknown): FormatMemory {
  const out: FormatMemory = {};
  if (!raw || typeof raw !== "object") return out;
  for (const kind of ["wav", "flac", "mp3", "m4a", "aac"] as const) {
    const v = (raw as Record<string, unknown>)[kind];
    if (v && typeof v === "object") out[kind] = { ...sanitizeFormat(v), kind };
  }
  return out;
}

/** Switching format: stash the one being left, bring back what the new one had (or a clean start). */
export function switchFormat(
  memory: FormatMemory,
  from: OutputFormat,
  to: FormatKind,
): { memory: FormatMemory; format: OutputFormat } {
  const next: FormatMemory = { ...memory, [from.kind]: { ...from } };
  return {
    memory: next,
    format: next[to] ? { ...next[to]! } : { ...DEFAULT_FORMAT, kind: to },
  };
}
