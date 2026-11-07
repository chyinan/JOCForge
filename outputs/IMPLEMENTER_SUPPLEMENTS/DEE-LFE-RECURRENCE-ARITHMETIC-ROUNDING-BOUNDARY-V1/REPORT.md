# LFE recurrence arithmetic and rounding boundary

## Static instruction result

In `FUN_14036d9e0` inline recurrence, the inspected packed-single arithmetic is separate `MULPS` operations followed by `ADDPS`. In `FUN_14036d7c0`, the inspected low- and high-bank arithmetic also uses separate `MULPS` and `ADDPS` instructions. These are packed binary32 operations with distinct multiply and add rounding points; neither body contains a fused multiply-add instruction in its arithmetic sequence.

Neither function body reads or writes MXCSR, changes rounding control, or contains explicit `ROUND*`/rounding-select conversion instructions. The active MXCSR control state at function entry remains `UNBOUND`.

The source package independently cross-checked all 105 instruction encodings in the two bodies against the hash-bound target executable. Its historical exporter script hash differs from the current file at that path; the supplement relies on the pinned Listing artifact plus the 105/105 target-image byte comparison, not on the current script.

## Limits

This is function-body evidence only. It does not establish the caller's active MXCSR, sample-specific LFE layout/generation, numeric parity, or the cause of the reported correlation. No DEE run or PCM/payload/key/license access occurred.

## Provenance

Source package `dee_lfe_recurrence_instruction_arithmetic_wash_v1` manifest SHA-256 `01617db22f7759937904489dc6f2087fcc9ee0330043270673b850bbb0b33624`, package-seal SHA-256 `149f5e8bd2cb806f6221f39dccf7fa9ce54099e36452c0ae7ed0edff950d7de2`; source checker passed normally and under `python -O`.
