# 7.1 I0/D0 AU transaction boundary

## Scope

This evidence-only supplement reuses four sealed Ghidra 12.2 static receipts and stops at the first missing typed join. It does not prove that the observed per-block or per-unit path is an I0 or D0 stream, or that the output transaction is an atomic I0+D0 AU.

## Bound static edges

- In one native JOC block path, FUN_1800942d0 calls FUN_1800a3710, then FUN_1800cb0d0, then FUN_1800a5810 for the same block ordinal and state base at +0x140e8. Field close/flush follows through FUN_180093360.
- The producer and packer use different pointer-table slots; pointer aliasing is UNBOUND.
- FUN_1800b7930 delivers per-unit ordinals in increasing order, but the ordinal is not typed as I0/D0 or joined to the user AU identity/source generation.
- FUN_1800e01e0 writes into a caller-provided in-memory per-unit buffer. A separate FUN_180175c80 to FUN_180189c80 path reaches fwrite with another buffer/length, without a typed join from the callback buffer.

## Open transaction boundary

No I0-specific builder/writer, D0-specific builder/writer, shared source generation, I0-then-D0 AU ordering, callback-buffer-to-fwrite join, or durable AU commit is bound in these receipts. Partial-I0 publication on a D0 failure and rollback behavior remain UNBOUND. Failure injection and runtime are NOT_RUN.

The user target I0 then D0 is a required contract, not a native observation in this static slice. Production readiness remains NO.
