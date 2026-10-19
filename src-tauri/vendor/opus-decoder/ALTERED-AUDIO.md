# opus-decoder 0.1.1, patched

Upstream: https://crates.io/crates/opus-decoder (MIT OR Apache-2.0).

One change, in `src/celt/kiss_fft.rs`: `flat_fft_forward` computed its DFT
directly (O(n^2), a sine and cosine per term), which made a 7 minute mono
file take over a minute to decode. It now uses `rustfft` for the same
unscaled forward transform. Tests and benches were left out of this copy.
Drop this folder and the `[patch.crates-io]` entry once upstream is fast.
