# DI Bool helper post-delegate field boundary

## Bound relation

The selected `FUN_1801aea50` initializer receives `param_2` from outer subcontext `+0x15c`. It first passes that pointer to `FUN_1801ae690`; the callee body is not included. On the later success path, if the source and helper-object addresses differ, the function copies 16 bytes from `param_2` to helper object `+0x10`, so the post-delegate DWord at `param_2+8` maps to helper `+0x18`. If the addresses match, helper `+0x18` aliases `param_2+8`.

This is a static post-delegate copy-or-alias fact only. It does not establish that the original outer Bool at subcontext `+0x164` survives `FUN_1801ae690` unchanged.

## First break

No typed owner/generation edge connects the helper object's `+0x18`, this initializer return, or the outer Bool to the per-channel status table at `+0x168 + 8*channel`, status slots `+0x128`, or accumulated gate word `+0x1350`. The Bool-to-algorithm input and DI effect remain `UNBOUND/NOT_RUN`.

## Exclusions

This package contains sanitized findings and source hashes only. No Ghidra source/log, binary, media, PCM, payload, key, or license content is included; no DEE/A-B/runtime was run.
