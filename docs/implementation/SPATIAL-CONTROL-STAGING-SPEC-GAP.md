# Spatial-control staging specification gap

## Status

P1.3-D1 remains locally verified as the v5 storage/staging foundation. The separately authorized `SOURCE-FLAGS-PRODUCER-V1` slice has now resolved the `flags_before_history` producer gap and locally verified its numeric packet output. That later producer result does not change D1's status or make D1 the owner of packet generation.

The `spatial_audio` GTC slot and F04C schema candidate are mapped. The cold root, P1.4 provider validators, E2 component consumer, E3 LFE ring, E4 Type10 row assembly, and typed PostWindow fanout foundation are locally verified in their separate slices. The F03W post-window render producer, matrix/destination producer, remaining full F04 composition (including receiver/QMF), and parent `CallbackTransaction` remain deferred. CurrentFeatures math and capability are locally verified; its render producer and parent integration remain deferred. CurrentFeatures consumes `CurrentFeaturesAudioView` with an explicit callback phase; a future full F04 composer consumes `SpatialAudioPostWindowView`. Neither view constructs the render proposal, and equal payload hashes do not establish shared render identity. Render production and parent transaction execution remain `NOT_RUN`. Selector/outer transitions, root commits, and parent transaction execution remain outside the completed SourceFlags slice.

## SourceFlags evidence

The identity-verified supplement binds 20 synthetic success vectors, one authorized first-callback WAVE boundary vector, and 11 preflight error vectors. The producer matched all 21 literal success payloads, including the authorized first callback, and rejected all 11 error cases without producing a packet or changing committed history. `flags_before_history` is therefore recorded as `LOCALLY_VERIFIED_NUMERIC_AND_PACKET` only under the separate SourceFlags task.

The authorized WAVE path is provided to the ignored boundary test through `JOCFORGE_AUTHORIZED_WAV`; the repository stores neither that path nor its PCM. The boundary evidence covers the first 1536-sample callback only. Upstream control/render/writer production, CurrentFeatures/FFT, parent transaction execution, and whole-song product replay remain `NOT_RUN`.

## Remaining coordination

P1.4-C validators are complete in `crates/jocforge-domain/src/frontend/current_features_provider.rs`; their identities use the full package-manifest and provider-file bytes/SHA, with no invented provider projection digest. The machine status is `CURRENT_FEATURES_PRODUCER=LOCALLY_VERIFIED_MATH_AND_CAPABILITY__RENDER_AND_PARENT_DEFERRED`: math and capability are verified, while render production and parent integration remain deferred. P1.3-D stays blocked until the remaining owner roots, g>=2 source-stage roots, render producer, and parent callback transaction are materialized and integrated. Equal payload bytes do not authorize packet reuse across distinct parent, input, profile, provider, route, control, writer, or history lineage.

## Authoritative parent cold-state blocker

The sanitized `SPATIAL-CONTROL-COLD-SERIALIZER-BLOCKER-V1` package is pinned by v5 manifest SHA-256 `d6bd47eb6c9d19942b6dcc6c41716986d52151107ca0534c90608fae743264cd`. Evidence is classified by scope. `NATIVE_OBSERVED` covers only the bounded parent callback path rooted at RVA `0x1a7710`: 73 functions and 211 call edges advance descriptors, controls, planar buffers, and downstream processing in memory; no serializer entrypoint was observed there. The broader authorized binary scan covered 64 PE files with zero serializer/cold-persist exports, zero schema-magic matches, and no parse errors. `UNKNOWN` covers host, wrapper, other DLLs, and product-wide behavior outside that bounded path; no product-wide absence claim is made. Parent canonical bytes and receipt schema are `CLEAN_ROOM_DESIGN` only, unapproved, undelivered, and not native-observed. Serializer execution remains `NOT_RUN`; canonical preimage, section offsets, authoritative length, and authoritative SHA are absent. No bytes were guessed and `AI48ST01` was not used.

The five framing rules have different evidence states:

1. Top-level envelope is missing: magic/version or explicit absence, total-length rule, section identifiers or explicit absence, section-length widths, alignment, and terminal rule.
2. F01E embedding is missing: representation kind, presence/version, length framing, and whether the nested value is raw JSON, a binary projection, or a digest reference. The exact 4,402-byte F01E JSON preimage is pinned, but its parent embedding is not.
3. Optional diffusion/source-cache framing is partially defined but blocked. Cold caches and diffusion instances are absent, and generic optional U8 presence is known; grouping, snapshot order, diffusion records, and optional array lengths are not selected.
4. `current_flags` / `current_features` persistence is conflict-blocked. The root schema includes these owners but also says the values have no independent history; persistence has not been selected.
5. Writer-set framing is partially defined but blocked. The cold set is empty and variable records generically use U32LE count/length, but the containing section, record schema, and non-empty encoding are not defined.

Parent before/after bytes, `SpatialControlReceipt`, and single-commit authority remain blocked. Spatial render, selector, U04, and outer stages remain `NOT_RUN`; success-vector count is zero and production remains disabled.

`MINIMUM-EXTERNAL-EVIDENCE.json` defines two accepted resolution paths. First, an executable serializer must include binary identity and entrypoint, calling convention and cold-constructor-only arguments, proof that render/selector/U04/outer do not execute, complete preimage, section offsets and lengths, total length and SHA-256, and an independent parser round-trip. Second, a normative framing specification must resolve all five rules without alternatives and include an independent reference serializer, a complete cold vector, section offsets and lengths, total length and SHA-256, and an independent parser round-trip. Either path must prove that every declared field is consumed exactly once, with no scratch fields, pointers, structure padding, or unknown zero-fill, and that re-serialization is byte-exact. A byte string or digest alone is insufficient. No private key or raw PCM is needed.
