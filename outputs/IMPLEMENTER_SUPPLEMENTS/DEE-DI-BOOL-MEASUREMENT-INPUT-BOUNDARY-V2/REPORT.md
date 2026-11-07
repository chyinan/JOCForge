# Dialogue Intelligence Bool measurement result boundary, V2

## Bound value-to-input path

The exact-key candidate remains a `SafeType<bool>` at `Priv+0x290`; the typed reader returns the stored Bool byte, and `FUN_1801535b0` writes it into measurement setup `+0x94`. The same setup structure is forwarded through the measurement engine and first per-record consumer.

## Next result edge

`FUN_1801764e0` returns a reader pointer through its output parameter. `FUN_1801769a0` stores that pointer as `local_130` and calls the reader's vtable slot `+0x10` to check status. The reader-factory branch is controlled by setup fields `+0x9c` and `+0x140`. This result path does not bind setup byte `+0x94` to the returned reader object or status value. The final `Priv+0xf6c` measurement effect therefore remains `UNBOUND`.

The upstream exact-key candidate remains conditional; runtime selection of `dialogue_intelligence` and owner/generation were not captured. No DEE/UI runtime or audio data access occurred.

## Provenance

Updated source package manifest SHA-256 `2f5e2b9c2c54f38eefae07dfae60e93f743c336a4f0927ff03822a12abae758b`, package-seal SHA-256 `5b62ee37f323fe985f14d51289e8914377f0983cbf3d0eebb8abe700861e7836`; normal and `python -O` checkers passed. This V2 supplement supersedes Wash V1 only for the added result-reader edge; the Bool-to-measurement-input fact remains.
