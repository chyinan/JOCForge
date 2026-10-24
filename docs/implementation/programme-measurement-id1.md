# Programme measurement to ID1

Status: `VALUE_BINDING_LOCALLY_VERIFIED__METER_PREPASS_NOT_RUN`.

The semantic binding uses washed `MEASUREMENT-ID1-SEMANTIC-BINDING-V1` (manifest SHA-256 `0ae40e05767686b600289e35d446e5df9ad2f84a7a69338e62eb3d99e9b1c9eb`); all five payload identities matched.

`ProgrammeMeasurementReceipt` preserves programme/attempt/input/config/terminal identities, finalized dialogue/ITU/true-peak words, selected LUFS, gating kind and selected callback/drain/window counts. Its constructor is crate-private: only the future verified meter-finalize path may mint it. Fixture receipts cannot authorize production READY.

`ProgrammeMeasurementValue` separately retains the nine frozen programme fields required by the meter/main join: dialogue, speech percentage, ITU, ungated, true peak, sample peak, selected loudness, public gating kind and dialnorm. Its canonical digest is SHA-256 over sorted compact JSON with the approved `*_host` keys. The approved nonzero structural value reproduces `8ca4448baf29a9d75141985e64d2f5f4c69f94d16b60e41561564f7699946fe9`. This value is distinct from the 13-field GTC receipt.

Gating accepts exactly `UNGATED`, `DIALOGUE` or `LEVEL`, corresponding to public semantic 0/1/2. Internal enum ordinals are different and cannot be cast. ID1 field2 is one only for DIALOGUE. The selected DI-off adapter admits UNGATED or LEVEL and rejects DIALOGUE; it never derives this field from `preferred_downmix_mode`.

The nine fields are `[-2,1,is_dialogue,0,half(ITU),half(dialogue),-512,half(true_peak),-512]`. Half-dB uses the approved R32 order, 2^-23 absence threshold, truncation toward zero and distinct loudness/true-peak clamps. Timer calls always receive a fresh copy of this immutable programme configuration, not the previous timer output.

Local tests cover 22 approved half-dB values, seven dialnorm vectors, exact gating parsing/rejection, generic dialogue conversion, selected DI-off admission, frozen-value canonical digest and the existing source0 metadata receipt. The leaf finalizer consumes actual true-peak/integrated/ungated state plus a crate-private terminal identity. Whole OAR/DV32/K-filter prepass, terminal-root construction, BSI join and whole-song execution remain unrun.
