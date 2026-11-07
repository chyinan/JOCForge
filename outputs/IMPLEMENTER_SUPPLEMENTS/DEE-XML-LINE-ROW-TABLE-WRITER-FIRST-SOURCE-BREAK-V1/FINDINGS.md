# Line DRC row-table writer first source break

## Bounded writer edge

The sealed Ghidra 12.2 static slice follows one direct writer: `FUN_1801535b0` calls `FUN_18013b870` at `0x180153636`, with destination `param_2` (the snapshot row-range field at `+0`) and source descriptor `param_3+1`. The helper treats both as vector descriptors, computes source count as `(end - begin) >> 5`, and reserves/copies records using a 0x20-byte element stride.

The writer is generic: it receives neither the snapshot profile U32 at `+0xA0` nor a core U8 selector as a value/address, and it does not interpret the 32-byte records' fields. The enclosing row table and profile U32 share a same-snapshot base, but row content/value lineage is not established.

## First break

The first unresolved input is the opaque source range `param_3+1`; no typed field mapping ties its records to the Line profile `+0xA0` value or the core curve-selector. The mode-to-curve/dynrng rule remains `UNBOUND/NOT_RUN`. This closes only one writer hop and makes no global no-consumer claim.

## Exclusions

This supplement contains sanitized findings and source hashes only, not the query script, trace excerpts, disassembly/decompilation, binary, media, PCM, payload, key, or license.
