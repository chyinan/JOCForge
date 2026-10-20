# F07-F11 analysis owner root

## Status

ANALYSIS_FIXTURE_SEAM=READY_CANDIDATE_TEST_ONLY

PRODUCTION_ALLOWED=NO

ACTUAL_FFT_TRANSFORM=NOT_RUN

REAL_ADM_EXECUTION=NOT_RUN

PRODUCTION_ANALYSIS_API=BLOCKED

PRIVATE_AUTH=EXCLUDED

## Authority and state

The analysis root owns four persistent families in this order:

1. input_history: generation and 7680 U32 words;
2. transient: generation, two-history20, three-history30, peak statistics90,
   previous mean5 U32, short queue20 I1 and other queue20 I1;
3. primary_companion_lfe: generation, primary FBW640, companion FBW640,
   primary LFE128 and companion LFE128 U32;
4. policy: generation, gain/db history64 U32, exponent refresh30 U16 and
   normalized anchors30 U16.

Root lifecycle fields are source, retired and failure_stage. Codec state,
previous_short, payload FIFO and publication count are not analysis-owned.

Canonical state is the complete expanded JSON object with those fields,
UTF-8, keys sorted, comma/colon separators, finite values only. SHA-256 covers
the exact bytes. Cold state is the frozen projection from
F07-F12-SOURCE-COMPOSITION-VERIFICATION-RESULT.json and all words/generations
are zero.

## Source clocks and stages

Source s is continuous. All four successful after generations equal s+1.
Source0/1 are publication priming but analysis executes normally. EOF after
4249 sources adds no transient, transform, exponent or converter flush.

Stage authority:

1. F07 input history advances once;
2. F07A transient consumes installed audio and advances once;
3. F08/F08NZ primary, companion and LFE transform advance once;
4. F09/F09E/F09PA/F09G/F09LB generate power, exponents, gain/db and policy;
5. F10 normalizes anchors/groups;
6. F11 derives main/converter plans without aliasing them.

Candidate search and frame writing belong to codec and cannot advance analysis.
F06ND consumes SharedAnalysisReceipt and may not rerun F07. F12 consumes
SharedAnalysisReceipt plus NumericPlanReceipt. F05JL has no direct authority to
advance analysis state; any shared data must remain immutable and lineage-bound.

## Typed receipts

SharedAnalysisReceipt binds source, generation, provider, before/after root
SHA, advancement_count=1, installed/delayed audio identity, short/other flags,
primary/companion/LFE spectrum identities and complete after-state.

NumericPlanReceipt binds the same source/generation/provider and the
SharedAnalysisReceipt SHA. It contains six-stage completion flags, power and
raw exponent identities, normalized plan, gain/db, main frame codes, converter
frame codes and converter SNR offset. Main and converter codes are distinct
producer outputs.

Production success proposes one full after-state. Fatal discards the clone,
returns no receipts and retires the attempt. Same-source retry is forbidden.
Reset restores cold. EOF validates count and emits no plan.

## Test-only seam

The seam validates three successful frozen recipes and one all-candidate-fail
record. It checks source/generation/provider, shapes, complete owner state,
owner hashes and receipt lineage. It does not execute ADM input, F07 equations,
FFT/transform, exponent or policy math.

Success returns validation receipts with production_allowed=false,
commit_count=0 and state_delta=null. Recorded source commits are observations.
The fatal record demonstrates committed analysis state unchanged while
lifecycle retires, but remains fixture evidence.

Synthetic meter/DRC traces do not establish analysis input or execution.
Production remains blocked until actual transforms and real ADM source replay
are run through the isolated API.
