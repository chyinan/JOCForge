# Same-input selector source identity reference and DDP slot boundary

Disposition: REFERENCE_RUN_SOURCE_IDENTITY_BOUND_DDP_JOIN_UNBOUND. This reconciles two separate evidence packages. It does not claim the reference job is the current accepted PreparedJobInput, does not bind the DDP slot to the selector table, and does not close U04 or Gate1.

## Provenance

New source package spatial-selector-identity-wash-v5 manifest SHA-256: 180a4b561b2e8e4326d30fdda5bd2f3b7d0fc28a415df05c84255c54471f273b.
New source EVIDENCE.json SHA-256: 966da2a07604c6a7927e3257a9f7425e18770a85e7b965d8f7ce3c15356e06d1.
Prior selector package spatial-source-selection-wash-v2 manifest SHA-256: 55ec6020b9b5f5f5802a359febaf37498592621b8b026e9a407d67e0d121892f.
Prior selector EVIDENCE.json SHA-256: f6d84b07d4071f59216bf3111f704f53fbab8d48432b68f5801c550b82cb201c.
Both source checkers passed ordinary Python and python -O. Wash rechecked the source member hashes and confirmed the manifest/evidence references agree.

The reference selector run used the same authorized 38-channel BWF hash 7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1 and approved input crosswalk hash dda4c052c11b47e5f2d4245ae3d9e9736ebb0dc1fb793ebdb5bd0b4cbf7bbac2. Its job XML was befb67c91ae67489090062d287785121c07bd34db0985d9b0dbed70279376b9e (reference job) and its configured input window was frames 2160..2208. The current accepted PreparedJobInput XML is 20669e276ac7b708b0a03b127d3f37331c8cbc5cfce9a0f4a2498739ecc36cb9. Therefore this is same-input reference evidence, not a same-job/current-run claim.

## Selector-to-ADM identity bound in that reference invocation

The prior package records a completed native reference call chain. It binds 48 source descriptor columns, 11 aggregate-prefix columns, and 37 regular owner rows. Owner row i maps to source column i+11; source columns 11..47 map to regular global source IDs 11..47 under the hash-matched F03R input crosswalk. Prefix columns 0..10 are aggregate provenance without ADM AudioObjectUID. In that invocation the gate had 36 eligible candidate rows; 15 were selected/active and fed a 15x48 matrix consumer.

This establishes the selector descriptor column to global source identity for the bounded reference fixture. Numeric matrix coefficients and the U04 invocation/output were NOT_CAPTURED. Model replay, Type10 and six-plane validation were NOT_RUN.

The approved crosswalk chain is AudioTrackUID to one-based CHNA track index, then physical slot = track_index-1, then F03R regular global ID 11+rank after excluding the direct-LFE physical slot. Aggregate IDs 0..10 remain a separate domain. The converter parser trace reports AO_xxxx normalization by subtracting 0x1001 and ATU_xxxxxxxx by subtracting 1; it does not prove the ATU value equals a CHNA track index and does not establish a DDP slot join.

## DDP join still open

The DDP static chain binds metadata-row ordinal to a descriptor ordinal, then to a caller-owned source-pointer-table local slot, and from there to category routing/consumer. No shared table identity or same-generation receipt connects that DDP local slot to the selector source-pointer table or any column 11..47. No DDP native gate capture or selected list was obtained in the failed short replay. Keep DDP slot ↔ selector column/global ID UNBOUND; do not join by ordinal, count, shape, or event proximity.

This package contains only sanitized evidence summaries and source hashes, no raw audio/features, code, disassembly, addresses, native binary, license, or key. The static DDP technical trace remains attributed to its source package and was not independently reproduced by Wash.