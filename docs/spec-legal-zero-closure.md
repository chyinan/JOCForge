# SPEC_LEGAL_NOT_IMPLEMENTED zero-closure audit

Status: implementation and validation complete; local commit pending final review.

This document records the human-scale audit for the capability cells that were
classified `SPEC_LEGAL_NOT_IMPLEMENTED` by the central
`CompositionResolver` at JOCForge HEAD
`3b88223353e161344609616d11ab0cd0dabcdd8c`.

## Initial census

The regenerated authoritative matrix has 140,400 rows and 18,369 initial
`SPEC_LEGAL_NOT_IMPLEMENTED` cells. The exact grouped census is in
`artifacts/spec-legal-zero/gap-families.json` and `.csv`.

| Family | Root cause | Cells | Current reason |
|---|---|---:|---|
| F1 | skip-field profile eligibility | 17,680 | I0 skip-field carriage is closed only for idx0 |
| F2 | skip-field independent LFE | 130 | skip-field LFE ownership has no validated source-derived path |
| F3 | skip-field canonical short partitions | 104 | deployed skip-field signaling has only a validated six-block carriage |
| F4 | legacy-core LFE ownership | 390 | legacy-core LFE ownership has no validated composition path |
| F5 | legacy-core deployed EMDF | 65 | legacy-core closure is validated only with strict EMDF |

`UNIQUE_GAP_FAMILIES_INITIAL=5`.

The first resolver-only correction already separated true implementation
families from stale classifications: skip-field dependent requests are not
applicable to an I0-only carriage, and `1+2+3` is still publicly ambiguous.
Those changes are recorded in `artifacts/spec-legal-zero/reclassifications.csv`
with the original status and exact public basis.

## Audit rules

Each family must be reproduced through the resolver or a production encode
before implementation. A family can leave the initial status only by:

1. generalized production support through `CompositionResolver` and the
   corresponding writer, or
2. an independently documented public-semantic reclassification with the old
   status, new status, exact dimensions, normative basis, and evidence.

Capacity is evaluated after semantic applicability. A legal family at a rate
that cannot contain its public carrier is `REJECTED_CAPACITY`, not a hidden
implementation gap. Existing public ambiguities, CMAF boundaries, canonical
semantic ceilings, and legacy short exclusions remain frozen.

## Family audit ledger

### F1 — skip-field profile eligibility

The current fail-closed path rejected every `idx1..idx4` I0 skip-field request
before bitrate planning. The audit distinguishes seven-input profiles, which
have no D0 topology in this carriage, from `idx3`, which reuses the five-input
phase family. Strict idx3 rows are implemented through the same five-channel
writer; dependent-free seven-input rows and unsupported signaling/rate
boundaries are independently reclassified.

### F2 — skip-field independent LFE

The existing six-block skip-field injector already walks independent 5.1
E-AC-3 audio syntax, but the resolver and CLI rejected independent LFE before
that path could be used. The closure now carries the real I0 LFE for both
six-block and canonical source-derived short skip-field output; it remains
outside the JOC matrix.

### F3 — skip-field canonical short partitions

The existing short writer creates public 1/2/3-block frames, but skip-field
carriage had no audio-block skip-field insertion path. The closure now emits
the canonical `3+3`, `2+2+2`, and `1+1+1+1+1+1` families through the plan;
the mixed `1+2+3` row remains an ambiguity and is not used as evidence.

### F4 — legacy-core LFE ownership

The legacy AC-3 I0 and E-AC-3 D0 writers already exist independently, but the
composition API did not pass an explicit LFE ownership graph into that path.
The plan-aware writer now covers independent, supplement, and replacement
semantics without duplicating or dropping the single programme LFE.

### F5 — legacy-core deployed EMDF

The EMDF serializer already has strict/deployed codec-data signaling. The
legacy resolver no longer preempts deployed signaling, and the plan-aware
legacy path preserves the selected signaling while retaining the original
AC-3 I0.

## Reclassification ledger

The current ledger records 18,284 evidence-backed reclassified cells and 85
initial cells moved to generalized production support. It intentionally groups
F2/F3 mixed and bitrate boundaries where the same public boundary applies;
the authoritative matrix counts remain the final source of row-level truth.

## Resolution results

| Family | Initial | Implemented | Reclassified | Final result |
|---|---:|---:|---:|---|
| F1 skip-field profile eligibility | 17,680 | 8 | 17,672 | 8 supported; 26 ambiguous; 96 capacity; 17,550 not applicable |
| F2 skip-field independent LFE | 130 | 8 | 122 | 8 supported; 26 ambiguous; 96 capacity |
| F3 skip-field canonical short | 104 | 6 | 98 | 6 supported; 26 ambiguous; 72 capacity |
| F4 legacy-core LFE ownership | 390 | 54 | 336 | 54 supported; 336 capacity |
| F5 legacy-core deployed EMDF | 65 | 9 | 56 | 9 supported; 56 capacity |

The implementation families use the central `CompositionResolver` and real
production writer paths. F1's supported cells are the five-input strict idx3
skip-field family; I0-only idx1/idx2/idx4 have no seven-channel dependent
content and are therefore not applicable. The separate idx3 deployed rows are
not applicable because deployed compatibility remains the established idx0/1024
preset. Mixed `1+2+3` cells were restored to the existing public ambiguity
boundary before capacity was evaluated.

## Final matrix

The regenerated matrix has 140,400 rows:

| Verdict | Cells |
|---|---:|
| `SUPPORTED_PASS` | 2,393 |
| `PUBLIC_AMBIGUOUS` | 2,990 |
| `NOT_APPLICABLE` | 110,825 |
| `SEMANTIC_CEILING` | 6,240 |
| `REJECTED_CAPACITY` | 10,152 |
| `INVALID` | 7,800 |
| `RESERVED` | 0 |
| `SPEC_LEGAL_NOT_IMPLEMENTED` | 0 |

The zero assertion is enforced by
`crates/jocforge-core/tests/plan.rs` and by
`scripts/run-spec-legal-zero-closure.ps1`. The generated count artifact is
`artifacts/spec-legal-zero/final-matrix-counts.json`.

## Validation evidence

The reproducible synthetic closure is
`scripts/run-spec-legal-zero-closure.ps1`; it validates deterministic output,
the independent JOCForge structural path, current OpenJOC Inspector JSON,
the frozen OpenJOC inspect/render authority, and FFmpeg decode for one
representative spanning the newly implemented families.

The real Hanabi supplement is recorded in
`artifacts/spec-legal-zero/hanabi-summary.json`. Three production CLI runs
passed: idx3 strict skip-field six-block; idx0 deployed skip-field
`2+2+2` with one independent LFE; and idx1 legacy AC-3 + E-AC-3 D0 deployed
with one independent LFE. Each has 9,042 access units, zero malformed AUs,
finite full render, continuous source accounting, and deterministic repeat
output. No OpenJOC or genuine stream-error failures were accepted.

## Status-laundering audit

`artifacts/spec-legal-zero/reclassifications.csv` lists every reclassification
family, affected count, old verdict, new verdict, reason, and public evidence.
`artifacts/spec-legal-zero/final-gap-disposition.csv` is the row-level audit
for all 18,369 baseline gap keys and is checked against a fresh baseline
worktree on every closure run.
No row was changed solely to reduce the gap count: applicability follows the
public topology, ambiguity follows the frozen mixed-partition rule, and rate
boundaries follow the central capacity allocator. The implementation rows are
covered by the generalized writer and representative evidence rather than by
matrix output alone.

## Capability claim

Within the currently modeled public ETSI E-AC-3 JOC capability space, JOCForge
implements every composition classified as publicly unambiguous, semantically
meaningful, applicable, and capacity-viable. This is not a claim of every
possible Dolby Atmos bitstream, proprietary encoder equivalence, patent
clearance, or byte equivalence with Dolby encoders.

## Validation and final claim

The final claim is permitted only when the authoritative regenerated matrix
contains zero `SPEC_LEGAL_NOT_IMPLEMENTED`, every implemented family has a
real generalized encode path, and structural/source-causality/OpenJOC
inspect-render/regression gates pass. This document will be updated with the
final matrix counts, evidence artifacts, commits, and any reclassification
records after that audit.
