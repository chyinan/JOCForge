# CurrentFeatures producer map

Status: `LOCALLY_VERIFIED_MATH_AND_CAPABILITY__RENDER_AND_PARENT_DEFERRED`

This slice implements only the approved `REAL_FFT4096_RADIX2_MATH_V1` consumer and its opaque staged capability. It does not implement render production, the spatial-control after-state composer, a parent transaction executor, selector/outer processing, or a whole-song path. This status is not product audio acceptance.

## Authenticated inputs

The math route is bound to `current-features-producer.math-route.v1` (supplement manifest SHA-256 `dcff6c7e009ba1cd5e18a03268eecfc2ac1abeceea319f84830978e68b3271ea`). The washed `SPEC.md` is 14,977 bytes with SHA-256 `1c298e21d81a702f8298dabba3f28e01576c9602729f5746d6665c0becfdeaa5`; its API schema is 1,291 bytes with SHA-256 `6841baa2f7435529316b16388c3a106fadebab7cff3b8f8ecdf394963cdd27d6`.

The math providers are consumed only as `ValidatedFeatureProcessorMathProvider` and `ValidatedFeatureFft4096MathProvider`. Their actual coefficient/root U32 words remain provider-owned; the producer does not regenerate, embed, or expose them. The FFT quality JSON is sampled synthetic evidence only, not a universal error bound, native comparison, or perceptual criterion.

The exact input-only literal vector file is copied byte-for-byte to `crates/jocforge-domain/tests/fixtures/current_features/INPUT-ONLY-LITERAL-VECTORS.json` (315,386 bytes; SHA-256 `1aaf1f5296569960e188b8d74fc8097b5bfb6099b5a1079366e595011c87bdc3`). Tests derive inputs only from each case's `input`, execute the producer, and compare all expected output bytes afterward: 15 positive cases, 9 negative cases, 35,280 F32 output words.

The product-facing capability contract is `current-features-integration-contract.v1`: manifest SHA-256 `e240701f0128877baefb305152f10553de9288dc05298452d3605011e63603ad`; `CONTRACT.md` SHA-256 `3cf97a638fbe02dc36edf8a29534493e2b7de8947b858f749eefdef636cae0a5`; `API-SCHEMA.json` SHA-256 `280ebef7034797e7efa84e2e3514cbde05ffd973ac644c5f13e804d057b778c6`. This is an internal product API design, not native receipt compatibility.

## Capability and lineage boundary

CurrentFeatures consumes `CurrentFeaturesAudioView`, returned by consuming one linear `PostWindowRenderProposal` through `fanout(self)`. The same call returns `SpatialAudioPostWindowView` for a future spatial-audio assembler. Both views hold the same immutable `Arc<[FiniteF32]>` allocation; they do not copy samples. The proposal binds programme, attempt, transaction, callback, explicit `NORMAL`/`CACHED_DRAIN` phase, selected profile, F01E/F03 provider lineage, spatial-control before identity, typed render-stage identity, typed after-proposal identity, payload byte count, and SHA-256 of exact source-major F32 U32LE bytes.

Only the future `spatial_control/F03W` renderer may call the internal proposal constructor; production construction requires an opaque `SpatialControlRenderAuthority` that only code within `spatial_control` can mint. There is no public raw/default constructor. Tests use a `cfg(test)` render-proposal fixture, then call the real one-shot fanout. CurrentFeatures receives an explicit expected callback phase and rejects a missing proposal before any output or state proposal, including when every source is RESET or CARRY; it never infers phase from zero samples. Successful CurrentFeatures lineage retains the render-stage identity, after-proposal identity, payload SHA-256, and payload byte count. The F04 assembler remains deferred.

The before feature vector/scalar identity is the containing `SpatialControlMachine.generation()` plus SHA-256 of the complete committed v5 root canonical bytes; no feature-only generation or digest is introduced. The producer checks the selected profile and full-root identity, then validates the existing `SourceFlagsPacket` with its crate-private parent-lineage check against the same programme, attempt, transaction, callback, profile, provider lineage, and before root. No new public SourceFlags receipt, JSON wrapper, or digest is added.

The output is one non-Clone `CurrentFeaturesProduced` capability containing the full `FeatureStateProposal` (source-major vector `[48][24]` and scalar `[48]`), the opaque band-major `CurrentFeaturesPacket` `[24][48]`, the existing linear `SourceFlagsPacket`, and immutable copied lineage including both exact math-provider receipts. Packet bits satisfy `packet[band][source] == after_vector[source][band]`. Only a future spatial-control transition composer may merge the proposal. No local commit, abort, fatal, persistent packet, or parent outcome is produced.

Identity, profile, provider, lineage, shape, or preflight mismatches reject before numerical work. A nonfinite or overflowing numerical intermediate returns no output capability or partial proposal; the parent applies the existing GTC fatal/retirement policy. A later parent failure discards the capability. Local Rust error names are not native error codes.

## Fixed numeric route

For each source, flags are `(processing_inactive, selection_suppressed)`: suppression `1` is RESET and has priority; otherwise inactive `1` is CARRY; `00` reads audio. The consumer-vector `01` case exercises reset priority only and does not claim that SourceFlags production emits `01`. RESET/CARRY do not access sample payloads or invoke the FFT. A READ source must have exactly 3,072 finite F32 samples.

READ copies those already shared-windowed values into real slots 0–3071 of a 4,096-point complex work array; the remainder and all imaginary slots start at positive zero. It performs 12 radix-2 stages using the validated provider roots and 12-bit bit reversal. Each butterfly uses independent `r32_mul`, `r32_add`, and `r32_sub` operations; it does not fuse, normalize, re-window, call libm, or add QMF scaling. It consumes natural-order bins 0–2047 only; Nyquist 2048 is excluded.

For each bin, power is `R32(R32(real*real) + R32(imag*imag))`, then weighted by the corresponding provider coefficient. Provider `window_counts` partitions the 2,048 bins into 24 successive groups; each group starts at positive zero and sums bins in ascending order. The forward and backward smoothers use `alpha = R32(0.0001)` and the specified ordered recurrences. The current sum and threshold comparisons are strict `>`; jump ties select observed bits. Blend uses `accept = r32_sub(1, slow_retain)`. Every next scalar is recomputed from positive zero in ascending band order.

The final proposal is built only after every source succeeds. This avoids publishing a partial packet or partial after-state if any later source fails.

## Error-vector mapping

| Literal case | Local outcome | Required result |
|---|---|---|
| `read_missing_audio` | `ReadAudioMissing` | no output capability |
| `read_wrong_audio_extent` | `PostWindowFanoutError::PayloadShape` | no render proposal or consumer capability |
| `source_count_47` | `PostWindowFanoutError::PayloadShape` | no render proposal or consumer capability |
| `wrong_flag_value` | `FlagDomain` | no output capability |
| `before_vector_23` | `BeforeVectorShape` | no output capability |
| `NaN_before` | `NonFiniteBefore` | no output capability |
| `NaN_read_audio` | `PostWindowFanoutError::NonFinitePayload` | no render proposal or consumer capability |
| `stale_callback` | `StaleCallback` | no output capability |
| `finite_audio_intermediate_overflow` | `NonFiniteIntermediate` | no output capability |

All nine expected outcomes specify `packet = null` and `state_commit_count = 0`. The test also verifies the owner generation and root digest remain unchanged.

## Files and deferred work

- `crates/jocforge-domain/src/spatial/current_features.rs` — fixed route, validation, proposal, and packet production.
- `crates/jocforge-domain/src/spatial/post_window_fanout.rs` — one-shot proposal, two typed views, shared immutable payload, and lineage.
- `crates/jocforge-domain/src/spatial/post_window_fanout/tests.rs` — `cfg(test)` render-proposal fixture and capability checks; not a production constructor.
- `crates/jocforge-domain/src/spatial/current_features/tests.rs` — literal vectors and identity/error tests.
- `crates/jocforge-domain/tests/fixtures/current_features/INPUT-ONLY-LITERAL-VECTORS.json` — exact washed fixture copy.

Still `NOT_RUN` / deferred: post-window render packet producer, full spatial-control after-state composition, parent transaction executor, selector/outer, real ADM replay, codec/EC-3 output, and perceptual acceptance. Accordingly P1.3-D remains blocked on spatial-audio/render integration and the parent coordinator.
