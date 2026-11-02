# Sample 6 process-frame input to core row boundary

Recipient: implementation evidence supplement  
Disposition: bounded static path; no runtime F32 row values or algorithm change established.

## Verified path

Wash independently verified the upstream R2 manifest, seal, all five payload hashes/sizes, and `CHECK.py` in normal Python and with `python -O`.

For the hash-bound `dee_ddp_encoder.exe` image (SHA-256 `0cf23520799f52323b0994e5fcc16924da7b027a5b83a0cff3b4f06792c18532`), the process API receives a frame view in `param3`. The checked accessors read sample count (U64 at `+0x28`, accepting `0x600` samples or zero for flush), channel count (U64 at `+0x20`), channel pointer-list address (U64 at `+0x08`), a flag at `+0x38`, and dialnorm at `+0x60`. `FUN_14031ef60` converts/copies source channels into a separate frame/work buffer.

The dispatcher reaches method-table slot `+0x28` at `0x1403160dc`, resolving to `FUN_140350d00`. That method loads the state pointer from `channel_instance+0xdf8` and enters `FUN_140317600`. A separate setdata path uses a descriptor at `channel_instance+0x4280` or channel-owned fields around `+0x2f28/+0x2f38`; the checked path finds no typed copy from those fields or the frame/work buffer into the inline row `[state+0x23e34,state+0x23eac)` before the core reader.

This is a bounded in-image static result, not a whole-process absence claim. Runtime values and external/unresolved writers remain `UNBOUND`. Constructor zeros are not runtime row values. The package establishes no PCM/LFE mismatch cause.

## Next concrete edge

Trace incoming references at `0x14076febc`, `0x1405537f8`, `0x14076fec8`, `0x140703210`, and `0x1405537f0` into `FUN_140214e80` / `FUN_140214ef0`, then continue toward a typed producer/copy into the same `channel_instance+0xdf8` state row. No such bridge is bound yet.

The symbolic input-only vector is `NOT_RUN`; it contains no media or PCM. No DEE run or runtime capture occurred. No private key/license material or raw payload was accessed. This supplement contains no Ghidra source or decompilation text.

## Upstream seal

- Package: `sample6-matched-drc-frame-input-row-writer-boundary-wash-v1`
- MANIFEST.json SHA-256: `c29cf85d37e545b706fa101102080607dbf7440c2ec61f822b1bc5847128b25a`
- package_seal.json SHA-256: `d84b30136756b47af71ec620171958908e4140539f4e012361729397e1e172d7`
- All five manifest payload hashes and sizes matched; upstream normal and optimized checkers passed.
