# Spatial-control staging specification gap

## Status

P1.3-D1 remains locally verified as the v5 storage/staging foundation. The separately authorized `SOURCE-FLAGS-PRODUCER-V1` slice has now resolved the `flags_before_history` producer gap and locally verified its numeric packet output. That later producer result does not change D1's status or make D1 the owner of packet generation.

The full P1.3-D path remains blocked on `CurrentFeaturesPacket`, the `spatial_audio` root, and the parent `CallbackTransaction` integration. P1.4-C has mapped the exact mathematical FFT providers and consumer contract, but no provider validator or CurrentFeatures algorithm is implemented. Selector/outer transitions, root commits, and parent transaction execution remain outside the completed SourceFlags slice.

## SourceFlags evidence

The identity-verified supplement binds 20 synthetic success vectors, one authorized first-callback WAVE boundary vector, and 11 preflight error vectors. The producer matched all 21 literal success payloads, including the authorized first callback, and rejected all 11 error cases without producing a packet or changing committed history. `flags_before_history` is therefore recorded as `LOCALLY_VERIFIED_NUMERIC_AND_PACKET` only under the separate SourceFlags task.

The authorized WAVE path is provided to the ignored boundary test through `JOCFORGE_AUTHORIZED_WAV`; the repository stores neither that path nor its PCM. The boundary evidence covers the first 1536-sample callback only. Upstream control/render/writer production, CurrentFeatures/FFT, parent transaction execution, and whole-song product replay remain `NOT_RUN`.

## Remaining coordination

Implement and verify both exact typed provider validators as the separate P1.4-C follow-on before opening the CurrentFeatures producer. The mapping is in `docs/implementation/CURRENT-FEATURES-PROVIDER-MAP.md`; its identities use full package-manifest and provider-file bytes/SHA, with no invented provider projection digest. `CURRENT_FEATURES_PRODUCER` is `BLOCKED_ON_P1.4_C_PROVIDER_CONTRACTS`; P1.3-D stays blocked until the CurrentFeatures path, remaining owner roots, and parent callback transaction are materialized and integrated. Equal payload bytes do not authorize packet reuse across distinct parent, input, profile, provider, route, control, writer, or history lineage.
