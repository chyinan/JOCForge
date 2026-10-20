# F12 codec owner root

## Readiness

CODEC_FIXTURE_SEAM=READY_CANDIDATE_TEST_ONLY

PRODUCTION_CODEC_ROOT=BLOCKED

PRODUCTION_ALLOWED=NO

REAL_4249_SOURCE_REPLAY=NOT_RUN

PRODUCT_ACCEPTANCE=NOT_RUN

PRIVATE_AUTH_HMAC_VENDOR_KEY=EXCLUDED

NATIVE_RECEIPT_SCHEMA=NOT_OBSERVED

The selected profile is E-AC-3, 5.1, 48 kHz, 1024 kbps, 4096-byte frames,
six blocks, acmod7, LFE present and one independent substream. deployed_compat
is fixed for this package.

## Authority and required production inputs

The codec owner controls only codec transform/bitwriter state, previous_short,
candidate scratch and complete frame proposal. It cannot execute or replace:

- F06ND BSI/DRC and its MeasurementReceipt binding;
- SharedAnalysisReceipt and NumericPlanReceipt from F07 through F11;
- F05JL ID14 body/clock receipt;
- F13 timer, four FIFO, popped writer lineage and complete EMDF receipt;
- F14K public integrity result;
- F15 durable output.

Production F12SC requires the actual same-source receipts and complete material,
with source/generation/provider hashes equal and advancement_count one.
Digest-only placeholders or supplied fixture fields cannot satisfy this gate.

## Clock and lifecycle

For source s, dispatch is s+1 and AU is null for s<2, otherwise s-2. Source0
and source1 execute and commit codec state but publish no AU. EOF after exactly
4249 sources adds no codec flush. Reset reconstructs cold state; fatal discards
candidate and staged codec state, returns commit0 and retires the programme.
The same source cannot retry in the same attempt.

## Selected state projection and canonical bytes

The available selected composition schema projects codec state as generation
U64, next source U64, 256 transform-history U32 words, 18 diagnostic/group
U16 words, bitwriter cursor U32, five previous_short I1 bytes and finalized U8.
Cold values are zero/false.

Canonical bytes are ASCII CodecStateV1 plus NUL; U16LE profile length and UTF-8
profile; raw provider SHA-256; generation and next source U64LE; 256 U32LE
history words; 18 U16LE words; cursor U32LE; five U8 I1 values; finalized U8.
State identity is SHA-256 over these bytes.

This is the frozen selected composition projection. F12S prose also describes
six transform histories without a complete independent per-history canonical
schema. Therefore this package does not promote the projection to a general
production codec root. Production remains BLOCKED until that identity is
reconciled with actual producer-owned history bytes.

## Test-only fixture seam

The frozen complete-frame input vectors provide spectra recipe, exponent plan,
BSI, short/previous_short, db/converter outputs and optional EMDF. The seam
validates input completeness and the recorded result: 4096 bytes, SHA-256,
CRC residual zero, candidate/search facts, failed-trial commit0 and one
recorded source commit.

Success emits codec-fixture-validation-receipt and the verified frame proposal.
It commits no codec state: commit_count=0 and state_delta=null. Recorded
state_commit_count=1 is evidence about the frozen model run, not a commit by
this seam.

The four fixtures are not production inputs. They do not prove new PCM,
upstream receipt execution, full-song continuity, decoder/render quality or
general codec behavior. The 512-byte payload rejection remains a fixture
boundary with trial0/commit0.

## Production proposal and commit semantics

A production proposal may be emitted only after all required receipts and
complete state bytes validate. Candidate trials use scratch. Failed candidates
never modify staged state. A successful 4096-byte frame with CRC residual zero
may propose one after-state. The enclosing source transaction commits that
state once. Any fatal returns no frame/proposal/receipt, leaves committed codec
state unchanged and retires the attempt.
