# Individual-slot helper first-write boundary

Wash independently verified the updated source package manifest and seal, all sealed payload hashes, the parent first-break receipt, and the Ghidra 12.2 target identity. The supplement contains sanitized findings only; it excludes raw query logs and Ghidra scripts.

In `FUN_1802809a0` (RVA `0x2809a0..0x2809d8`), the first tracked RCX/RDX-based field write is at `0x2809a8`: `qword ptr [RDX+0x18] = 0xF`. Before that store, the v2 incremental query records no RCX/RDX field read, conditional guard, or call. From the sealed caller receipt, RDX points to a stack-local output object in `FUN_1804d8b90`; RCX is copied from the caller's entry RDX, whose semantic identity is unbound. The output field's type and meaning are also unbound.

This is not a write to a proven individual-record `+0x4c` or UID-derived slot. The slot producer and pre-assignment guard remain `UNBOUND`; no relation to a typed descriptor or K_count is established. The query stopped at the first store; post-store body is `NOT_SEMANTICALLY_REVIEWED`. An initial exploratory query predecoded 17 instructions but emitted and analyzed only the four-instruction first-break prefix; the v2 query stopped incrementally. Runtime remains `NOT_RUN`.
