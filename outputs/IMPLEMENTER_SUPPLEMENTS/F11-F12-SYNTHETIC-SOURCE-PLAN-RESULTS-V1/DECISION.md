# F11 source plan to F12 handoff audit

Status: `APPROVED_MODEL_HANDOFF_CROSSWALK_COMPLETE; FRESH_RECEIPT_EXECUTION_NOT_RUN`.

This is a path-free Wash addendum for the approved 48 kHz / 1024 kbps / 5.1 source model. It records the existing hash-bound source-0/1/2 F11 outputs and their F12 plan/frame rows, then maps them to the approved `NumericPlanReceipt` contract. It does not change product policy or create a new state owner.

## Existing source rows

`F07-F12-SOURCE-COMPOSITION-INPUT-VECTORS.json` and `F07-F12-SOURCE-COMPOSITION-VERIFICATION-RESULT.json` already carry three successful synthetic source rows. In each result row, `composition.real_leaf_evidence.f11` contains five main codes, five converter codes, and the converter SNR offset. That row's `plan_sha256` equals its F12 best candidate's plan hash. The input and result artifacts, plus their hashes, are listed in `SOURCE-IDENTITIES.json`.

The three recorded outputs are:

| Source | Main codes | Converter codes | Converter SNR offset | F12 best | Published AU |
|---:|---|---|---:|---:|---|
| 0 | `[0,1,2,3,4]` | `[16,17,18,19,20]` | 341 | q=422; 4096-byte frame; CRC residual 0 | suppressed |
| 1 | `[5,6,7,8,9]` | `[21,22,23,24,25]` | 414 | q=422; 4096-byte frame; CRC residual 0 | suppressed |
| 2 | `[10,11,12,13,14]` | `[26,27,28,29,30]` | 487 | q=422; 4096-byte frame; CRC residual 0 | AU 0 |

The source-composition result records these as local synthetic-model execution. It is not a fresh ADM run, native capture, or producer-to-product acceptance. Equal plan hashes across rows are content equality only; source identity comes from the row's explicit source index and transaction contract.

The new F11/F12 approved-source-leaves supplement was rehashed 14/14. Its referenced crosswalk resolution was also replayed against the approved package and washed supplements: all 25 source identities match their recorded hashes (12 approved-package leaves, 4 codec-allocation leaves, 4 F06ND→F12 BSI leaves, and 5 F11/F12 availability leaves).

## Receipt mapping

The frozen cross-stage name is `NumericPlanReceipt`. It references the canonical digest of the same-source `SharedAnalysisReceipt`, carries the completed six-block plan including converter results and provider identities, and is consumed by F12 together with the codec clone, same-source BSI/DRC, and the F13 payload disposition. `SelectedSourceNumericPlan` is not a separately named approved receipt in the sources checked here; if the implementation uses that name, map it as an adapter view of `NumericPlanReceipt` and preserve its source/digest/generation checks.

The source-composition result gives each source's before/after owner-generation counters: all five recorded counters (input, transient, transform, policy, codec) advance 0→1 for source 0, 1→2 for source 1, and 2→3 for source 2. F11's values and F12's chosen plan are nested under the same source row. The saved result does not contain the canonical `SharedAnalysisReceipt` digest or a typed receipt-generation token. `SOURCE-NUMERIC-PLAN-VECTORS.json` preserves the row-level outputs; `INPUT-ONLY-GENERATION-VECTORS.json` carries symbolic same-generation and mismatch inputs without expected outputs. Neither file relabels the symbolic tokens as captured IDs.

The callback clock is a separate domain: the global transaction contract maps codec sources 0/1/2 to callback indices 2/3/4 using `s=g-2`. These callback indices are not owner-generation counters or receipt identities.

## Converter cost boundary

F11.1–F11.4 already define the converter's input shapes, selected-profile cost rules, feasibility procedure, search, and failure behavior. The saved source-composition result persists the final converter codes and SNR offset, but not literal `S`/decoded-exponent arrays or per-source intermediate `H`, `F[0..5]`, `M[0..5]`, residual, strategy/group, or BAP-histogram values. The approved plan handoff names the converter result, not a separate cost-trace receipt. Those per-source intermediate values are `NOT_RECORDED`; this addendum does not derive them from the final offset or matching plan hashes.

F11's `converter_snr_offset` is a 10-bit U16 model value. It is distinct from F12's two-bit `SNR offset strategy` wire syntax value 0. Candidate trials consume the fixed per-source numeric plan; they do not rerun F11. Sources 0 and 1 still build the plan and execute F12 with absent published payload; source 2 publishes AU 0.

## Evidence boundary

- Approved F11, GTC, state-schema, and source-composition contracts: `FROZEN_APPROVED_MODEL`.
- Three F11/F12 result rows and their payload identities: `HASH_BOUND_SYNTHETIC_MODEL_EXECUTION`.
- The 25 resolved source identities referenced by the existing handoff crosswalk: `25/25 REHASHED`; the newly supplied source-leaves package: `14/14 REHASHED`.
- Symbolic same-generation and mismatched-generation vectors: `CONTRACT_DERIVED_INPUTS; NOT_EXECUTED`.
- Fresh source-to-frame execution, F06/F13 production receipts, native execution, full-song replay, and product acceptance: `NOT_RUN`.
- Numeric per-source F11 cost intermediates and a serialized typed `SelectedSourceNumericPlan` receipt: `NOT_RECORDED_IN_THE_APPROVED_FIXTURE`. The approved name is `NumericPlanReceipt`; F11 cost intermediates remain scratch, while converter result and provider identity cross the F12 boundary.

No source code, disassembly, machine code, addresses, PCM, DTHD material, or private key is included.
