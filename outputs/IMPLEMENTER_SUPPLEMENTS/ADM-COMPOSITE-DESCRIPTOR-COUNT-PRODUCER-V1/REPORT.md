# Composite descriptor B+0x0A count producer

## Bound typed store

`FUN_18054fa70` writes `*(ushort *)(param_1+0x1A)`. With the composite owner `T` and embedded descriptor `B=T+0x10`, the destination is the same `B+0x0A` U16 field consumed by the previously bound row-copy path.

## Mutually exclusive static producer rules

- **Ordinary mode:** the selected source adapter `J` exposes a U16 count getter (vtable slot `+0x80`) that reads its retained opened-source descriptor `K+0x0A`; the writer copies that count to `B+0x0A` without an additional transform. This preserves the K count only if the selected J wraps that same opened K and generation.
- **Extended mode:** for each source adapter, the writer counts six-byte rows whose tag byte at row offset `+2` is `255`, takes the maximum across adapters, adds ten prefix rows, and stores the resulting U16 to `B+0x0A`. This is a transformed object-row count, not a direct physical CHNA count.

Under the prior metadata-only examples, 26 and 28 object rows plus ten prefix rows project to 36 and 38. Those numeric matches do not select the extended branch and do not establish the actual native B count. The selected composite mode, source adapter J, opened K identity/generation, and runtime B value remain `NOT_CAPTURED`/`UNBOUND`.

The precise remaining edge is the selected source adapter and composite mode for one concrete invocation. No Ghidra query or DEE run was performed in this source receipt; no PCM, payload, key, or license was read.

## Provenance

Upstream Wash-only manifest SHA-256: `6a9456349c5414bdedd03d342b787e544e8d82566333344adcd59ade26c5c2b9`. Its normal and `python -O` verifier passed. The Wash package includes the upstream manifest, source-link receipt, and checker-result record, but no Ghidra scripts or logs.
