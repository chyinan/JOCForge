# Human Cat DRC plugin-local row producer

## Washed producer rule

In the Atmos plugin, a per-source F32 input is selected through a mapped strided source table. Each block copies 256 F32 words into a scratch buffer. A filter helper produces another 256-F32 vector; the producer clamps each value to image bounds, accumulates 256 squared values, multiplies by an image constant, and stores one F32 value in a plugin-local six-by-five row:

```text
plugin_root + 0x21e9c + 0x2170 + 0x14*block + 4*row_slot
```

The same plugin invocation later passes the same plugin-local context and block index to a reader. This binds the plugin-local producer to its plugin-local consumer statically. The row's semantic label remains unknown; do not call it RMS, peak, or release from shape or naming alone.

## Core owner boundary

The E-AC-3 core separately owns an inline six-by-five row at `state+0x21cc4`. No typed host/API copy or owner edge connects it to the plugin-local row at `plugin_root+0x21e9c`. Matching relative offsets and similar row layout do not establish identity. The core row's own generation-time writer remains unbound.

This is a concrete plugin-side DRC analysis statistic, not yet a usable core input contract. Compare it against the current Rust DRC scheduler only after an explicit selected-path/owner bridge is found. No PCM, encoded payload, private key, license material, native source code, or disassembly is included.
