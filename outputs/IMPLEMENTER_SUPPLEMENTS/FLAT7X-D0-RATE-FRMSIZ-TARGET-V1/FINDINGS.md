---
status: washed-partial
topic: "Flat7x D0 1024 kbps frame-size target vector"
spec: "Approved Flat7x target and E-AC-3 frame-size representation"
confidence: high for target arithmetic; native selector join unbound
tags: [JOCForge, Flat7x, D0, bitrate, frmsiz]
audience: implementer
---

# Flat7x D0 rate and frame-size vector

The requested AU is two substreams: I0 at 640 kbps and D0 at 1024 kbps. At 48 kHz and six blocks per frame, one frame spans 32 ms. The D0 target frame length is 4,096 bytes; the separate I0 frame target is 2,560 bytes. The AU total is 6,656 bytes / 1,664 kbps, not one 1,664-kbps frame.

The washed E-AC-3 frame-size field is 11-bit `frmsiz` in 16-bit words minus one. Therefore the 4,096-byte D0 frame target maps to `frmsiz = 4096 / 2 - 1 = 2047 = 0x7ff`.

This is target arithmetic and an input-only vector for the Flat7x D0 profile. The native profile-rate caller, typed substream slot, and rate-to-writer generation join remain `UNBOUND`; this supplement does not claim that a DEE runtime selected 1024 kbps. Keep I0 and D0 as separate frames under one AU pair receipt.
