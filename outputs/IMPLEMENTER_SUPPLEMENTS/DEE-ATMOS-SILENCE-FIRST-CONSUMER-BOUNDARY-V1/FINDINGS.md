# Atmos DDP silence-count first consumer

This is an addendum to the sealed silence-duration sample rule. It does not
repeat the XML parsing or 48 kHz conversion analysis.

## One typed consumer edge

`FUN_180145380` puts each parsed `longlong` prepend count into an eight-byte
per-source vector at record byte offset `+0x38`; it puts append into a
separate eight-byte vector at `+0x50`. `FUN_18014e980` appends one qword to
each vector. `FUN_1801535b0` copies those vectors to encode-context offsets
`+0x18` (prepend) and `+0x60` (append), using a vector-copy helper with an
eight-byte element stride.

In `FUN_180175c80`, the first typed downstream consumer call is
`FUN_1801844c0` at RVA `0x175e10`. The caller passes the prepend vector from
context `+0x18` in argument 3 (`R8`) and append vector from context `+0x60`
as argument 6 on the stack. The callsite window confirms the register and
stack argument setup. The callee's six-argument prototype treats argument 2
as a vector of 32-byte per-source rows and arguments 3 through 6 as parallel
vectors of eight-byte values; it checks that their element counts match.

`FUN_1801844c0` then zips these values into new 0x40-byte composite rows.
The prepend vector element is copied verbatim to row `+0x20`; the append
vector element is copied verbatim to row `+0x38`. This is the first typed
consumer edge established here. The values are passed through as per-source
duration/count metadata; the function does not add samples to a PCM buffer
or arithmetically shift a range/PTS in this body.

## Boundary

No downstream consumer of the composite row was inspected. Whether later
code zero-fills PCM, adjusts source/sample range or PTS, or uses these fields
for another duration operation remains `UNBOUND`. No DEE job was run; no
PCM, media, payload, key, or license data was read. Ghidra 12.2 queried the
existing project in read-only/no-analysis mode.
