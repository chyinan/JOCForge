# DRC owner Option dialnorm erratum

This supplement replaces only the prior awaiting_measurement/ready outer-tag union. The canonical owners.drc root keeps every F06ND leaf present and represents the measurement-dependent dialnorm as Option<U5>. None is used before measurement finalize; Some(U5) is installed after finalize. No DSP default is introduced.

The complete finalized MeasurementReceipt remains once at the top level of GTC ProgrammeState. The DRC root contains the F06ND-declared immutable programme_dialnorm U5, not a nested receipt or receipt digest. GTC selected_lufs_u32 is interpreted by the approved F06 measurement semantics, and F06.3 derives the U5. If the full F06WH programme_measurement object also carries dialnorm, require exact equality.

Both input-only cold values contain the same complete DRC leaves. They differ only in programme_dialnorm: JSON null versus integer24 derived from the synthetic selected_lufs=-24.0 input. The owner generation remains0 across finalize. The None root hash remains this attempt's cold_sha256; state_sha256 changes to the Some root hash at finalize.

The root uses F06WH-compatible UTF-8 canonical JSON with sorted keys, compact separators, no BOM and no trailing newline. F32 words use exact U32 bit patterns. The canonical value includes complete nested arrays; this supplement does not reuse the combined 13-owner preimage/hash. The two hashes are new standalone root identities calculated from this representation only.

The production root keeps secondary forward/inverse histories, statistics, scheduler, metadata override FIFO and programme_dialnorm. F07 ordinary input history belongs to shared analysis and is passed through SharedAnalysisReceipt. The opaque category5 packet uses its separate path; it does not populate the DRC FIFO. Static coefficient tables remain immutable providers.

GTC state_ref is owners.drc. Generation is the successful source count; source0 and source1 advance DRC while output publication remains suppressed. Callback0/1 do not have a codec source. F06ND prepares next state on the parent transaction clone; the enclosing source/callback root swap is the only commit. Failure discards the DRC after state and requires full programme restart. F06ND adds no EOF flush.

This is a local state representation erratum, not a native serializer or implementation change. Product materialization, GTC receipt execution, DRC source execution and failure/restart integration remain NOT_RUN.
