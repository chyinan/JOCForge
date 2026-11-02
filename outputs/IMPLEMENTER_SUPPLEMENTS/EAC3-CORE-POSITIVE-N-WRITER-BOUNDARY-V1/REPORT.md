# E-AC-3 core positive U7 group-count writer boundary

## Bounded Ghidra result

The E-AC-3 core frame writer passes the same `state+0x140f0` base to
`FUN_14032d580` (D/N consumer) and `FUN_14036f8f0` (U4/U7 syntax writer).
`N` is read as `u16(state+0x141c8+2*j)` and `D` as
`i16(state+0x1418c+10*block+2*lane)`, where `j=5*block+lane`.

The newly checked direct callees do not assign a positive N. The only typed
write candidate in this slice is reset `FUN_14032d9b0`, which clears both D
and N. A nearby positive U16 write by `FUN_14032b520` targets
`state+0x14150+2*j`, 0x3c bytes before D and 0x78 before N; no copy from this
field into D or N was found in the bounded pass.

## Candidate retained, not promoted to native fact

`N=(D+2)//3` remains a confidence-0.78 implementation inference for a
minimum triplet count with zero-to-two tail values. The input-only vector
`D=4 -> N=2, tail=2` is arithmetic only. The positive N producer, runtime
owner/generation join, and broader candidate-callee closure remain open.
This is E-AC-3 core evidence only and cannot be transferred to the separate
Atmos JOC `FUN_180...` functions.

## Scope

Ghidra 12.2 static only. No DEE run, media/PCM/payload/key read, or
implementation contact occurred in the reverse slice. No binary or
decompilation is included. The delivered source package had a hash-bound
`MANIFEST.json` and passing checker, but no separate `package_seal.json`.
