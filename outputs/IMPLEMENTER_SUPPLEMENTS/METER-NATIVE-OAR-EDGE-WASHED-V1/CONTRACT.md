# Native OAR edge observed inside Meter callbacks

## Bounded native finding

The Wash-verified captures bind the native OAR spatial-coding process, `SpatialCodingState` construction, spatial-coding frame consumer and metadata-row consumer to the same selected Meter callback path. In run 020 callback 1, the constructor return is joined to the OAR nested-state field and frame-consumer state pointers; the metadata consumer reads constructor table B, which has 16 rows at 52-byte stride. Sampled row indices differ across runs and are preserved as observed, not normalized into a target slot.

Run 019 callback 4247 is a 38-source / 1536-sample input frame. Callback 4248 is a 0-source / 0-sample `SOURCE_EMPTY_CANDIDATE`; OAR and consumers still run synchronously on that callback. It is not the approved model F06OF DRAIN input, which is a full 38 x 1536 positive-zero raw frame. Numeric ordinal equality does not join the native callback to the model drain generation.

## Explicit non-joins

The native 16-row metadata table is not identified as product F06OF `control16`; its payload was not read. The native frame is not identified as F06OF `SCP16`. No native F06OF generation token, input cursor, EOF marker or three-fresh-zero callback match was captured. The three downstream zero-length DPLM finalizer submissions remain separate from frontend callbacks and are not joined by event order.

This supplement adds native-only callchain and owner-path evidence. It does not close the F06OF packet join, change product policy, or claim a model/DEE-equivalent type. No raw PCM, control-row payload, raw pointer/address, code or EC-3 artifact is included. Production OAR/F06OF execution, receipt integration and whole-chain readiness remain unverified.
