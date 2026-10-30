# U04 local source-pointer slot producer boundary

Disposition: PARTIAL_STATIC_EVIDENCE_ONLY. The static trace reported by the source package now binds the local descriptor ordinal to a caller-owned source-pointer-table slot and its category consumer. It does not identify that slot as an ADM physical source or global source ID. This adds no production rule, algorithm, expected output, or implementation approval.

## Provenance and identity

Source package: spatial-selector-identity-wash-v3.
Source MANIFEST.json SHA-256: 027005783ec9afd7323cd5b02c634523bcf0ba1b825b517586aeef88d74f50cc.
Source EVIDENCE.json SHA-256: c8f0e30efb4a2441ea6c96ff8cca558f2edf7fc65264a58fcaaa69d05d49a890.
Base PreparedJobInput XML SHA-256: 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9.

The source manifest, all listed payload hashes, and the supplied checker were independently verified. Ordinary Python and python -O checks passed. Wash did not independently reproduce the static source trace; the technical claims below are attributed to the hash-bound source package. This handoff contains no raw audio/features, source code, disassembly, addresses, native binaries, license, or key.

## Static producer and consumer path reported

- A caller supplies a record count and contiguous six-byte metadata records. The records are copied byte-for-byte into a configuration object.
- The parser processes one record per iteration, converts its type byte to an internal type category, and compares a 16-bit grouping key with the previous key to form adjacent groups.
- It emits one packed 16-bit descriptor per input record. The descriptor ordinal, zero-based in that sequence, is written into the per-group source-pointer-table slot field. Special high-bit descriptors carry that same local ordinal through a dependent-item record.
- A category dispatcher later uses that slot to dereference a pointer in the caller-owned source-pointer table and route it into a category table. A same-record downstream consumer reads the derived group flag and consumes the routed vectors.

This supports input-record ordinal → descriptor ordinal → local pointer-table slot → category-table pointer → consumer, as a static local chain. The 16-bit grouping key is used for adjacent-group equality and is not the slot value. The remaining input-record bytes are not semantically attributed.

## Identity boundary

The pointer table is supplied by an external caller-owned descriptor object; its owner and initialization are not closed. No explicit ADM physical slot, global source ID, UID, or approved crosswalk key was observed in this bounded component chain. Bulk-copy or external initialization remains possible. The observed local slot must not be equated with ADM identity.

Prior native evidence remains limited to a wrapper entry with candidate_count 48, target_count 15, and a 24x48 band-major F32 feature matrix. The actual selector gate entry and selected ordinal list were not captured. The prior U04 geometry helper input was 20x7 with local columns mapped to geometry-point indices 0..6 and mask bits [0,1,2,4,5,6,7]; its join to the source-pointer slot and to F03U 24xN source-weight columns remains UNBOUND.

Keep distinct: ADM global ID, physical input slot, descriptor ordinal, pointer-table slot, local candidate ordinal, selected ordinal, feature-matrix column, geometry-point index, mask bit, U04 input column, and source-weight column. Do not infer identity from shape, ordering, event timing, or equal values.

## Current status and next action

No new native measurement was captured in this package. The current attempt failed during audio-component initialization before measurement and produced no partial reference output. Model replay, native gate comparison, Type10, six-plane validation, and Gate1 remain NOT_RUN or NOT_EVALUATED; Gate1 is not closed.

Next, trace the caller-owned pointer-table producer and the six-byte metadata owner to an explicit ADM identity key or approved crosswalk. Then bind that key in the same generation through the local slot, selected ordinal, and U04 consumed slice. Keep each edge separately open until its producer and consumer are identified.