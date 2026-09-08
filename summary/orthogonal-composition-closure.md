# Orthogonal JOC composition closure investigation

Verified 2026-09-08 against JOCForge HEAD `36532efa2fc47b7bb5bf2a5dbe40beeca5c63620`, local OpenJOC, ETSI TS 102 366 V1.4.1 and ETSI TS 103 420 V1.2.1.

## Current boundaries

- `crates/jocforge-core/src/plan.rs` resolves only profile, carriage, EMDF, bitrate and block partition. The dependent count and LFE ownership graph are passed through separate conformance/writer APIs.
- `package_source_derived_short_block_access_unit` is a source-derived two-substream writer: it writes I0/D0 pairs and places EMDF only on final D0. It already supports independent I0 LFE for canonical raw strict short partitions.
- `package_multi_dependent_source_derived_access_unit_with_plan` validates immutable ownership and writes one six-block I0 followed by D0..Dn. It already supports dependent LFE supplement/replacement and final-dependent EMDF ownership.
- CMAF is intentionally separate and currently requires a complete six-block I0+D0 raw sample.

## Normative composition findings

- TS 102 366 E.1.3.1.2 requires dependent substreams to immediately follow their associated independent substream, use sequential IDs, and contain the same number of blocks per syncframe and sample rate as I0. The canonical short multi-dependent form therefore uses one shared partition for I0 and every dependent; heterogeneous component partitions are not emitted.
- TS 103 420 clause 8.2 requires OAMD/JOC EMDF in the last dependent whenever dependents exist. The final dependent remains the per-programme JOC/EMDF owner for short groups.
- TS 103 420 Table 47/48 defines idx0..idx4; LFE is optional and bypasses JOC processing. LFE must remain outside the full-band reconstruction matrix, with exactly one programme owner.
- TS 103 420 Annex E restricts CMAF to one I0 plus optional D0 and six blocks. Raw multi-dependent and raw short rules must not relax CMAF.
- A fixed AC-3 I0 does not expose the E-AC-3 `numblkscod` axis as a composable per-component short plan; legacy-core short is therefore `NOT_APPLICABLE` in the canonical closure.

## Implementation boundary

1. Add a pure `CompositionRequest`/`CompositionResolver` in the core, returning a plan or a typed capability classification with an exact reason.
2. Keep compatibility wrappers for existing six-block APIs, but have them call the same resolver.
3. Close canonical raw strict/deployed I0+D0 short cells and raw strict multi-dependent short cells only when a source-derived writer and structural validator cover the same shared partition.
4. Keep dependent-LFE short, legacy short and CMAF short/D1+ fail-closed unless the public and writer contracts are both closed.
5. Generate a cheap multidimensional matrix from the pure resolver; run OpenJOC/FFmpeg only for canonical representatives, boundary rates, pairwise interactions and one legal maximum-complexity raw stream.
