# Spatial-control staging specification gap

## Status

P1.3-D1 remains locally verified as the v5 storage/staging foundation. The separately authorized `SOURCE-FLAGS-PRODUCER-V1` slice has now resolved the `flags_before_history` producer gap and locally verified its numeric packet output. That later producer result does not change D1's status or make D1 the owner of packet generation.

The `spatial_audio` programme root is now mapped to the complete F04C `F04CompositionState` and is `READY_FOR_IMPLEMENTATION` as a root mapping only. The full P1.3-D path remains blocked on the deferred post-window render producer and its exact linear packet fanout, F04-specific P1.4 provider contracts, spatial_audio root staging/integration, and the parent `CallbackTransaction`. P1.4-C's two exact math-provider validators and the CurrentFeatures math/capability producer are locally verified; `CurrentFeaturesPacket` math/capability production is complete, but its future PostWindowAudioPacket render input and parent callback transaction remain unintegrated. Render production and parent transaction execution remain `NOT_RUN`. Selector/outer transitions, root commits, and parent transaction execution remain outside the completed SourceFlags slice.

## SourceFlags evidence

The identity-verified supplement binds 20 synthetic success vectors, one authorized first-callback WAVE boundary vector, and 11 preflight error vectors. The producer matched all 21 literal success payloads, including the authorized first callback, and rejected all 11 error cases without producing a packet or changing committed history. `flags_before_history` is therefore recorded as `LOCALLY_VERIFIED_NUMERIC_AND_PACKET` only under the separate SourceFlags task.

The authorized WAVE path is provided to the ignored boundary test through `JOCFORGE_AUTHORIZED_WAV`; the repository stores neither that path nor its PCM. The boundary evidence covers the first 1536-sample callback only. Upstream control/render/writer production, CurrentFeatures/FFT, parent transaction execution, and whole-song product replay remain `NOT_RUN`.

## Remaining coordination

P1.4-C validators are complete in `crates/jocforge-domain/src/frontend/current_features_provider.rs`; their identities use the full package-manifest and provider-file bytes/SHA, with no invented provider projection digest. `CURRENT_FEATURES_PRODUCER` is `READY_FOR_IMPLEMENTATION` / `NOT_STARTED`; this does not close the CurrentFeatures algorithm or product path. P1.3-D stays blocked until CurrentFeatures, remaining owner roots, and parent callback transaction are materialized and integrated. Equal payload bytes do not authorize packet reuse across distinct parent, input, profile, provider, route, control, writer, or history lineage.
