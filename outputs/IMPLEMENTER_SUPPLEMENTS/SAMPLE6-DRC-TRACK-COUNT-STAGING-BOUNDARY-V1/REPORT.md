# Sample 6 DRC track-count staging boundary

Recipient: implementation evidence supplement  
Disposition: static safety boundary only; it does not establish that ADM track count equals the native process-frame count.

## Bound static path

Wash independently verified the upstream R2 manifest and seal and recomputed all five payload hashes and sizes. Its `CHECK.py` passed in normal Python and with `python -O`.

In the hash-bound `dee_ddp_encoder.exe` image, `FUN_140213470` reads an unsigned 64-bit count at `param3+0x20` and an unsigned 64-bit channel pointer-list address at `param3+0x08`. On the multi-entry path it copies pointer entries into `local_d8[16]` before the later conversion/overflow check. No `count <= 16` guard was found before that copy in the checked function. The count is then narrowed to U32 in a one-group descriptor passed to `FUN_140315db0`.

`FUN_140315db0` compares the per-group count against `root+0xc140` and returns `0x1d` when the count exceeds that configured capacity. `FUN_14031fae0` obtains the capacity through configuration key 2 (`FUN_140321500`), from a custom `(i32 key, i32 value)` pair or the selected profile's built-in table. The active profile and capacity value are not observed. This gate runs after the wrapper's pointer-list staging.

There is no fixed 36/38 comparison in this checked path. The symbolic vectors show only conditional outcomes of the later capacity gate; they are not native observations. If a process-frame count of 36 or 38 actually reaches the 16-slot staging loop, it exceeds that local extent. However, ADM-declared track count → process-frame count has not been joined, and runtime overwrite behavior was not observed. Do not claim that the authorized 36-track ADM triggers this path without that typed join.

## Implementation implication

Do not change a track-count field to 36/38 and assume it is the same dimension as the native frame-view count. Bind the actual ADM-row-to-frame-view transformation first. In Rust, preserve the variable ADM row count and use safely sized owned storage; validate count versus allocation capacity before any copy. A conditional downstream capacity rule does not make an earlier fixed 16-entry staging safe.

## Source seal and limits

- Upstream package: `sample6-matched-drc-track-count-admission-boundary-wash-v1`
- MANIFEST SHA-256: `613d934c86bb9a3bf21049fed7a5bc92f6650d11fadddcf79167533adef9b066`
- package_seal.json SHA-256: `d76e3cf9015f6d6aeedb80b998d8d97084c64cdc880c7e5fbe41e6fe8fd705cd`
- Five manifest payload hashes and sizes matched; upstream normal and optimized checkers passed.
- This is Ghidra 12.2 static evidence plus a symbolic input-only vector. No DEE run, runtime capture, PCM/payload read, or private-key/license access occurred.
