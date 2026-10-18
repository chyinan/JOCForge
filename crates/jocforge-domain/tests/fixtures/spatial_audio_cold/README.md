# spatial_audio cold-state verification fixtures

These sanitized vectors come from the verified `f04-spatial-audio-cold-state.v1` supplement.
The root and six temporal binaries are post-construction comparison targets only. Tests must
construct the typed F04 state and run the project encoder before reading/comparing KAT bytes.

Supplement manifest SHA-256: `77ad0562fcd4efa1e8fe37cc1dc7219570b49b9b332cca1f1273ccda2e2cef9c`.
Expected root: 813,733 bytes; SHA-256 `35cfac927bbdca64f28954844938fe2936531f59f049dfca911c44d3a38b0b5e`.

No `research_basis` material is included. The fixture contains no secrets or proprietary audio.
