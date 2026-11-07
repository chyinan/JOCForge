# LoRo center helper-result first-reader boundary

## Bound observation

The reused parser receipt maps XML key `lorocmixlev` to a four-byte helper-result slot `puVar16[0xf]` at callback object `+0x1a8`; the slot's semantic level value remains `UNBOUND`.

One selected post-parse function, `FUN_1403f3610`, was checked. Its `+0x1a8` displacement references are stack-relative locals, not callback-object field reads. Its separate pair of double getters at callback offsets `+0x28/+0x38` is not joined to the LoRo center slot. Thus the first typed reader/constructor of `puVar16[0xf]` remains `UNBOUND` on this edge.

## Disposition

No E-AC-3 BSI mapping, channel-matrix consumer, PCM effect, signaling-only classification, or intentional-ignore rule follows from this slice. This is function-scoped and not a global no-consumer claim.

## Exclusions

This washed summary contains only source package hashes and sanitized facts; no Ghidra script/log, binary, media, PCM, payload, key, or license is included. No DEE or A/B run occurred.
