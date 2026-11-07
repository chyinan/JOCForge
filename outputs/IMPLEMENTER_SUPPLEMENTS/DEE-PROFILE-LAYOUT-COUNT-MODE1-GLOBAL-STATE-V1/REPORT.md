# DEE profile/layout count mode-1 global-state boundary

## Disposition

Wash verified the source package manifest and all eight payloads, its parent C-origin package, and the current Atmos DDP plugin image. The source package has no formal package seal; its manifest and payload hashes are the integrity basis. The global state value at runtime and any dynamic execution remain `NOT_RUN`.

## Descriptor-less open-config branch

For `open_config[0] == 1`, the static path validates `open_config[2]` with layout code at `[6]`: code 1 requires count 12, code 2 requires count 10, and other codes require count 16. In the type-2/count-10 branch, the constructor skips its known reset call to `FUN_1801ae590(0)` and reaches helper `FUN_1801ae5a0`.

The helper conditionally transforms code 2/count 10 to code 1/count 12 only if `DAT_1809d17c0 != 0`; it then revalidates the pair. The initialized `.data` image value of this global is 1. If no prior call reset it, the static projection yields `C=12`; if a previous non-type-2 call reset it to zero, the projection yields `C=10`. The final C is written into state count `count-1` plus one and passed to `FUN_1801acfe0`, which stores it at layout/profile `+0x08`.

The exact-address inventory found four direct reads and one direct writer (`FUN_1801ae590`). Its four inspected call sites pass zero on non-code-2 cases and skip that reset on code 2. This is not a complete alias/indirect-writer inventory, and prior call order/runtime state was not captured.

## Relation to ADM row count

This branch derives C from `open_config[2]`, with a conditional stateful transform. The inspected slice does not derive C directly from ADM physical row count N or canonical count M; the upstream identity of `open_config[2]` with either domain remains `UNBOUND`. The separate descriptor-bearing branch `open_config[0] == 0` remains C=16 from the previous verified package.

## Limits

- Static Ghidra only; runtime global value, call sequence, and selected branch are not captured.
- Direct exact-address references do not exclude aliases or indirect writes.
- No DEE run, input audio read, key/license read, or implementation modification.
- Normal and `python -O` checker status is agent-reported; Wash rehashed the package but did not execute the checker.

## Implementation scope

Do not hard-code layout count C=16 across all branches. Preserve this mode-1 branch's input count and conditional state transition as a separate layout domain; do not equate it with physical ADM row count until the typed upstream field is identified. The statically possible `C=10/12` outcomes are not runtime observations.

## Source package

`D:/Tools/Dolby_Encoding_Engine/work/research-audit/profile-layout-count-mode1-global-state-v1/wash-package/`; manifest SHA-256 `117c2bd64ebe9240901f05165821939a1014cc8704132fd9301bef319ab48b27`.
