# ProcessFrame M to E-AC-3 writer boundary

## Configurable capacity gate and shared static handler

The hash-bound Ghidra 12.2 receipt traces a ProcessFrame descriptor's per-group U32 count M into `FUN_140315db0`, where it is compared with configurable capacity `root+0xC140`; if capacity is below M, the function returns `0x1D`. The capacity comes through selected configuration/profile key 2, but the active value and selected runtime table index remain `UNBOUND`.

Static table entries 0 and 1 share vtable `0x140610510` and handler `FUN_140350D00`. The handler passes caller-provided scratch storage derived from `root+0x38` to child encoder states. Its bounded layout uses `selector*0x2000` offsets, child views spaced by `0x1000` with size `0x800`; the scratch allocation owner is not identified and no M-proportional allocation is bound in these functions.

## Writer-local guards are separate domains

The handler reaches E-AC-3 stuffer `FUN_14033B5F0`. Its explicit checks reject `frame_count > 6` at state `+0x182` with `0x12E` and `block_count > 6` at `+0x180` with `0x135`. The inspected path does not type or join ProcessFrame M to either stuffer counter. These six-count limits are not channel-row limits.

The reused ProcessFrame receipt also reports a 16-qword local pointer staging array without a `count <= 16` guard before a multi-entry copy. If actual M exceeds 16 on that path, this is a potential out-of-bounds boundary; runtime entry count, selected owner/generation, and memory effect are not observed. It is not a confirmed runtime fault or a supported maximum.

## 36/38 and exact open boundary

Symbolic input-only vectors check hypothetical M=36/38 against matching or one-short configured capacities. They show only how the static gate would branch if reached; they do not join ADM physical N=36/38 to ProcessFrame M. The actual N→M conversion, runtime table selection, active capacity, and M→stuffer frame/block counters remain `UNBOUND`.

Therefore this receipt confirms one configurable M capacity gate and a shared static handler for table entries 0/1. It does not prove arbitrary-track end-to-end support. Keep the N, ProcessFrame M, capacity C, stuffer frame count, and stuffer block count as separate domains.

## Provenance

Upstream manifest SHA-256: `ea654ef938b0dd5020ddef2cfaf93bcdc78e80b466a87877c5662af49fd21ae3`; upstream package-seal SHA-256: `2a09daa833fe8076d8b668737598825ad5ee765e7cfd4927d3f383162160220b`. Upstream normal and `python -O` checks passed; source identities bind the binary and Ghidra function hashes. No DEE run or media/PCM/payload/key/license read occurred. The upstream package's scripts/logs are not copied here.
