# Keyless reference core/bed audio comparison

## Result

The Wash-only comparison package reports that three E-AC-3 files—the protected Python/DEE reference stream, its sample-table-extracted reference, and the user-designated keyless compatible candidate—were decoded with the same FFmpeg 8.1 command to 48 kHz, six-channel `f32le` PCM. Each decode produced 6,523,392 frames. The in-memory PCM SHA-256 matched across all three (`84d298228269be6a44af7346893a93f3de773568d19ec7519bfd9f0562ba4c79`); per-channel max/mean/RMS error was zero for FL, FR, FC, LFE, SL, and SR.

This is a strong positive result for the FFmpeg E-AC-3 **core/bed decode path**. The decoder path skips additional substreams, so this does not establish JOC object or Atmos spatial-render equivalence. It also does not compare the current Rust attempt-10 output, which the user reports FFmpeg/Audition cannot import.

## Identity and limits

- Protected stream and reference-extracted stream: 17,395,712 bytes, same independently streamed hash `34d18a6fb0f477cbccd0b7d633a80b4191c0c535385ff66b8d9cbf7ff1440448`.
- Keyless compatible candidate: prior Wash-bound hash `383d7de4a57fd9bb32b8f871af1d2b5b0ad686a4b25e5fc6e51cd1be3adf6e86`; this comparison reused that hash and did not rehash the file.
- Decoder: FFmpeg 8.1, binary SHA-256 `865b4a599b589982cbfa875c32eec92abbe0d6e78120ab3d28701b59e95d8265`; ffprobe SHA-256 `d1c3740b2c2a23e3b8990afdf08492a9879fe1845ccdbc43e5cb2404ac01b2f1`.
- Decoded PCM was streamed through memory only, not saved to disk or included in the Wash package. No private HMAC/key/license was read.

Use this as a core/bed regression target for the keyless candidate. Do not report it as Atmos spatial equivalence or as a verified JOC-object comparison.
