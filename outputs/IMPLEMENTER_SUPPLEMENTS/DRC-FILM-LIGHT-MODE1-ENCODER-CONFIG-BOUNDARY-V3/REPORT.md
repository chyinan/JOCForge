# Film Light DRC conditional encoder-config transfer

## Newly bound typed path

The DLL-local profile tree maps `film_light` to U32 `0`. On the `FUN_1801475f0` mode-1 branch, that same snapshot writes U32 0 at `snapshot+0xdc`, passes the snapshot to `FUN_1801754f0`, and its wrapper copies the qword at `snapshot+0xd8` into a raw Line enum object. `FUN_180178dc0 param_1[1]` reads the U32 at raw-object offset +4, which is the same snapshot `+0xdc` value. The sealed conversion maps raw U32 0 to DDP_COMPR code 2, installed at encoder config `+0x28`.

This is a direct static value chain and supersedes the earlier declaration-order-only candidate (`film_light` ordinal 1 → code 1) **for the mode-1 path**. It does not establish that the target job selected mode 1 or that XML submission reaches this descriptor.

## Audio and job boundary

DDP_COMPR is an encoder configuration control, but no DEE job was run and no output PCM was compared. The job derivation remains `NOT_STARTED`; runtime mode selection and property submission are `UNOBSERVED`. Do not encode `film_light → code 2` as unconditional until the selected path is observed. This package also does not characterize the compression curve or audible effect.

## Source verification

Wash rechecked the corrected R5 package manifest/seal/unique payload hashes, target image identity, and five cited Ghidra logs. The parent DRC conversion package manifest and value-map hashes also match. No source code, raw Ghidra logs, binary, PCM, payload, key, or license is included.
