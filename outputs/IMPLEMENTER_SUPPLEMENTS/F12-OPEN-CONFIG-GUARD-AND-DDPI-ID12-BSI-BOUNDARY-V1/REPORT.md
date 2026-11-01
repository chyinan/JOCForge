# F12 open-config guard and DDPi ID12/BSI boundary

## Result

The open-config scalar helper's byte `+0x10a` is zero-initialized on the resolved open path and then read for a mismatch comparison. The direct-call closure has no writer to that byte; two same-offset writers found elsewhere are not reachable in that closure and are not object-joined to the DDPi DialNorm stream field.

Separately, DDPi ID `0x12` is a 32-bit value at per-stream `+0x9c`, validated in `[1,31]`. The setter and frame process share a stream-pointer table, but use different count fields and have no proven same-index/generation join. The observed frame helper copies DDPi IDs `0x15` and `0x17` to a nested context; the nested `+0x9c` consumer is the ID `0x17` copy, not stream-base ID `0x12`.

No direct ID `0x12` read was found in the bounded 533-function frame-output closure; 28 indirect/unresolved calls remain. The frozen F12 BSI contract is integer U5 at bits 45–49, but the ID12-to-BSI typed consumer and same-generation owner join remain `UNBOUND`.

## Scope and limits

The package contains a Ghidra-only static trace and input-only structural checks. It does not claim that absence across unresolved indirect calls proves global non-use, and it does not explain the attempt-10 SHA mismatch. No DEE run, PCM, payload, bitstream, key, or license content was read.
