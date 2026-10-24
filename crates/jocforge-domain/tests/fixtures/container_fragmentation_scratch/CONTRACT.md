# Container fragmentation and scratch-parent failure observations

This supplement records a bounded native scratch-parent result. It does not replace the approved F13.1 limit for the selected production profile and does not establish a full production cold path.

## Observed scratch behavior

The 16 input-first cases build a complete four-record container using the approved 40-byte F13.3 configuration layout, but ID11/ID14/ID2/ID1 bodies are deliberately owned opaque test bytes, not semantic producer output. A fresh isolated parent/serializer graph uses either an empty provider or an explicitly caller-owned public one-byte zero test provider; the vendor default initializer was not called and no authentication result bytes or hashes are included.

With the public provider, all tested serialized-container sizes 507, 508, 511 and 512 returned success and were passed from the same container buffer into the scratch planner/refill path. The 512-byte case was represented by two transport fragments: the first is 511 wire bytes (4-byte framing prefix plus 507 container bytes) and the second is 5 wire bytes. The 9-bit limit is per fragment in this scratch path, not a global 511-byte ceiling. This is a transport-fragment observation; it is not a semantic `carrier_block` mapping. Agent 3 is separately verifying that relationship.

With the empty provider, the child serializer returned 9 and the parent returned 7 with zero published output bytes, but the output buffer, selection scratch and some header/history state changed. This disproves a unified rollback guarantee for this scratch parent. It does not establish the complete production outer-codec, four-FIFO, tolerance or durable-publication outcome.

The direct raw-reservation control is separate: at declared length 512 the raw writer emits the low nine bits and returns 0; it does not enforce fragment planning. Implementations must use the planner's fragment lengths and must not infer safe admission from this raw status.

## Normative conflict remains open

The approved F13.1 text states a complete carrier frame beyond its 9-bit range fails. R19 proves that a scratch wrapper with the stated public provider and transport composition can fragment a 512-byte serialized-container buffer into multiple 9-bit fragments. The scratch graph is not production-cold-equivalent, and its serializer→planner composition differs from the production order. Therefore this package records a scoped discrepancy; it does not silently supersede F13.1. Keep the selected-profile global-versus-per-fragment rule open until the semantic carrier/block mapping and same selected production path are independently established.

## Scope limits

The scratch parent was independently constructed and did not clone live state. Production cold equivalence is NOT_ESTABLISHED; full codec/source generation, four FIFO state, outer tolerance/failure root, durable frontend publication and complete semantic ID11/ID14 producers were NOT_RUN. The parent failure results are limited to the tested scratch owner roots. The native fragment equality receipts are checked, but cryptographic fragment contents are not independently reproduced.

This internal package contains no vendor key, authentication result bytes/hash, PCM, original binary or decompiled source. It does not claim whole F13C readiness or product acceptance.
