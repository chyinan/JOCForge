# Real cold callback0 spatial-control production admission

## Current result

RAW_ADM_INPUT_IDENTITY=PINNED

CALLBACK0_CONTROL_MATERIAL=NOT_RUN

AUTHORITATIVE_PARENT_BEFORE_STATE_BYTES=BLOCKED

CALLBACK0_NUMERICAL_EXECUTION=NOT_RUN

AUTHORITATIVE_PARENT_AFTER_STATE_BYTES=BLOCKED

SPATIAL_CONTROL_RECEIPT=BLOCKED

SPATIAL_AUDIO_PROPOSAL=BLOCKED

PRODUCTION_ALLOWED=NO

The real ADM file, PCM extent and ADM-related chunk identities are pinned in
REAL-ADM-INPUT-IDENTITY.json. No PCM or metadata content is included.

## Owner and input boundary

frontend_control owns parsing axml/chna/dbmd into immutable programme metadata,
physical binding, callback0 event/control state and control38. spatial_control
consumes the same callback's raw host38/control identity and owns installed48
render history, flags/features, selector/U04/outer, target state and matrix
assignment. spatial_audio/F04 uniquely owns component, LFE, receiver and QMF.

An input hash alone is insufficient. Production callback0 requires complete
runtime material or an authorized content-addressed material reference for:
host38, control38, authoritative parent before-state bytes and every provider.

## Authoritative parent state

The schema field order is clocks, source_cache, event_control, render_audio,
source_flags, feature, selection_outer and matrix_assignment. Fixed arrays are
row-major. Integers are little-endian. F32 values use exact U32 bits. Optional
values have U8 presence. Variable records have U32LE count and length.
Callback scratch, pointers, structure padding and unknown zero-fill are
forbidden.

Pinned constructors:

- frontend event/control constructor SHA-256
  b0548b0c45e321795455c68095e804de5507af318636676353a73f119a41f7cb;
- selected target cold constructor SHA-256
  d9cd904b23aebc72e008f5aaacd06c095e4906acdd6de16fe7358421b4769073;
- provider manifest SHA-256
  32cdd7d80f62db4f2eb5e53f9bcd28b83bfc2b3b9fac23f8bef6143b9668ee0a;
- parent state schema SHA-256
  5138318de304b83d26e47271c1661bcc903cebcd2cea457ae98e0048b161ed46.

These identities define required constructors and layout. They do not provide
the authoritative serialized callback0 before-state. No old test fixture hash
may substitute for it.

## Same-generation execution contract

Callback g=0 has generation1 and no codec source or JOC dispatch. A successful
execution must produce complete same-generation material:

1. installed48 F32[48][3072];
2. selector inputs/outputs and selected prefix;
3. U04 input/output or explicit empty-candidate skip;
4. outer/assignment input/output;
5. matrix F32[15][48], destinations[15], markers[15], ordered writer-set;
6. target previous/current/active and transition material;
7. authoritative parent after-state bytes.

Each stage proposal references the previous material identities required by
its actual dataflow. Every generation is1. The final parent receipt carries
complete before/after state hashes and single commit authority.

SpatialAudioProposal consumes the accepted parent receipt and the full
installed48/matrix/destination/target material. It contains an F04 before token
and receiver_qmf_mutated=false. F04 alone creates its after-state.

## Canonical digest and commit

Parent digest is SHA-256 over authoritative serialized bytes, with no domain
prefix. Receipt digest is SHA-256 over its complete sorted compact UTF-8 JSON
object excluding digest. Receipt hashes cannot replace required runtime
materials.

All parent computation occurs on a clone. The enclosing callback transaction
may commit the parent once only after the target proposal and F04 input
validation. Callback0 has no source/AU publication.

## Failure, reset and EOF

Test-harness failures after selector, U04, outer and target proposal discard
the clone. Parent before/after identity is exactly equal, F04 before token is
unchanged, receipts/proposals are absent, commit_count=0 and attempt status is
RETIRED. Fault controls are not production parameters.

Reset invokes every pinned constructor and must output authoritative cold bytes
before callback0. EOF is not exercised by this admission slice. The full
programme still requires normal callbacks plus four cached drains.

## Exact blocker

No production success vector is delivered. The missing bounded execution is:

real ADM identity -> parsed callback0 control38 -> authoritative cold bytes ->
installed48/selector/U04/outer/matrix/target full material -> authoritative
after bytes.

Until one execution supplies these bytes and hashes in the same generation,
SpatialControlReceipt and SpatialAudioProposal remain BLOCKED. The sanitized
test-only callback0 amendment cannot be promoted.
