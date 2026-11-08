# RF music_standard raw-code and selector boundary

Reference-only Wash supplement. The reused RF token map assigns `music_standard` a raw 32-bit candidate value `3), which is written to per-frame `+0xa4). No DRC meaning is assigned to this code.

A separate 32-bit field `Priv+0xfc8` is initialized to zero by a constructor; a parseWav path can store 2 when channels < 10, otherwise 1. The encode-side path reads that selector into separate per-frame fields `+0x9c) and `+0x12c), and tests selector values 1 and 2. The inspected path does not transfer the RF raw code at `+0xa4) into `Priv+0xfc8).

The RF code-to-selector relationship, same-run owner/generation, BSI writer, and audio effect remain `UNBOUND/NOT_RUN). This is a bounded static edge, not a global no-reader claim. `IMPLEMENTATION_READY=NO`.
