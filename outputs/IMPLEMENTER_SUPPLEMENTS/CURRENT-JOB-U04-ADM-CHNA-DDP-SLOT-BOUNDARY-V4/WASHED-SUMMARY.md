# U04 ADM CHNA identity and DDP local-slot boundary

Disposition: PARTIAL_STATIC_EVIDENCE_ONLY. This supplement places two independently reported identity chains side by side. It does not close the cross-component join between them, does not add a production rule, and does not establish Gate1 or product readiness.

## Provenance

Source package: spatial-selector-identity-wash-v4.
Source MANIFEST.json SHA-256: e2d162c30b689f2cc01b57c775755388aca441da2faa53ad534c986b1fecb08f.
Source EVIDENCE.json SHA-256: 3714303d7867ee3b08ddb9eb7fff9a1f17b5b4a821dcb718131ba2427da84172.
Source cites approved input crosswalk SHA-256: dda4c052c11b47e5f2d4245ae3d9e9736ebb0dc1fb793ebdb5bd0b4cbf7bbac2.
Base PreparedJobInput XML SHA-256: 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9.

The source package manifest, listed member hashes, and verifier passed independent checks; ordinary Python and python -O also passed. Wash did not independently reproduce the native static traces; each technical claim remains attributed to the hash-bound source package. No audio, feature payload, source listing, disassembly, address, module path, license, or key is forwarded.

## Chain A: DDP local pointer slot

The source package reports a caller-supplied sequence of six-byte metadata records. The parser emits a descriptor per row and writes the zero-based descriptor ordinal into a per-group local source-pointer-table slot. A dispatcher dereferences the caller-owned pointer table at that local slot and routes the pointer to a category table consumed by a later vector stage. The pointer-table owner and initialization are not closed.

## Chain B: ADM BWF track identity

The source package reports that the ADM reader parses each CHNA AudioTrackUID with its one-based track index, and the bed reader uses track-index minus one as a physical slot before storing the ADM source-channel index. The cited approved input crosswalk maps the physical source slots to regular global source IDs according to F03R, excluding the semantic direct LFE; aggregate prefix IDs are a separate domain. This chain is source/input scoped and is not a same-generation join to the DDP pointer table.

## Unclosed join

The following remain UNBOUND or NOT_CAPTURED:

- DDP local pointer-table slot ↔ CHNA UID or track index;
- DDP local slot ↔ ADM global source ID;
- ADM global source ID ↔ DDP feature-matrix candidate column;
- selected ordinal list and its consumer;
- U04 geometry input columns ↔ source-pointer slot or approved F03U 24xN source-weight column;
- same-generation runtime receipt joining the two owners.

Do not connect these domains using ordinal, array shape, numeric coincidence, or event order.

## Native attempt and limits

The prior native wrapper observation remains candidate_count=48, target_count=15, and a 24x48 band-major F32 matrix; the gate entry and selected list were not captured. The latest attempted run exited before measurement and produced no partial output. The package reports its source and component hashes matched the earlier short run and only temp/output paths changed; that is not a new native observation. Memory pressure was not supported by the reported available-memory snapshot, but the failure cause remains unresolved.

No model replay, native gate comparison, Type10, six-plane validation, or Gate1 closure is claimed. The bounded next step is to find the caller-owned pointer-table producer or a same-generation typed receipt that joins its slot to CHNA UID/track index, then bind that to the approved crosswalk and the feature/U04 consumer.