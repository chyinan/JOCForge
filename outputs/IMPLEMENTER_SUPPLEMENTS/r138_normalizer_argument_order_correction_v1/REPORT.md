# Wash review: R138 normalizer argument order

This Ghidra-only correction supersedes the previous reversed R8/R9 note.

For the hash-bound host build, the source-time normalizer arguments are RCX=host source-property map, RDX=timecode_frame_rate configuration, R8=FFOA configuration, R9=offset configuration. The helper branches on R8 for FFOA and R9 for offset; the callsite disassembly supplies R9 even though the decompiler omitted that argument. Constructor defaults are not_indicated / auto / auto for rate / FFOA / offset.

This is a static argument mapping only. The selected internal run method for R138, runtime map owner/generation, resolved FPS, and host-map to DDP setter join remain NOT_CAPTURED/UNBOUND. No native job or audio payload was accessed in this package.
