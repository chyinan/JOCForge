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

The sanitized `SPATIAL-CONTROL-COLD-SERIALIZER-BLOCKER-V1` package is pinned by manifest SHA-256 `aeeb8776b77aa200908978d780b8bf29829bb9a8a70e1b512ebaba81540d4a27`. Its bounded review found no executable authoritative cold serializer in the authorized evidence. Serializer execution remains `NOT_RUN`; canonical preimage, section offsets, authoritative length, and authoritative SHA are absent. No bytes were guessed and `AI48ST01` was not used.

Five framing decisions remain missing:

1. Top-level envelope: magic/version, total length, section identifiers, section-length widths, alignment, and terminal rule.
2. F01E embedding: representation, presence/version, length framing, and whether the nested value is raw JSON, a binary projection, or a digest reference.
3. Optional diffusion/source-cache records: absence encoding, presence grouping, snapshot order, and record framing.
4. `current_flags` / `current_features` persistence: exclusion, zero arrays, or an optional callback snapshot.
5. Writer-set framing: field order/types, count/length framing, and inline versus referenced owner/generation.

Parent before/after bytes, `SpatialControlReceipt`, and single-commit authority remain blocked. Spatial render, selector, U04, and outer stages remain `NOT_RUN`; success-vector count is zero and production remains disabled.

The minimum external unlock is one of: (a) an executable authoritative cold serializer with version/build identity, input schema, reproducible cold vector, complete output bytes and offsets, and explicit field inclusion decisions; (b) a complete binary framing specification plus the full cold preimage and hash; or (c) a hash-pinned complete cold fixture with field mapping, section offsets, parsing rules, and a round-trip check. A byte string or digest alone is insufficient. None of these requires a private key, raw PCM, or unrelated research material.
