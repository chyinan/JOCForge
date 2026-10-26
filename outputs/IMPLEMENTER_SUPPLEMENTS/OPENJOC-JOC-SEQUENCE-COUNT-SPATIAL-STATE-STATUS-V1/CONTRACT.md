# OpenJOC JOC sequence-count spatial-state contract

Status: `IN_BAND_SEQUENCE_STATE_CONSUMER_STATICALLY_CLOSED; NATIVE_SOURCE_GENERATION_JOIN_UNBOUND`.

## Parsed field and decoder state

In the reviewed OpenJOC E-AC-3/JOC path, the JOC payload parser reads a 10-bit sequence count into `JocFrame.sequence_count` (`u16`, numeric range 0 through 1023). The E-AC-3 metadata carrier retains the JOC payload bytes and passes them to the JOC payload decoder; the inspected carrier record does not convert this field into an E-AC-3 substream identifier or an explicit native source-generation token.

The JOC decoder retains the previous sequence count together with per-object coefficient matrices and QMF analysis/synthesis state. Sequence zero, a missing/unexpected count, or a channel/object-topology change resets that decoder state. A previous count of 1023 accepts 1 as the normal wrap successor. When state reset reaches the speaker renderer, it clears delayed reconstruction/timeline and spatial bridge state before rendering.

An explicit packet discontinuity recreates stream state. A timestamp gap without that discontinuity is rejected before decode. The payload decoder separately rejects a non-next outer frame index; that outer frame index is not the 10-bit in-band sequence count.

## Audio-behavior impact and limits

The in-band sequence count directly controls whether spatial interpolation/QMF history is retained or reset. It therefore affects the decoder's continuous spatial-state behavior. It does not act as a direct object-matrix coefficient or PCM synthesis parameter. A single frame with a valid in-band count can be parsed and decoded without proving its native producer generation; a claim of continuous same-generation spatial reconstruction across a run requires that upstream join.

The checked files contain no named `sequence_base` field or native source-generation field, but this is a bounded file-scope observation, not a whole-repository absence claim. The selected DEE source/reset-generation to emitted sequence-count join remains `UNBOUND`. No listening test or real-audio decode was performed, so perceived severity is not assessed.
