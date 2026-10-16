# Spatial control state v5 amendment

## Identity chain

This repository records logical schema `selected-adm-installed48-state.v5`. The frozen package remains unchanged.

- Frozen source schema: `selected-adm-installed48-state.v3`, 7,237 bytes, SHA-256 `5138318de304b83d26e47271c1661bcc903cebcd2cea457ae98e0048b161ed46`.
- Previous local schema amendment: `CALLBACK-OUTPUT-PERSISTENCE-AMENDMENT.md`, logical v4, 4,366 bytes, SHA-256 `2e55a2e142b6482e456e40a9f3154f82574c2ed4259acab5ebab08369a84d474`.
- Approved supplement identity: `selected-adm-installed48-state.v5-wash-amendment`; MANIFEST SHA-256 `c881f9b11c169d29e532a18de38fe1f31c600b0167e18f3c0e7c3bee91945f38`.
- Supplement `SPEC.md`: 3,859 bytes, SHA-256 `c7010d04546b7661663e33ea2822d306665487ff303845bdd541a9ce8a0b4411`.
- Supplement `SCHEMA-AMENDMENT.json`: 1,629 bytes, SHA-256 `0bd3be009962e6498c584718e3b63d5aafbdbf423e474c7cf58879385b60fe94`.
- Supplement `INPUT-ONLY-VECTORS.json`: 2,387 bytes, SHA-256 `c5292f9a4dfd607704f4a442963b56b3f239e0bc69b8a9d03619fc65bdbc0d69`.

The v3 file remains source evidence; it is not renamed or presented as v5. V4 remains preserved as the previous local amendment. No files referenced by the supplement's `research_basis` are implementation inputs.

The narrower D1 storage/staging scope recorded below was explicitly approved by the user in the current task handoff on 2026-10-16. This is a scope authorization, not an additional washed supplement or a new producer-behavior claim.

## Persistent root and profile binding

The v5 persistent root excludes `/f03w_owned_fields/selection_outer/profile_dimensions`. It contributes exactly zero bytes: no field, header, null member, presence byte, or other absence tag is serialized; there is no tag for the omitted member. It is immutable admission metadata with `source_count_N=48`, `feature_bands_B=24`, `target_capacity_T=15`, and `axes=3`.

The text `active_target_count_n = "runtime integer 0..15; arrays allocate T=15 and consume prefix [0:n]"` is documentation only. It is not a number, state field, option, or canonical member.

Bind these dimensions to the selected EncodingProfile identity, validated F01E provider receipt, and validated F03 target provider receipt. `SpatialControlPrepared` and `SpatialControlStaged` retain the selected profile identity and both provider lineages in join metadata. Equal state hashes do not authorize a token with a different profile or provider lineage.

V4 removed `source_flags.current_flags` and `feature.current_features` from persistent state. V5 newly removes `selection_outer.profile_dimensions`. Each contributes zero root bytes and has no absence tag. Persistent feature cold state is `source_vector: F32[48][24]` plus `source_scalar: F32[48]`, with every element's bits equal to positive zero `0x00000000`; negative zero is invalid.

Recompute the complete root SHA-256 from the full v5 persistent preimage. Do not subtract bytes from a v3/v4 digest or use the 630-byte F03 target projection as the root digest.

## Callback packets and count domains

`SourceFlagsPacket` contains `callback_index=g` and `flags: U8[48][2]`. It is calculated from the same callback's before history plus current input, control, route, and prefix-writer set. It must not read already-replaced next history.

`CurrentFeaturesPacket` contains `callback_index=g` and band-major `features: F32[24][48]`. For every band and source, its bits equal `source_vector_after[source][band]` from the same callback.

Both packets are nonpersistent, contribute zero root bytes, have no absence tag, and belong to the same parent `CallbackTransaction` as their staged-after root. Parent failure invalidates packets and leaves the committed root unchanged. Cold/reset has no packet; packet reads return `NOT_READY`. The next callback recomputes flags and reads features from committed vectors/scalars, never from an older packet. A positive-zero cold feature view may be derived but does not imply callback 0 completed.

There is no universal active count. Candidate count, selected count, installed destinations, outer-solver dimension, committed active flags, previous-marker flags, and NNLS working-set count remain distinct. Committed active count is the derived sum of `target_active U8[15]`; capacity is 15.

## Input-only vector scope

The 14 supplement cases are input-only checks. The implementation computes actual results before comparing them to expected values; expected records must not select implementation branches. They cover cold profile binding, cold positive-zero state, packet non-persistence and readiness, feature transpose, flags-before-history behavior, count-domain separation, profile mismatch, parent failure, and reset. They do not establish product root serialization, callback execution, transaction fault execution, FFT/audio replay, or whole-chain conformance; those remain `NOT_RUN`.

## D1 scope amendment and deferred packet producers

Wash-side coordination has approved a narrower P1.3-D1 scope. D1 may implement the complete v5 persistent cold root, full canonical bytes/digest, immutable non-Clone machine, provider/profile lineage checks, generation/digest and callback/transaction join foundation, and no-local-commit boundary. P1.3-D remains blocked on the source-flags and current-features producers.

`SourceFlagsPacket` and `CurrentFeaturesPacket` remain future producer-owned opaque linear capabilities. D1 implements neither producer, nor a public raw-array constructor, zero/default packet, or forgeable lineage. The packets are not fields of `SpatialControlState`, contribute no root bytes, and do not enter its digest. Cold/reset packet reads remain `NOT_READY`. If no production producer can supply a complete typed after proposal, the non-cold prepare entry remains crate-private/deferred; no test-only constructor may expose it.

The approved `flags_before_history` record supplies only `same_before_history` and `different_current_input_or_writers` booleans, with expected relation `flags_may_differ`. It has no concrete callback input/control/route/writer-set records, `U8[48][2]` generation rule, or expected flag bytes. It is therefore classified `DEFERRED_TO_SOURCE_FLAGS_PRODUCER`; it must not be run or reported as PASS/LOCALLY_VERIFIED by D1. The full flags producer contract remains a coordination follow-up in [SPATIAL-CONTROL-STAGING-SPEC-GAP.md](SPATIAL-CONTROL-STAGING-SPEC-GAP.md).

For the same scope reason, `feature_transpose` is `DEFERRED_TO_CURRENT_FEATURES_PRODUCER`. The negative-zero after-proposal validator, F03 transition vectors, parent-failure packet invalidation, and programme reset vectors are outside the cold storage/staging slice and remain deferred to their respective typed producer or coordinator. The complete supplement vector bundle remains `NOT_RUN`; no deferred case is counted as PASS or LOCALLY_VERIFIED.

The approved `profile_mismatch` vector changes provider identity while state hashes match. A separate local input-only vector now changes only the EncodingProfile identity away from the unique selected six-dimensional row, while keeping F01E/F03 receipts, state hash, and generation equal. It must return `REJECT_PROFILE_BINDING` without changing state hash or generation. This negative test does not register or support another profile.
