# Individual-record `+0x4c` slot producer first break

Wash independently verified the package's sealed evidence manifest, payload inventory, target module hash, and normal/`python -O` checkers. The forwarded supplement contains only sanitized findings; it excludes Ghidra scripts/logs and raw traces.

On the selected zero-byte U-open branch (`0x408160..0x40a585`), the bounded scan reports no explicit read or store of the individual-record `+0x4c` field. The selected call to `FUN_1804d8b90` at `0x4081d7` also has no explicit `+0x4c` access in its 126-byte body. Its next call is `FUN_1802809a0` at `0x4d8bbc`, with `RCX` copied from the helper's entry `RDX` and `RDX=&local output`; the callee's type and relation to the UID-derived slot are not established.

After that call, `[RAX+0x18]` is compared with `0x10`; this is not established as an individual-slot or `K_count` guard. The slot producer and pre-assignment guard remain `UNBOUND`. This is a selected-branch/one-helper boundary only, not a global writer scan and not evidence of runtime out-of-bounds behavior. Runtime remains `NOT_RUN`.
