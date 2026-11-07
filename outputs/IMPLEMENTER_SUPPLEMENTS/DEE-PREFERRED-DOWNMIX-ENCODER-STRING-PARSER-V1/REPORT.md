# DEE preferred-downmix encoder-string parser boundary

## Bound static path

The DdpjocEncoderApp option schema registers encoder as a Boost typed string at this+0x360, with short alias e. The runner transfers that string into its config; only an empty stored string takes the static ddpjoc: fallback. A later parser reads the same runner config, splits colon-delimited fields, splits each field at the first equals sign, creates key/value string pairs, installs them in a map, and selects the encoder section.

The reused typed-reader receipt binds the map lookup path to preferred_downmix_mode and its four-value enum parser. This closes the static option schema→composite parser→map→typed reader path.

## Runtime and audio boundary

No actual child argv or selected parser branch was captured. The ddpjoc: fallback is only an empty encoder-string fallback; it does not establish the effective preferred_downmix_mode value. The first audio/BSI/frame-writer consumer remains UNBOUND. This package is static parser behavior, not runtime acceptance or an audio result.

The upstream manifest SHA-256 is 1fffe96f847866e3e26a891685d9b005c5e2fa1bf9eebee911b8c31c623852f1. Its checker passed normally and under python -O. The target image hash is from a parent project record; the installed image was not rehashed. Raw Ghidra material, source, media, PCM, payload, key, and license are excluded.
