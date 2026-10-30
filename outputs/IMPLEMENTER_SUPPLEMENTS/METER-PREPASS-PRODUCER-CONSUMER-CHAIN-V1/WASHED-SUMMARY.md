# Selected Meter prepass producer-to-consumer chain

Disposition: APPROVED_CONTRACT_CHAIN_MAPPED; ACTUAL_PREPASS_NOT_RUN. This is an operational map of approved contracts and runtime gaps, not a production execution result.

## Source binding and verification

Source package manifest SHA-256: 8a3b5e22fa5e6ec4867c7c6b2cd4cba4cd1da9cab4fb3ed8625a2b6a6811b3c1.
PREPASS-CHAIN.json SHA-256: 1e452acc0ff9afe8e30a35c81f65ad6b7e31eac6a4003265f0216c9d12b09701.
INPUT-ONLY-VECTORS.json SHA-256: 64a7728de5069f85c3636922c5a54fdaaac9dc9d4c727fa706de4cc4b0bd217a.
The source checker passed normal and optimized Python. Wash rechecked the manifest and payload hashes. The package contains no implementation source, raw audio, or private material.

## Required order

The approved chain is: input admission; F01/F03/F04/F05 frontend rows; OAR; shared DV32; Q normal callbacks plus three fresh-zero meter drains; 3104-sample latency trim; the same complete 1024-sample blocks into sample/true-peak and K; ordered 960-sample EnergyRecords into integrated and ungated owners; terminal getters/finalize; immutable MeasurementReceipt; then one GTC METERING-to-READY commit. Main cached drains and the seven post-prepass GTC owners are separate.

The 14 D11 crosswalk entries previously classified as receiver/QMF/JOC, OAMD metadata, or JOC score are not Meter-prepass numeric inputs under their approved owner contracts.

## What is statically available and what must be generated

Selected numerical rules/assets are identified: OAR 32 I32 words; DV32 eight parameters plus the 32-word kernel; K matrix/state bounds; authorized true-peak values; log constants; and fixed F06I/U rules. Several local checks cover these constants or synthetic cases. The selected D11 static register is exhaustive; no new selected static numeric constant gap is claimed.

The Q+3 execution still must produce the actual admitted input identity/extent/cursor, same-callback controls and six-lane render, OAR output, warm DV32/K/true-peak/energy states, valid spans, ordered EnergyRecords, terminal metrics/digest, and receipt. Static cold assets and synthetic vectors are not these runtime values.

## Three concrete gaps

1. F06 whole-composition producer/executable: no single selected composition yet runs the complete meter call order, stage state and atomic transition into main none-DRC.
2. Real selected input to rendered meter lanes: the approved frontend/OAR/DV32 pieces lack one fresh admitted-ADM execution that yields the exact same-generation six-lane stream across normal callbacks and drains. F06OF drain metadata installation is unresolved.
3. Cross-owner failure transaction: local modules define fail-stop, but whole-chain tests must prove zero-commit boundaries for callback commit, meter finalize commit, and main-source commit.

The current meter source-owner key/extent/cursor and terminal receipt remain unbound/absent. No provider digest, prepass, terminal getter, READY, output, or Gate1 pass is claimed.
