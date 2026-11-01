# Range output to spatial consumer trace

This Wash-only addendum reuses the authorized 38-channel sample and the existing reader-to-codec evidence. It performs a narrow, read-only static trace in the already imported processing image. No new native job was started while the R4 capture was reported in progress.

The source file-position range is converted to separate start and end sample offsets. A multi-source IO-set builder reads those values and assembles two per-source qword vectors. Its callers pass the resulting vectors into the output-item setup loop before selecting an encoder branch. This closes the static range-to-IO-set edge.

The spatial keyframe selector accepts a binary32 query scalar, compares it with binary32 control-point times, and emits interpolation weights. Some observed callers generate control tables over fixed query times. The static evidence does not identify a live query scalar as the absolute ADM sample position, nor join the range vectors or R138 reader cursor to a spatial callback, host frame, or fit generation. The requested same-call edge therefore remains unbound.

## Minimum next observation

After reviewing the pending R4 receipt, capture only any still-missing fields in one selected 38-channel reference path:

1. At the completed range-to-IO-set boundary, record source-owner token, input count, start/end sample offsets, sample rate, and output vector identity.
2. At the first spatial keyframe query reached from a live render callback, record the binary32 query value, control-track owner, control-point count, output-vector identity, and caller frame/callback/fit-generation tokens. Exclude setup-time fixed-grid calls.
3. At the adjacent reader request, record reader-owner token, relative cursor, requested frame count, thread, and active call-stack token.
4. Join only through explicit object identity and nested call/callback state. Keep ADM sample position, reader cursor, host frame, spatial callback, fit generation, and consumer generation separate. Read no PCM and persist no audio payload.

If R4 already captured any of these fields, reuse its receipt and observe only the remaining edge. No new run is authorized by this static addendum.

`GATE1_CLOSED=false`; model replay and composition remain `NOT_RUN`.
