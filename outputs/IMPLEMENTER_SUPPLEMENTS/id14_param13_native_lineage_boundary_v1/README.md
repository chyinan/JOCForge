# ID14 count and parameter-13 producer boundary

Ghidra static evidence closes two distinct paths in the hash-identified DDP Atmos image. The ID14 count is formed by subtracting two U32 fields from an input root, validated to 1..15, copied into nested JOC state, then emitted as a six-bit count-minus-one field. Parameter-13 is read from a separate property list, has its own configuration member/default, and reaches the additional-BSI U8.

Both paths are statically co-resident in one child worker, but use distinct input fields. No same-parent/generation capture joins their values. The upstream writer/owner of the two ID14-count root fields also remains UNBOUND. The sanitized R138 profile does not prove a keyless execution configuration, and a prior authorized setup stopped before native input processing, so no two-input capture was run.

This closes useful native producer and serializer edges without turning static co-residence into a same-run rule. It contains no source, decompilation, raw media, PCM, or private key.
