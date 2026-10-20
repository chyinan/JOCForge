# Selected none-DRC owner root

## Status

NONE_DRC_LIFECYCLE_SEAM=READY_CANDIDATE_TEST_ONLY

PRODUCTION_ALLOWED=NO

WHOLE_METER_FRONTEND_AUDIO_PATH=NOT_RUN

REAL_4249_NONE_DRC=NOT_RUN

SECONDARY_PRODUCTION_API=BLOCKED

OTHER_DRC_PROFILES=BLOCKED

NATIVE_RECEIPT_SCHEMA=NOT_OBSERVED

This package covers the selected none profile boundary only. It does not
promote the meter root or other line/RF profiles.

## Provider and numerical boundary

The provider includes thirteen authorized binary32 words in fixed order:
filter5, normalize3, quantize3 and auxiliary rotation2. The U32 words are
authoritative. It pins the separately generated 34-word mathematical provider
for dialnorm and mix values.

Normalize, quantize, paired filters, statistics, peak carry, limiter,
scheduler and auxiliary rotation require those providers plus finite primary
and secondary 6x1536 F32, strict 6x5 short flags, frozen programme measurement
digest and U5 dialnorm.

The selected F06NF fixed FFT and F06NI inverse rules have local review
evidence, and F06NR defines forward/inverse overlap state. Their actual
whole-source production API and ordinary window integration have not been run
here. Secondary input in local none-DRC statistics vectors is synthetic.
Other FFT sizes, other windows, category1 authored control and every non-none
DRC profile remain BLOCKED.

## Committed state and canonical digest

State fields, in order of ownership, are:

1. schema/profile/provider identity;
2. source_index and generation;
3. immutable measurement_sha256 and programme_dialnorm;
4. ordinary history 7680 U32;
5. secondary forward history 640 U32, inverse carry 640 U32 and frame_index;
6. statistics: filter pairs10, primary peaks6, secondary peaks6 and downmix
   peak, all U32 binary32 words;
7. scheduler: line/rf dynamics level/hold/log_gain, line/rf limiter gain/hold,
   peak logs6 and scheduler generation;
8. override FIFO records4, metadata_delay, append_slot and frames_processed;
9. finalized.

Canonical serialization is UTF-8 JSON with keys sorted, comma/colon separators,
ensure_ascii=false and finite values only. Arrays are always expanded; repeat
descriptors are forbidden. SHA-256 over those bytes is the state digest.

Cold main state requires a committed programme measurement and dialnorm. All
audio histories/statistics are positive-zero words. Dynamics levels are
binary32 1.0, holds/log gains/limiter/peaks zero. Override records are four
zeros, metadata_delay3072, append_slot3 and frames_processed0.

## Clock, authority and transaction

Source s is continuous from zero. DRC generation after success is s+1.
Source0/1 are codec priming sources but none-DRC executes normally; this owner
does not publish AU. EOF after 4249 sources adds no DRC zero frame or flush.

The DRC owner controls ordinary delay history, secondary forward/inverse carry,
statistics, scheduler and override FIFO. SharedAnalysis supplies immutable
same-source primary spectrum/audio/short flags and advances analysis once.
DRC must not rerun F07.

A production proposal requires complete primary/secondary material and all
providers. It computes on a clone and returns RF U16, six line U16 and a full
after-state. The enclosing source transaction commits once. Shape, provider,
clock or numeric failure returns no controls/receipt and leaves committed
state unchanged; the attempt retires and requires full reset.

## Test-only lifecycle seam

The seam accepts a caller-provided synthetic F06ND stage receipt containing
clock, provider/measurement identity, input shapes, finite flag, RF/line
controls and complete after-state. It verifies every field and canonical
before/after hashes. It does not execute secondary FFT, normalize, filters,
statistics, limiter or scheduler.

Success emits a validation receipt with production_allowed=false,
commit_count=0 and state_delta=null. The receipt's recorded after-state is
test evidence only. Missing material, wrong capacities, shape, clock, provider,
measurement or non-finite status fails without a validation receipt.

Reset recreates cold main state from the same measurement/provider. EOF checks
source and generation only, emits no controls and marks the test projection
finalized. Whole meter restart and global CallbackTransaction remain outside
this seam.
