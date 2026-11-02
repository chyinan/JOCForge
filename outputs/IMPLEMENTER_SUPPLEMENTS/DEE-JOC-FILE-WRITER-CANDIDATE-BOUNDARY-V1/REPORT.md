# DEE JOC per-unit buffer and file-writer candidate boundary

The washed JOC output path binds per-unit records through a caller-provided
buffer. `FUN_1800b7930` supplies a repacked input at record `+0xb90` with byte
length `U16(+0xb9a)*2`, and a distinct caller destination pointer
`plVar14[-0x282]`. That destination passes through `FUN_180077ac0` and
`FUN_1800b53b0`; the callback `{FUN_1800b4eb0,0}` reaches
`FUN_1800e01e0`, which writes to the supplied memory buffer and returns an
output count.

A separate file-writer candidate exists in the same module:
`FUN_180189c70` thunks to `FUN_180189c80`, which calls `fwrite(param_2,1,
*param_3,FILE*)` for nonzero length. Its direct callers in this analysis are
`FUN_1801746d0`, `FUN_180175500`, and `FUN_180175c80`. The Wash source did
not join the per-unit callback buffer to `fwrite`'s `param_2`/length, nor
join the `FILE*` to the produced `.ec3` output. The file writer is therefore
a candidate boundary, not a proven consumer of the JOC per-unit buffer.

The existing host `OutputWrapper::combineSources` evidence describes input
source registration and does not supply the missing output-buffer edge. This
does not contradict the independently verified parseable JOCForge full file;
it says only that this particular DEE static per-unit path has not been
joined to a file write.

## Scope

Ghidra 12.2 static only. No DEE run, media/PCM/payload/key access, or
implementation contact occurred. No binary, raw decompilation, or source
code is included in this Wash supplement. The next smallest edge is the
caller/adapter that transfers the per-unit buffer and count into a confirmed
`.ec3` writer.
