# Spatial control parent to spatial audio boundary

## Status

PRODUCTION_SPATIAL_CONTROL_RECEIPT=BLOCKED

PRODUCTION_SUCCESS_EXECUTION=NOT_RUN

SYNTHETIC_BOUNDARY_FIXTURE=READY_CANDIDATE_TEST_ONLY

PRODUCTION_ALLOWED=NO

REAL_ADM_COLD_INSTALLED48=NOT_RUN

NATIVE_RECEIPT_SCHEMA=NOT_OBSERVED

PRIVATE_AUTH=EXCLUDED

## Authority

The parent owns raw ADM/control cursor and cache, event/control state,
installed48 render history, source flags/features, selector/U04/outer state,
target current/previous/active state, matrix assignment, destinations,
markers and writer-set. It commits those fields once per callback.

F04 spatial audio owns component/LFE/receiver/QMF state. The parent must not
advance receiver/QMF. It emits immutable same-generation inputs and a
SpatialAudioProposal. F04 validates the parent receipt and is the unique
executor of its own after-state.

F05R56 owns reader56/target transition where selected owner projection assigns
it. The production parent receipt must reference that immutable receipt rather
than duplicate its committed state.

## Callback and source clocks

Callback g starts at zero and parent after-generation is g+1. Phase is NORMAL
for real input callbacks and CACHED_DRAIN for exactly four drains. Drains reuse
the last complete padded host38 and control snapshot.

Codec source exists only when g>=2: s=g-2. JOC dispatch is d=s+1=g-1.
Callback-only g0/g1 still commit the parent but have no source/dispatch.
EOF occurs after normal callbacks plus four cached drains; no extra parent
callback is added.

## Production SpatialControlReceipt

The receipt requires:

1. programme/input/config identity and callback phase/sample range;
2. raw host38 and control38 canonical identities;
3. before parent generation/hash;
4. installed48 shape, complete bytes identity and owner token;
5. flags/features identities;
6. selector complete material, selected IDs/count and provider identity;
7. U04 executed flag, inputs/outputs and provider identity, or explicit
   EMPTY_CANDIDATE skip;
8. outer/assignment complete material;
9. matrix15x48 bytes identity, 15 destinations, markers and ordered unique
   writer-set;
10. target previous/current/active complete identities and target transition
    receipt;
11. proposed parent after-state complete bytes/hash;
12. callback/source/dispatch clocks and commit authority.

All stage generation values equal g+1. The receipt digest is SHA-256 of sorted
compact UTF-8 JSON over the complete receipt excluding digest. Hashes never
replace required arrays or bytes at a production boundary.

## SpatialAudioProposal

The proposal contains the parent receipt SHA plus same-generation installed48,
matrix, destinations, markers, direct LFE, control and target material. It also
contains the F04 before-state token. It has no F04 after-state and performs no
receiver/QMF mutation. F04 alone returns the spatial-audio owner receipt.

## Parent state and canonical bytes

The authoritative field layout is selected-adm-installed48-state.v3:
clocks, source cache, event_control, render_audio, source_flags, feature,
selection_outer and matrix_assignment in that order. Fixed arrays are
row-major; integers little-endian; F32 use exact U32 bits; optional values have
presence U8; variable records have U32LE count/length. Callback scratch is
excluded. No pointers, padding or zero-filled unknown fields are allowed.

Cold and callback fixture bytes use the existing AI48ST01 diagnostic encoding
only for test identity. That repeated-digest encoding is explicitly forbidden
as production state. Production canonical bytes remain BLOCKED until a fresh
real cold state is serialized by the authoritative field layout.

## Transaction, failure, reset and EOF

All stages write a clone. Only complete target proposal and downstream input
validation make the parent after-state eligible. The enclosing callback
transaction commits once.

Failures after selector, U04, outer or target proposal discard every staged
parent byte and emit no SpatialControlReceipt or SpatialAudioProposal.
Committed parent and F04 before-state hashes remain unchanged; commit_count=0
and the attempt retires. Fault plans are test-harness controls, never production
parameters.

Reset destroys parent and referenced owner roots and invokes every cold
constructor. EOF validates normal/cached counts and performs no additional
selector, U04, outer or target step.

## Evidence boundary

The seven-callback synthetic fixture provides deterministic recipes, 91 stage
receipts, matrix/target/state material and one recorded outer failure. It has
no PCM file and is not a real ADM production run. Existing F04 material starts
at consumer inputs and cannot serve as the missing parent receipt.
