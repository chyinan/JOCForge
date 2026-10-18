# F04 remaining provider validation map

Mapping status: `MAPPING_COMPLETE`. Validator status: `NOT_STARTED`. This is the
P1.4-D2 boundary for identity-first, typed validation of three F04 parameter
assets. It does not implement QMF, FFT, descriptor, receiver, audio, state, or
rendering logic.

## Package identity and validation order

The approved package uses the existing single
`crates/jocforge-domain/tests/fixtures/f01e_provider/MANIFEST.json` fixture
(76,302 bytes, SHA-256
`c7cddb7d5ac073c0f410fe44be9a431cfbb21c3df469d8b43c32a4297298674a`). Do not
add another manifest fixture. Each provider has one exact manifest entry. The
validator order is: verify the complete manifest bytes and SHA before parsing;
require exactly one entry for the selected provider path; compare observed
provider bytes and SHA with that entry and the pinned map; only then decode
UTF-8 and parse JSON. Paths are diagnostic, not identity.

The validator fixtures will be byte-for-byte copies of the three pinned files.
Providers will use private typed storage and immutable read-only receipts;
`serde_json::Value` does not cross a public interface. No provider projection
digest or canonical writer is defined. U32 words are raw IEEE-754 binary32 bit
patterns: parse as unsigned JSON integers, reject booleans/floats/out-of-range
integers, reinterpret only for finite-value checks, and preserve the original
bits. Do not regenerate or replace supplied words.

## Pinned assets

| Asset | Package path | Bytes | SHA-256 | Schema / route | Qualification retained |
|---|---|---:|---|---|---|
| QMF receiver functional | `parameters/qmf-receiver-functional-v1.json` | 20,325 | `30e63cc110b63dd88118ad37f02750a39167bd01279a3b40853afdf3151b552c` | `qmf-receiver-authorized-functional-parameters-v1` | `delivery_scope` is internal isolated implementation only; ordinary non-secret functional constants are authorized; vendor-secret credentials are excluded. |
| QMF mathematical FFT | `parameters/qmf-mathematical-fft-v1.json` | 6,983 | `e8d3fb1514a5ffd857740222eacc8995208a29c3447e48c1b63083af368c0431` | `qmf-mathematical-fft-parameters.v1` / `QMF_FFT_RADIX2_MATH_V1` | Source is `MATHEMATICAL_GENERATION_NO_COMPILED_VALUES_USED`; `audio_acceptance=NOT_RUN`. |
| Descriptor mathematical | `parameters/descriptor-mathematical.json` | 4,861 | `a493fbdaf2dab30e940cae9976ed3305f3af083e5614ab0cbd6889c71a02b197` | `descriptor-mathematical-parameters.v1` | Gain comparison is `PASS_ALL128_WORDS`; threshold comparison is `36_WORD_DIFFERENCES_NOT_IDENTICAL`; threshold route is `MATHEMATICAL_CANDIDATE_AUDIO_ACCEPTANCE_PENDING`; no vendor key is included. |

The slice touches provider clauses for F04Q, F04QF, F04R, and F04D; that is not
an implementation of those functional algorithms. The receiver functional
provider maps to `specs/04-qmf-receiver-whole-binding.md`,
`specs/04-qmf-prototype-source-and-loading.md`, and
`specs/04-descriptor-control.md`. The FFT provider maps to
`specs/04-qmf-mathematical-fft.md` and
`specs/04-qmf-receiver-whole-binding.md`. The descriptor provider maps to
`specs/04-descriptor-control.md` and
`specs/04-qmf-receiver-whole-binding.md`.

### QMF receiver functional provider

Require exactly these top-level fields, with no unknown or duplicate fields:
`schema`, `delivery_scope`, `authorization`, `not_included`, `arrays`,
`scalar_u32`, `generated_providers_reused`, `counts`.

`arrays` contains exactly five records. Every `u32_words` element is a JSON
unsigned 32-bit integer representing finite F32 bits; each array preserves its
listed order and raw words. Verify the declared F32LE digest against the words
without substituting a generated provider.

| Array field | Words | Source classification | Consumer | F32LE SHA-256 |
|---|---:|---|---|---|
| `qwin_synthesis_640` | 640 | `PUBLIC_STANDARD_FUNCTIONAL_VALUES_WITH_VERIFIED_EXISTING_COPY` | F04Q synthesis prototype; analysis is the exact reverse and is not a second delivered array. | `870d3c80e4fe7ee368b5da1f7db487dfa84be44c4cf8c82813b78fb06a8e94a2` |
| `band0_real_mode1_21` | 21 | `AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS` | F04Q mode=1 audio surround band0 FIR; mode=0 fitting history update contract. | `b352accaefeb0ff0bac1981dcbe9112ff9bd9437e14f1662076b41cf9fd70c70` |
| `band0_imag_mode1_21` | 21 | `AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS` | F04Q mode=1 audio surround band0 FIR; selected mode=0 fitting does not multiply taps. | `946593788adebf7dd8beafb057c77e167e5b066607306d2a4e31d99244b8b3be` |
| `lfe_post_filter_4x5` | 20 | `AUTHORIZED_NONSECRET_FUNCTIONAL_CONSTANTS_BOUND_TO_FROZEN_BITEXACT_MODEL` | F04R lane3 four-section pair-order post-filter. | `ddd163cfb2116be81bf283746b3faf38fe771b0d01cee1bf31ea3b0fd04a16eb` |
| `descriptor_threshold_127` | 127 | `AUTHORIZED_NONSECRET_COMPILED_AUDIO_FUNCTIONAL_CONSTANTS` | F04D strict lookup for ordinals 0..126. | `5ba787b1ee814e20bbc86290eca1daa6af33968e4e2458b5a3011c87a78fceac` |

`scalar_u32` contains exactly 12 finite F32 bit patterns, with no arrays or
decimal-F32 substitution: `preconditioning_alpha`, `preconditioning_beta`,
`preconditioning_tiny_strict`, `accumulator_magnitude_threshold`,
`accumulator_delta_threshold`, `accumulator_smoothing`,
`descriptor_level_offset`, `descriptor_level_linear`,
`descriptor_level_quadratic`, `descriptor_headroom_step`,
`descriptor_result_scale`, and `descriptor_rise`. The declared delivered totals
are 829 array F32 words and 12 scalar F32 words; 21 unused mode-other imaginary
words are excluded. `generated_providers_reused` must bind
`QMF_FFT_RADIX2_MATH_V1` to `parameters/qmf-mathematical-fft-v1.json`,
`descriptor_gain_PG03` to `parameters/descriptor-mathematical.json`, and
`receiver_blend32_PG01` to `parameters/window-q15.json`.

The provider's excluded categories are vendor-secret credentials,
authentication secrets, research source, raw original table artifacts,
compiled images, and addresses. Preserve its authorization and delivery-scope
strings in the receipt. This receipt establishes provider identity and typed
field validation only, not receiver/QMF/DSP acceptance.

### QMF mathematical FFT provider

Require exactly `schema`, `route`, `source`, `stages`, `rotation_u32`, and
`audio_acceptance`. Require the pinned schema, route, source, and acceptance
strings above. `stages` has exactly six records in order with lengths
`[2,4,8,16,32,64]`; each record has exactly `length` and `roots_u32`. Root
counts are `[1,2,4,8,16,32]`, and each root is exactly two U32 F32 bit patterns
(real, imaginary), both finite. `rotation_u32` is exactly 128 U32 F32 bit
patterns, all finite. These are supplied generated words; validation does not
compute roots, rotations, FFTs, or QMF outputs. The receipt retains route,
source, and `audio_acceptance=NOT_RUN`; it is not a math or audio PASS.

### Descriptor mathematical provider

Require exactly `schema`, `origin`, `gain_formula`, `gain_u32`,
`threshold_formula`, `threshold_u32`, `gain_research_comparison`,
`threshold_research_comparison`, `threshold_route_status`, and
`private_vendor_key_included`. Require the pinned schema and metadata strings
above, `private_vendor_key_included=false`, 128 gain words, and 127 threshold
words. Both arrays contain U32 raw finite F32 bits. Gain values must be
positive, as required by F04D. Preserve the exact gain/threshold comparison and
route qualification strings in the immutable receipt. The 36 threshold word
differences and pending audio acceptance are not to be normalized away or
reported as descriptor/audio acceptance.

## Validator modules, receipts, and tests

Planned production modules:

- `crates/jocforge-domain/src/spatial/qmf_receiver_functional_provider.rs`
- `crates/jocforge-domain/src/spatial/qmf_mathematical_fft_provider.rs`
- `crates/jocforge-domain/src/spatial/descriptor_mathematical_provider.rs`
- `crates/jocforge-domain/src/spatial/mod.rs`

Planned integration test:
`crates/jocforge-domain/tests/f04_remaining_provider_validators.rs`.
Fixtures live beside the existing D1 F04 parameter fixtures and are exact file
copies; the manifest remains shared. Each asset has its own private typed
provider and receipt with manifest lineage, file SHA/length/schema, and only its
source qualification fields. No shared receipt may erase differences among
the three asset qualifications.

Tests and boundary mutations cover: manifest/provider identity before parse;
unique exact manifest entries; exact root/nested fields and schemas; route and
stage-shape binding; scalar-versus-array classification; unsigned integer
versus boolean/float rejection; finite raw F32 words and raw-bit preservation;
per-array count and digest; generated-provider crosswire/substitution;
descriptor status retention and no readiness overclaim; no public
`serde_json::Value`; no file/process/environment I/O in domain; no FFT, QMF,
descriptor, DSP, audio, or render implementation; no second manifest fixture;
and no public/third-party distribution claim.

This slice does not implement any provider consumer. QMF receiver processing,
FFT/QMF math execution, descriptor control, fitting, render/fanout, parent
transaction behavior, real ADM execution, and product/listening acceptance
remain `NOT_RUN` or deferred under their own gates.
