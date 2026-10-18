# F04 component parameter provider mapping

Status: `MAPPING_COMPLETE_READY_FOR_VALIDATOR_IMPLEMENTATION`. This mapping authorizes only the P1.4-D1 identity-first validators. It does not authorize component DSP, receiver/QMF processing, or product acceptance.

## Package identity and permission

The approved package manifest is `MANIFEST.json`, 76,302 bytes, SHA-256 `c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`. Its sole repository fixture is `crates/jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json`; P1.4-D1 reuses that fixture and adds only the two provider JSON fixtures.

Each provider must have exactly one entry in the complete manifest. Recompute and validate the manifest identity before parsing it; then validate the unique provider entry and the observed provider byte length/SHA before UTF-8 or JSON parsing. A path is diagnostic only, not identity. No provider projection digest or canonical writer is defined.

Both selected providers are for internal research and internal implementation only. Public or third-party distribution is not authorized. The window provider excludes the private vendor key and copied raw assets; the component mathematical provider copies no source table. P1.4-D1 must not expand these permissions.

## `window-q15.json`

Identity: 2,704 bytes, SHA-256 `a12f9cbf91f1cf540c2eae77e4d236970defa70409f00ddf62ba87bfeafbb130`; manifest path `parameters/window-q15.json`; provider id `F04_WINDOW_Q15`; kind `GENERATED`.

The exact top-level field set is `schema`, `product_role`, `private_vendor_key_included`, and `parameters`. Require `schema = generated-window-parameters.v1`, `product_role = ALGORITHM_PARAMETER_NOT_SONG_LOOKUP`, and `private_vendor_key_included = false`. `parameters` contains exactly one record for each of these IDs; reject unknown or duplicate IDs:

| Parameter ID | Count | Runtime type | Consumer mapping |
|---|---:|---|---|
| `receiver_blend32` | 32 | `q15_values: [U16; 32]` | Receiver double-end blend, named by PG01. This validator does not implement the receiver. |
| `component_kernel64` | 64 | `q15_values: [U16; 64]` | F04A.8 component filter kernel. |

Each record has exactly these fields: `parameter_id`, `count`, `representation`, `q15_values`, `generation`, `to_F32`, `F32LE_sha256`, `origin`, `all_words_research_comparison`, and `raw_asset_or_source_code_copied`. Require `representation = q15_unsigned_integer`, the record-specific count and array length, and integer values in `0..=32768` (booleans and floating-point JSON numbers are not integers). Require the declared generation `RNE_integer(32768*sin((j+1/2)*pi/(2*N))^2)`, conversion `R32(q15_value/32768), exact`, the exact 64-character lowercase F32LE digest string, `origin = MATHEMATICAL_FORMULA_GENERATED_THIS_TASK`, `all_words_research_comparison = PASS`, and `raw_asset_or_source_code_copied = false`.

PG01 defines `N=32` and `N=64`, `j=0..N-1`, real-math sine, integer round-to-nearest-even, exact binary32 conversion, and symmetric complement sum 32768. The file carries Q15 arrays and F32LE identity digests, not F32 word arrays. P1.4-D1 preserves those Q15 values and digests; it does not regenerate or hardcode F32 words. The window values are N-bound, not independently sample-rate-bound by this file.

## `component-parameter-mathematical-v1.json`

Identity: 1,077 bytes, SHA-256 `5d0fec6f62a33eeef5c64016760fcbe94a1872a564b6d0e03f959c95c37eb0f5`; manifest path `parameters/component-parameter-mathematical-v1.json`; provider id `F04_COMPONENT_PARAMETER_MATH_V1`; kind `GENERATED`.

The exact top-level field set is `schema`, `route`, `sample_rate`, `block_samples`, `coefficient_rules`, `dt_rule`, `reciprocal_pivot_rule`, `target_pivot_rule`, `envelope_weight_rule`, `q15_integers`, `coefficient_words`, `reciprocal_pivot_word`, `target_pivot_word`, `envelope_weight_word`, `original_design_rationale`, `source_table_copied`, `public_or_third_party_distribution`, and `implementation_ready`.

Require `schema = component-parameter-mathematical-generation.v1`, `route = COMPONENT_FIXED_48K_Q15_PARAMETER_MATH_V1`, `sample_rate = 48000`, and `block_samples = 64`. The eight Q15 integers are ordered `c0,c1,c2,c3,c4,reciprocal_pivot,target_pivot,envelope_weight`; require exactly eight unsigned integer values in `0..=32768`. The five `coefficient_words` strings encode raw binary32 words for `c0..c4`; the three scalar word strings encode `reciprocal_pivot`, `target_pivot`, and `envelope_weight`. Each word is exactly eight lowercase hexadecimal digits. Parse and preserve the bits; require finite binary32 values and the F04P.1 ranges: `c0..c4` and `envelope_weight` in `[0,1]`, `reciprocal_pivot` in `[0.5,1)`, and `target_pivot` in `[0,1]`.

The five ordered coefficient rules and the three scalar generation rules must match F04PG.1-.2 exactly. `dt_rule` is `64/48000 seconds`; `reciprocal_pivot_rule` is `ceil(32768/sqrt(2))/32768`; target and envelope rules are the declared Q15 round-to-nearest-even formulas. Require `original_design_rationale = not claimed; equivalent generation settings only`, `source_table_copied = false`, `public_or_third_party_distribution = NOT_AUTHORIZED`, and `implementation_ready = NO`. The implementation validates these declarations; it does not execute the generation formulas, regenerate the integers or words, replace them with literals in Rust, or infer an original design rationale.

Bind this provider to the sole supported selected profile: ADM BWF / 38-channel PCM24 / 48 kHz; 5.1 six-carrier; DEE selected mode0/phase1/index3; one independent and zero dependent substreams; 1024 kbps, six blocks, 1536 samples/AU, 4096 bytes/AU; `deployed_compat` default and `etsi_emdf` optional. The F04P component sub-profile is 15 shared rows, 64-sample block, and 48 kHz. Validate the existing typed selected-profile input; do not introduce a profile variant or bind to `RenderTarget`.

### Readiness qualification

The provider-local `implementation_ready = NO` is retained and validated. Wash-side semantic clarification dated 2026-10-18 states that this field disclaims component DSP, full F04 consumption, real ADM, listening, and whole-product readiness; it does not prohibit parsing, validation, or construction of a typed provider. Package-level `IMPLEMENTATION_READY = YES` permits the registered internal slice and does not override the provider-local declaration.

The component receipt must carry:

- `asset_implementation_ready = false`;
- `authorized_scope = INTERNAL_SELECTED_PROFILE_PROVIDER_VALIDATION_AND_FUTURE_CONSUMPTION`;
- `dsp_consumption_status = NOT_YET_VALIDATED`.

The receipt proves only identity and typed provider validation. It is not a DSP or product PASS.

## Validator boundary

The two validators accept immutable package-manifest and provider-file evidence. Domain code has no file, process, or environment access. JSON parsing is permitted only after both identity gates; outputs use private typed fields and immutable read-only receipts, with no public `serde_json::Value`. Since neither provider has a canonical preimage, do not compute a projection digest.

Planned files:

- `crates/jocforge-domain/src/spatial/window_q15_provider.rs`
- `crates/jocforge-domain/src/spatial/component_parameters_provider.rs`
- `crates/jocforge-domain/tests/f04_parameter_providers.rs`
- `crates/jocforge-domain/tests/fixtures/f04_parameters/window-q15.json`
- `crates/jocforge-domain/tests/fixtures/f04_parameters/component-parameter-mathematical-v1.json`

The test fixtures must be byte-for-byte copies of the two pinned providers. Do not add a second `MANIFEST.json`. P1.4-D1 excludes component DSP, receiver/QMF processing, the other F04 providers, rendering, parent transaction behavior, real ADM execution, and product acceptance. P1.3-E remains blocked on those broader prerequisites.
