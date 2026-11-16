# Altered Audio Editor 0.36.0

A better speech denoiser for hard noise.

## What's new

- **Speech Denoiser (High Quality).** A second denoiser in the Restoration group, built on a larger network (DeepFilterNet3, MIT or Apache-2.0). It handles harder noise than Speech Denoiser: keyboards, traffic and a faraway second voice. It has one control, Amount, starting at 100.

## Good to know

- It is built into the program. Nothing is downloaded and nothing goes over the network.
- It uses more of the processor than Speech Denoiser and adds about 40 ms of delay, which the session makes up for so tracks stay in step.
- Use it on speech only, one voice per track. On music it thins the sound.
- The installer is larger because the model is included.
