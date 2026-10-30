# Source-weight candidate identity status

## Wash finding

The 32-bit per-candidate descriptor is a packed metadata descriptor, not a proven global candidate ID. Static analysis establishes shared **local ordinal** flow among the candidate feature location, descriptor, metadata row, selector output, and metadata gather. The selector and gather use that local ordinal.

The source-weight caller passes a matrix view and source count to the consumer. Run 120 binds the caller matrix object to that consumer input for a 24-band, 25-source matrix and records 600 in-helper `powf` calls. It did not capture the candidate list or global candidate IDs.

Therefore the join `global candidate ID → matrix column → weight output` remains `UNBOUND`. No global-ID mapping rule is approved by this package. Do not infer one from local ordinal, matrix column count, descriptor values, or matching hashes.

Run 121 did not reach the native callback: filter initialization/workflow creation failed before the candidate or source-weight paths ran. It adds no identity evidence. No raw feature values, PCM, or candidate IDs were persisted in the source package.

## Status boundary

Wash independently verified the source package manifest and all three payloads. Its ordinary and optimized checkers pass. The two previously washed F03U source-weight package manifests and payloads were also rehashed and match. This supplement records a bounded native lineage result; it does not close the global candidate identity join or whole-chain readiness.
