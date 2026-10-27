# F11 converter to F12 plan handoff audit

Decision: `NO_NEW_SELECTED_MODEL_GAP`.

Scope is the already approved ADM 48 kHz / 1024 kbps / 5.1 model. This addendum binds the selected F11 producer inputs, provider owners, per-source clock, and F12 candidate boundary. It adds no product policy and does not extend the selected profile.

## Input and receipt map

| Input or value | Selected producer and owner | F11 use | F12 use |
|---|---|---|---|
| Profile shape | Selected F01E/config and the frozen profile; the profile fixes `acmod=7`, one programme, RF/dialnorm/mixing field widths, and the line-compression field's presence. | F11.2 uses the selected fixed header shape. `H = 91 + 14 × int(surround_EX > 0)`. The selected line-compression presence contributes its fixed width; the DRC code value is not an F11 search input. | The complete same-source BSI is serialized. |
| `surround_EX` selector | F01E's parsed `authored_source_mode_byte`; F06.4 states host byte 3 selects mode 2, otherwise mode 0. DBMD `warm_trim_mode` is a separate field/domain; the selected warm-trim-3 case maps to host byte 4 and mode 0. | Selects the optional 14-bit header-cost increment. Do not substitute a numerically equal warm-trim value for the authored-source selector. | The selected BSI field is serialized from its profile producer. |
| Analysis/source values | One immutable `SharedAnalysisReceipt` for source `s`, then F08/F09/F10 continue in the same analysis clone. | The first main POWER refresh's saved current rows `S` and the same consumed short/other flags feed independent converter refresh/strategy/pattern selection. Main decoded exponent rows, converter reuse spans, main LFE strategy, and fixed six-block `db` codes feed conversion/costing. F11 does not consume the main frame code or rerun POWER over PRIMARY. | Reads the resulting `NumericPlanReceipt`, including the separate converter codes and SNR offset. |
| F06 BSI/DRC | The F06 consumer reads the same `SharedAnalysisReceipt` digest in a separate DRC clone and produces same-source-generation BSI plus RF/line U16 control words. | F11's selected cost contract accounts for fixed field presence/width only; no RF/line numeric code is defined as an F11 search input. In the selected successful F06ND path RF and all six line-presence flags are 1. | Receives the complete same-source BSI/DRC. The F06ND→F12 map binds RF and six line codes, including their presence and high-byte projection. |
| F13 EMDF or absent payload | F13's four-queue producer and publication gate run after F11's numeric plan. | Not an F11 input. F11's `auxiliary absent` cost of 1 is a separate virtual-budget field; the F11 contract explicitly says EMDF skip is not this auxiliary. | Receives the current source's complete EMDF or `ABSENT_PUBLISHED_PAYLOAD_V1`; sources 0 and 1 use absent payload after the F13 queue transaction. |

The fixed-profile/header and same-source receipt roles are therefore distinct. F11's cost model needs only the selected field shape and the profile-derived optional selector; F12 needs the actual BSI values and F13 payload. No value-equality or neighboring-stage inference is used to create a receipt join.

## Clock, state, and failures

- For codec source `s`, the transaction contract verifies `s = g - 2` and all source/generation/provider tokens before work. `SharedAnalysisPrepare` advances F07 once. F06 consumes that receipt; F08/F09/F10/F11 continue the same analysis clone and create one `NumericPlanReceipt` for the source.
- F11 runs once per complete codec source, before F12 candidate trials. Its FBW converter refresh work, markers, and trial costs are fresh scratch; the converter result is a value in `NumericPlanReceipt`, not a persistent F11 owner. The main and converter frame codes remain distinct.
- F12 candidates read the fixed plan, same-source BSI/DRC, codec clone, and F13 payload disposition. A candidate trial creates fresh BAP/mantissa/bitwriter scratch. Candidate failure does not mutate stage owners, and a successful candidate commits once only after complete frame creation and the transaction's durable append gate.
- F11 frame-pattern mismatch is a producer failure. F11 negative costs/search exhaustion use the specified zero-cost fallback and fail only if that fallback is infeasible; these failures occur before F12 candidates. If no F12 candidate succeeds, the source is fatal and staged roots are discarded under the global transaction contract.
- Sources 0 and 1 still form their numeric plan and run F12 with absent published payload; F13 payload suppression does not suppress F11.

## Evidence limits

The approved package and prior washed supplements already contain the needed selected model contracts and provider identities. This audit found no new selected-model contract gap and did not repeat converter math, F06ND presence tracing, or native observation.

`FRESH_SOURCE_TO_FRAME_INTEGRATION=NOT_RUN`; the frozen availability audit explicitly says actual F07/F08/F09/F10→F11→F12 producer execution, production F06/F13 connection, and 4,249-source replay were not run. Full-song, product audio/render, and native parity remain `NOT_RUN`. The existing native-only unbound item remains the metadata-owner to exact F12 validated-count/working-header owner join in one source/reset generation; it is not evidence against the selected model contract.

No source code, disassembly, machine code, addresses, PCM, or keys are included.
