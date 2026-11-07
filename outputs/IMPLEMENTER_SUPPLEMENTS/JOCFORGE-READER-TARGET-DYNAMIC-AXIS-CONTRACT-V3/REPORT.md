# ReaderTarget dynamic source-axis contract — current worktree snapshot

Wash rebuilt this contract from a fresh 12-file source hash snapshot after noticing V1 was unsealed and V2 had become stale during active implementation. The current snapshot records partial progress: `active_source_ids` is now carried in parts of production callback and SpatialControl, while the shared backing source axis is still fixed at 48.

The implementer reports the dynamic F01E owner stage now passes a synthetic 36/38 transition test, but the first selected Meter 36-row failure is at F03W feature staging: `installed48_frame.rows().len() != 48` returns `MeterFrameIdentity`. Independently, source inspection still finds fixed 48 requirements in installed audio, component matrices, ReaderTarget flags/completeness, SourceFlags, CurrentFeatures, PostWindowFanout, and SpatialControl state. The ReaderTarget completeness boolean is derived from a 48-row geometry count and does not carry presence keys.

For the selected 10-bed/one-LFE profile, one row-shaped pipeline should preserve physical N, regular R=N−1, global source S=N+10, and target T=15. For N=36, S=46 and IDs 46/47 do not exist; do not fill them with default flags or zeros. Preserve one ordered `CodecSourceIndex` sequence and owner/callback identity across every axis consumer.

DEE's separate profile count C is not mapped to JOCForge S here. The 36/38 equality S=N in the sampled 10-bed vectors must not be generalized into a cross-product count identity. This is an implementation contract and source snapshot only; no 36-row full encode or audio parity is claimed.
