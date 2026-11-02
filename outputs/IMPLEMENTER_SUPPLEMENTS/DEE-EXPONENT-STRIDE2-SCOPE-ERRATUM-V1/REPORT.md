# Erratum: E-AC-3 core selector-2 vector is not a JOC oracle

This erratum supersedes the cross-scope comparison in
`DEE-EXPONENT-STRIDE2-EMULATOR-COMPARISON-V1`. The R2 EmulatorHelper target
`dee_ddp_encoder.exe!FUN_14032d580` is an E-AC-3 core exponent-code producer.
The washed core producer package joins it to the adjacent core syntax writer
`FUN_14036f8f0` through the same `state+0x140f0` base. R2's selector-2
synthetic result `[76,12]` remains valid for that core function's supplied
state and row.

The JOC candidate is a separate Atmos DLL path: its local transform and
writer are `FUN_1800a4d70` and `FUN_18008aea0` in
`dee_audio_filter_ddp_atmos.dll`. Its own washed evidence maps the per-block
selector table to strides `[0,1,2,4]`, but the existing JOC EmulatorHelper
vector exercises selector 1/stride 1 only. No hash-bound JOC selector-2
input/output vector has been evaluated here.

Therefore the prior calculation that fed the E-AC-3 core row into the Rust
JOC candidate and labeled it a JOC mismatch is withdrawn. Keep the core
observation and the JOC candidate in separate scopes. The JOC candidate's
selector-2 behavior must be compared with a vector produced from the JOC
Atmos functions themselves; the E-AC-3 core values cannot stand in for it.

## Next minimal JOC check

Use the JOC `FUN_1800a4d70` raw row/seed, block-local selector 2, and the
same block's `total_slots`; observe selected inputs, delta/tail vector and
U7 writer output through `FUN_18008aea0`. Bind the exact row shape and caller
fields. Do not infer a JOC runtime selector from an E-AC-3 core table or
vector. R9 is separately tracing the selected JOC profile row-2 producer.

Ghidra evidence is static or synthetic, not a DEE run. No binaries, source
decompilation, media, PCM, payload, private key or license material is in this
supplement. No output parity or JOC runtime profile claim is made.
