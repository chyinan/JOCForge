# Independent meter installed48 leaf

## Scope

`MeterInstalled48State` is a fresh value state for the selected F04A path. It reads the same admitted ADM resource identity as the source cursor, consumes the selected source map, F03 primary activity and membership proposals, and produces one 48×3072 installed/post-window frame per normal callback. It owns previous PCM, previous installed rows, and per-host diffusion after-state. Each step returns a proposal; this leaf does not install a callback root or create an OAR packet.

The implementation uses the selected prefix membership producer for current gains, the validated selected diffusion parameters for active tails, and the shared F04A triangle window and R32 numeric helpers. It does not read expected vectors to seed gains or audio.

## In-memory validation

The authorized internal validation WAV was read in memory under its whole-resource identity: 743,696,526 bytes, SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`. A locally constructed accepted F01E caller document referenced `input-adm.wav`; this was not the source's original job document.

An earlier isolated run produced three 48×3072 values from source intervals `[0,1536)`, `[1536,3072)`, and `[3072,4608)`. A subsequent numeric audit found that the filtered-prefix weights had been promoted from F32 instead of using the specified binary64 constants. Those pre-correction hashes were withdrawn as expectations. The current code uses binary64 literals `0.85` and `0.15`, computes each binary64 square root, casts once to F32, then performs finite R32 scale products. The approved diffusion provider package was identity-validated (`c7cddb7d…7298674a`) and the corrected code was rerun on the same resource.

Fresh actual F04A output hashes are g0 `06d881…f5d1dbb`, g1 `8ba728…ff1d054`, and g2 `c03318…adb54b4`. These newly computed hashes happen to equal the withdrawn strings; the earlier run was not used as an expected vector. No independent corrected expected-output hashes are in the Wash package, so the actual replay is recorded but numeric conformance comparison remains `NOT_RUN`.

The workspace tests in `crates/jocforge-domain/tests/meter_installed48_renderer.rs` cover the cold zero-input frame shape and, separately, the cross-consumer F03R/F04A boundary. The cross-consumer fixture keeps host10's authored Control52 word10 at 1, binds the same host/callback to typed F03R `(component_enum=0,diffuse=0)`, runs the F03R value-only transition on an impulse, and confirms F04A requests its missing diffusion provider for the active event-diffuse path. It does not validate the corrected nonzero real-source F04A branch.

The isolated probe did not persist PCM or use an output hash as a later input. The selected-profile F03R typed adapter is a local translation, not a claim about a native hidden-field alias. The updated probe ran g0-g2 against the approved WAVE in memory and carried each returned raw-history proposal into the next F03R value-only transition. It also ran F04A separately with a fresh meter leaf state and the same callback's raw PCM, activity, membership, source map, and validated diffusion parameters. F03R payload hashes were `da2819…7cc4`, `7f2d20…248e4`, and `da2819…7cc4`. Both stages returned local values only: no SourceFlagsPacket or callback root was committed. No values have been joined to CurrentFeatures, selector/U04, component, direct LFE, SCP16/control16, an F06 owner, or a measurement receipt.

## F03R/F04A field separation

Selected F01E metadata has authored audio diffuse in Control52 word10 for hosts 10–17 and 20–21; word11 component is zero. Wash V2 binds the selected F03R feature inputs independently from the admitted profile as `(component_enum=0,diffuse=0)`, while preserving the original event value for F04A. For host10, the real g0-g2 probe observed word10=1 alongside typed feature diffuse=0 on callback indices 0, 1, and 2. F03R pair counts `[00,01,10,11]` were `[30,0,16,2]`, `[31,0,15,2]`, and `[30,0,16,2]`; pair `00` therefore requires the F03F audio view on each of these callbacks. This is a local typed translation; no native hidden-field mapping is claimed. The temporary pre-F04 `Installed48PostRouteRow` candidate remains nonnormative.

The F04A leaf was rerun separately over those raw callbacks and validated diffusion parameters. It produced actual 48×3072 values for g0-g2, but the washed package does not provide independent corrected expected hashes; comparison is still `NOT_RUN`. Its meter leaf frame cannot substitute for `CurrentFeaturesAudioView`, which must carry the same `SpatialControlJoinMetadata`, render-stage identity, profile, callback phase, and provider lineage from the future spatial-control renderer.
