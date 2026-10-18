# CurrentFeatures provider mapping

Status: P1.4-C mapping only. The two provider validators and CurrentFeatures producer are not implemented in Rust.

This mapping binds the washed `current-features-producer.math-route.v1` supplement to the two exact selected math providers. It does not implement the feature algorithm, claim native/SIMD equivalence, or claim product audio acceptance.

## Identity and scope

The approved package identity is `MANIFEST.json`, 76,302 bytes, SHA-256 `c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`.

The washed supplement is `current-features-producer.math-route.v1`, manifest 1,576 bytes, SHA-256 `dcff6c7e009ba1cd5e18a03268eecfc2ac1abeceea319f84830978e68b3271ea`. Its allowlisted files are:

| File | Bytes | SHA-256 |
|---|---:|---|
| `SPEC.md` | 14,977 | `1c298e21d81a702f8298dabba3f28e01576c9602729f5746d6665c0becfdeaa5` |
| `API-SCHEMA.json` | 1,291 | `6841baa2f7435529316b16388c3a106fadebab7cff3b8f8ecdf394963cdd27d6` |
| `INPUT-ONLY-LITERAL-VECTORS.json` | 315,386 | `1aaf1f5296569960e188b8d74fc8097b5bfb6099b5a1079366e595011c87bdc3` |
| `FFT-NUMERICAL-QUALITY.json` | 3,028 | `f17440c8ce7534de6baafdee1077c7d7b5d0e8f2fbeaebb30341c018329ccd8d` |

The boundary checker reads only the supplement manifest and these four listed files. It does not follow `research_basis` references or open research-source files.

The selected FFT route is exactly `REAL_FFT4096_RADIX2_MATH_V1`. The washed spec selects the mathematical radix-2 FFT; it does not require or claim bit-exact equivalence to the former SIMD route. No alternate FFT route, fallback, or runtime route selector is registered.

## Exact providers

Both provider files are bound to a unique entry in the authenticated package manifest. Each file has one exact byte length and SHA-256. Those complete-file identities, plus typed shape validation, are the provider identities.

| Provider | Package path | Bytes | SHA-256 | Schema ID | Selected value |
|---|---|---:|---|---|---|
| Feature processor math | `parameters/feature-processor-math-v1.json` | 35,441 | `c2cc80eed698285a7a022324bfed36e04c3b6e682cb21674b6316b3ec6a2ec0b` | `feature-processor-math-parameters.v1` | `MATHEMATICAL_L3072_FS48000_GENERATION` |
| FFT roots | `parameters/feature-fft4096-math-v1.json` | 279,305 | `aea85e4fb5dcb0354ac11790466d324f1020a8bf5db3f9d981bc46755751fb33` | `feature-fft4096-math-parameters.v1` | `REAL_FFT4096_RADIX2_MATH_V1` |

The `.v1` suffix is the version carried by each schema ID; the JSON has no separate numeric schema-version field. Both assets set `real_ADM_new_route_acceptance` to `NOT_RUN`.

`schema_id_version_annotation` in `IMPLEMENTATION-MAP.json` is documentation metadata derived from the schema ID suffix; it is explicitly not a provider JSON field. The coefficient and root-word arrays are not duplicated in this map: the complete authenticated package assets remain the sole source of their exact words. Do not regenerate those words or substitute generated/compiled tables.

### Feature processor math provider

The exact top-level field set is:

`schema`, `source`, `window_length`, `sample_rate`, `fft_length`, `component_count`, `vector_count`, `coefficient_u32`, `window_counts`, `fast_decay_u32`, `slow_retain_u32`, `real_ADM_new_route_acceptance`.

| Field | Typed mapping | Exact contract |
|---|---|---|
| `window_length` | integer | 3,072 |
| `sample_rate` | integer | 48,000 |
| `fft_length` | integer | 4,096 |
| `component_count` | integer | 2,048 complex bins, `k=0..2047` |
| `vector_count` | integer | 24 bands |
| `coefficient_u32` | `U32[2048]` | Each word is one F32 coefficient bit pattern; exact values are bound by provider file SHA-256. |
| `window_counts` | integer array `[24]` | `[4,5,6,8,9,11,14,17,20,25,30,37,45,54,66,80,97,118,144,175,213,259,315,296]`; sum 2,048. |
| `fast_decay_u32` | U32 F32 bits | `0x3F06FCA3` |
| `slow_retain_u32` | U32 F32 bits | `0x3F4ED1AB` |
| `source` | exact string metadata | `MATHEMATICAL_L3072_FS48000_GENERATION` |
| `real_ADM_new_route_acceptance` | status metadata | `NOT_RUN`; not a runtime parameter. |

`schema` is the exact schema ID in the provider table. Dimensions, arrays, and scalar bit words must not be regenerated from platform `libm` or replaced with inline constants.

### FFT roots provider

The exact top-level field set is `schema`, `route`, `source`, `fft_length`, `stages`, `real_ADM_new_route_acceptance`.

| Field | Typed mapping | Exact contract |
|---|---|---|
| `schema` | exact string | `feature-fft4096-math-parameters.v1` |
| `route` | closed route identity | `REAL_FFT4096_RADIX2_MATH_V1` |
| `source` | exact string metadata | `MATHEMATICAL_ROOT_GENERATION_NO_COMPILED_TABLE_INPUT` |
| `fft_length` | integer | 4,096 |
| `stages` | array `[12]` | Exact stage lengths `[2,4,8,16,32,64,128,256,512,1024,2048,4096]`. |
| each `stages[i].length` | integer | Must equal the corresponding stage length above. |
| each `stages[i].roots_u32` | array `[[U32;2]; length/2]` | Each pair is `(cos, sin)` F32 bit words for `-2πj/length`, in increasing `j`. The exact words are bound by the complete provider SHA-256. |
| `real_ADM_new_route_acceptance` | status metadata | `NOT_RUN`; not a runtime parameter. |

Across the 12 stages the provider contains 4,095 complex root pairs, or 8,190 U32 words. The validator must check each stage length, pair count, pair width, and U32 range before returning the typed value.

## Identity-first validator boundary

The future exact validators reuse `PackageManifestEvidence` and `ProviderFileEvidence`; they do not read files in `domain` and do not introduce a generic parameter platform.

1. Recompute and verify the complete package manifest byte length and SHA-256 before parsing it.
2. Parse the authenticated `files` array and require exactly one entry for each provider path, with the exact byte length and SHA-256 above.
3. For each `ProviderFileEvidence`, recompute observed length and SHA-256 from the actual bytes and compare them with the unique authenticated manifest entry before UTF-8 or JSON parsing.
4. Parse only after identity matches. Reject malformed UTF-8/JSON, duplicate keys, missing or unknown fields, booleans/floats in integer fields, wrong array shapes, out-of-range U32 words, wrong schema, dimensions, route, stage lengths, and root pair counts.
5. Return a private typed provider and immutable receipt. No error returns a partially validated provider or receipt. `diagnostic_path` is for diagnostics only and is never identity.

The provider files contain no canonical preimage, canonical byte count, or canonical digest. Therefore their identity is the authenticated complete file identity plus typed schema/shape validation. Do not invent a projection digest, canonical writer, or semantic hash for either provider.

## Typed outputs, receipts, and errors

The minimal validator outputs are two distinct private-field types: `ValidatedFeatureProcessorMathProvider` and `ValidatedFeatureFft4096MathProvider`. Both expose only read-only typed accessors and never expose `serde_json::Value`.

The corresponding immutable receipt shapes are internal API mappings, not provider JSON fields or native receipts:

| Receipt | Fields |
|---|---|
| `FeatureProcessorMathProviderReceipt` | `validated_manifest_lineage`, `provider_file_sha256`, `provider_file_bytes`, `schema_id` |
| `FeatureFft4096MathProviderReceipt` | `validated_manifest_lineage`, `provider_file_sha256`, `provider_file_bytes`, `schema_id`, `route` |

`validated_manifest_lineage` proves the expected full package-manifest identity and that provider's unique entry. There is no provider canonical digest or generation because these are immutable package assets, not mutable state owners.

The provider validator error categories are separate from the washed consumer-vector labels. Reuse the existing F01E/F03 identity-first error pattern:

| Error case | Typed error variant | Category |
|---|---|---|
| Observed manifest bytes disagree with their evidence | `ObservedManifestEvidenceMismatch` | `PARAMETER_IDENTITY` |
| Complete manifest SHA mismatch | `PackageManifestIdentityMismatch` | `PARAMETER_IDENTITY` |
| Manifest UTF-8/JSON syntax failure after expected identity gate | `PackageManifestSyntax` | `INPUT_REJECTION` |
| Missing, duplicate, or mismatched exact provider entry | `PackageManifestEntry` | `PARAMETER_IDENTITY` |
| Provider observed length/SHA disagrees with authenticated entry | `ObservedProviderEvidenceMismatch` | `PARAMETER_IDENTITY` |
| Provider path, bytes, or SHA disagrees with authenticated entry | `ProviderIdentityMismatch` | `PARAMETER_IDENTITY` |
| Provider UTF-8/JSON syntax failure | `Utf8` / `JsonSyntax` | `INPUT_REJECTION` |
| Schema or exact field set mismatch | `Schema` | `PARAMETER_IDENTITY` |
| Integer range or array shape mismatch | `Shape` | `PARAMETER_IDENTITY` |
| FFT route mismatch | `RouteMismatch` (FFT provider only) | `PARAMETER_IDENTITY` |

Each failure returns no validated provider or receipt. These are proposed isolated Rust API outcomes following P1.4-A/B; they do not claim native error names. The nine supplement negative-case labels are likewise research test API classifications, not native error codes.

## CurrentFeatures consumer binding

The future producer must accept the two validated typed provider outputs, not raw evidence or an unverified JSON value. Its same-callback context is the committed before feature state (`F32[48][24]` plus `F32[48]`), same-callback `SourceFlagsPacket` (`U8[48][2]` source-major), and post-window audio (`F32[48][3072]` source-major) with render, flags, profile, provider, programme, attempt, transaction, callback, and before-state lineage.

The lineage dimensions are programme, attempt, `CallbackTransactionId`, callback index, before-feature generation/digest, render receipt, flags receipt, selected profile, and both provider receipts. The consumer keeps its after-state staged; it has no local commit, and the parent transaction remains the commit authority.

The pair order is `(processing_inactive, selection_suppressed)`: `01` maps to RESET before any carry test; `10` maps to CARRY; `00` maps to READ_AUDIO_INPUT. RESET and CARRY do not read audio; READ_AUDIO_INPUT requires all 3,072 samples. The mathematical consumer includes a `01` priority vector, but the validated SourceFlags producer emits only `00`, `10`, and `11`. Whether the product boundary rejects an unreachable `01` packet is explicitly left to the coordinator contract; do not infer it from the math branch.

The washed input-only vectors contain 15 positive cases and 9 error cases. Their three positive outputs each contain 4,608 bytes of after-vector, 192 bytes of after-scalar, and 4,608 bytes of band-major packet data: `15 × (1,152 + 48 + 1,152) = 35,280` F32 words. The required bitwise relation is `packet[band][source] == after_source_vector[source][band]` by U32 bits. The nine negative vectors require a null packet and zero state commit count.

The separate numerical-quality file reports five synthetic FFT comparisons to the first 2,048 bins of a NumPy float64 mathematical rfft; maximum sampled relative L2 error is `1.2336887697203963e-7`. It states no universal error bound, has no native reference, and marks perceptual/real-ADM acceptance `NOT_RUN`.

The nine input-only negative cases use research-side labels such as `READ_AUDIO_MISSING` and `NONFINITE_INTERMEDIATE`; they are not native codes. Their expected result is no packet and `state_commit_count=0`.

## Readiness boundary

`P1.4-C` is mapping-only complete; neither typed validator exists in Rust. `CURRENT_FEATURES_PRODUCER` therefore remains `BLOCKED_ON_P1.4_C_PROVIDER_CONTRACTS`. This map is sufficient to implement the two exact provider validators as a separate future slice, not permission to implement the CurrentFeatures algorithm now.

`SourceFlagsPacket` remains `LOCALLY_VERIFIED_NUMERIC_AND_PACKET`. P1.3-D1 remains `LOCALLY_VERIFIED_STORAGE_AND_STAGING_FOUNDATION__PACKET_PRODUCERS_DEFERRED`. P1.3-D remains blocked on CurrentFeatures, the `spatial_audio` root, and the enclosing `CallbackTransaction`. Product FFT/feature execution, full provider-loader integration, real-ADM route, codec/EC3, perceptual acceptance, and parent transaction execution remain `NOT_RUN`.
