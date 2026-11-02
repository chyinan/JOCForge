# DEE JOC bit-writer to AU byte-publish boundary

## Bound static data path

Ghidra 12.2 evidence on the hash-bound Atmos module joins the per-unit bit
writer to its repacked destination and first byte-publish callback. Each
output-unit iteration resets cursor/accumulator state and selects its source
word pointer. The same writer context is passed through the field writer and
field-finalization helper. If residual bits remain, the core writes one
16-bit accumulator word; the separately recorded valid bit span ends before
padding, so the repacker selects valid bits and forms its own word-padded
destination.

For unit ordinal `k`, the repack bridge copies the source at
`outer+0xb30+0x10*k` into destination `outer+0xb90+0x10*k` and writes a
16-bit word count at `outer+0xb9a+0x10*k`. The following per-unit dispatcher
submits these destinations in increasing ordinal order. Its selected
per-index stream object is processing metadata; it is not the bit-emission
callback target.

The first concrete byte-publish edge in this direct chain is the callback
`FUN_1800b4eb0` → `FUN_1800e01e0`, which writes bits into the caller-supplied
per-unit memory buffer and updates its pointer/capacity. The direct chain
does not reach a file/container or OS I/O writer. This locates an in-memory
publish boundary; it does not show how a higher layer appends that buffer to
the already observed full output file.

## Disposition

This closes a useful serializer-to-buffer edge for the JOC path. It does not
change the transform rule, prove full-file publication semantics, or indicate
that the existing parseable full-length JOCForge output is blocked. Keep the
application-file append stage outside this static slice open until its caller
is traced.

## Evidence limits

Static Ghidra evidence only. No new DEE run, media/PCM/payload read, key or
license access, or implementation contact occurred in the reverse slice.
This supplement contains no binary, decompilation, raw disassembly, media,
PCM, encoded payload, key, or license material. The source manifest lists the
hash-bound analysis artifacts; a separate source `package_seal.json` was not
present in the delivered directory.
