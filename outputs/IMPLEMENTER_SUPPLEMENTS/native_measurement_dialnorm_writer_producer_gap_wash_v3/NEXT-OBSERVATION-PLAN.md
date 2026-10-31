# One-point field observation proposal

Status: **PROPOSED; NOT_RUN pending Wash coordination.**

If static Ghidra cannot identify the record member's constructor or first writer, use one authorized 38-channel ADM run with an isolated temporary output directory and the same Atmos DDP configuration.

At the metadata serializer entry, assign a run-local opaque token to the record base. At the Ghidra-mapped member-plus-eight helper call, require the same record token, `element_count == 1`, and `width == 5`; read exactly one two-byte word from the field pointer before the helper consumes it. Record the scalar, width, count, call status, and same-thread token only. Do not persist the pointer, read PCM/EC-3 payload, inspect license contents, or repeat the run if the call is not reached. Keep owner/generation **UNBOUND** unless a typed lineage token is independently observed.
