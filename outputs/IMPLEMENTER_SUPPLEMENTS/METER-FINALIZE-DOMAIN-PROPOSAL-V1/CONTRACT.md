# Meter finalization domain proposal

Decision: WASH_APPROVED_LOCAL_CLEANROOM_BOUNDARY. This is an internal API design derived from the approved GTC, Meter, PreparedJobInput and architecture contracts. It is not a claim about DEE's native API or observed transaction implementation.

## Layer-safe identity input

Application owns the admitted PreparedJobInput, including its immutable resource snapshot, normalized F01E configuration, and parsed ADM graph. Application derives exactly once an immutable, pure-data, Domain-owned PreparedJobInputIdentity from that same admitted object. Keep these nested in the one typed value:
- whole validated RIFF/WAVE ResourceIdentity: input_sha256 and resource_bytes;
- canonical normalized F01E config identity: config_sha256, with raw job XML SHA and selected input label as provenance;
- parsed ADM graph provenance whose parent resource identity is the same ResourceIdentity.

The projection carries no file handle, path, I/O capability, or independent mutable source. Do not pass separate hash/path/graph arguments. This preserves the one-input identity join while keeping Domain independent of Application and IO. The architecture dependency reference is hash-bound in SOURCE-CHAIN.json.

## Proposed Meter-domain operation

Proposed API shape: StandaloneMeterRoot::prepare_finalization(PreparedJobInputIdentity, current GTC programme identity). This method is not claimed to exist. It validates the immutable input identity against the Meter root and current GTC programme_id/attempt/phase/generation, and validates the selected profile and terminal Meter state.

Compute from the same input sample count N at runtime; do not hardcode the selected example:
Q = ceil(N / 1536) normal callbacks; exactly 3 fresh-zero Meter drains; P = Q * 1536; C = floor(P / 1024) * 1024; E = floor(C / 960).
Require the terminal root to contain all committed callbacks, drains, and complete energy windows. Capture SHA-256 of the complete canonical owners.meter value after the last callback commit and before getters. Then read the approved read-only getter sequence: true peak, sample peak, integrated, ungated, DI-off, loudness selector, dialnorm. Getter reads must not mutate the Meter root.

The Meter domain keeps ProgrammeMeasurementReceipt construction private. It constructs the existing receipt fields from the validated parent identity, PreparedJobInputIdentity and terminal Meter/getter values, then returns a typed proposal for the existing GTC MEASUREMENT transaction. No new receipt field, transaction kind, digest algorithm, or DSP rule is introduced. The receipt remains top-level and shared; it is not nested inside the DRC owner. The proposal carries the existing DRC Option<U5>::Some value derived by approved F06.3 from selected LUFS.

## Parent commit

The GTC parent revalidates METERING phase, programme_id, attempt, phase generation and proposal/input identities. One parent root swap installs the existing top-level MeasurementReceipt, retains the terminal Meter root bytes, sets DRC programme_dialnorm to Some(U5) at generation 0, transitions METERING to READY, and increments phase_generation once. Application submits the typed proposal; it cannot mint the receipt, set READY, or commit owners separately.

The approved MeasurementReceipt field set is unchanged: programme_id, attempt, input_sha256, config_sha256, dialogue_u32, itu_u32, true_peak_u32, gating_kind, selected_lufs_u32, meter_terminal_sha256, normal_callbacks, zero_drains, energy_windows.

## Failure and evidence limits

Missing/mismatched identity, incomplete terminal clock, absent getter, invalid result, or stale parent proposal yields no receipt, no DRC Some and no READY commit. GTC retires the attempt; recovery requires full_restart with attempt+1 and fresh metering. This failure behavior comes from the approved local GTC contract, not a claim about DEE rollback.

The attached vectors are synthetic input-only mismatch/counter cases and contain no expected outputs. The source package reports real terminal hash and GTC finalize as NOT_RUN; whole-chain and product acceptance remain NOT_RUN. The proposed API has not been implemented or executed.
