# Independent meter installed48 leaf

## Scope

`MeterInstalled48State` is a fresh value state for the selected F04A path. It reads the same admitted ADM resource identity as the source cursor, consumes the selected source map, F03 primary activity and membership proposals, and produces one 48×3072 installed/post-window frame per normal callback. It owns previous PCM, previous installed rows, and per-host diffusion after-state. Each step returns a proposal; this leaf does not install a callback root or create an OAR packet.

The implementation uses the selected prefix membership producer for current gains, the validated selected diffusion parameters for active tails, and the shared F04A triangle window and R32 numeric helpers. It does not read expected vectors to seed gains or audio.

## In-memory validation

The authorized internal validation WAV was read in memory under its whole-resource identity: 743,696,526 bytes, SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`. A locally constructed accepted F01E caller document referenced `input-adm.wav`; this was not the source's original job document.

An earlier isolated run produced three 48×3072 values from source intervals `[0,1536)`, `[1536,3072)`, and `[3072,4608)`. A subsequent numeric audit found that the filtered-prefix weights had been promoted from F32 instead of using the specified binary64 constants. Those pre-correction output hashes are invalidated and are not accepted as verification. The implementation now uses `sqrt64(R64(0.85/0.15))` before the F32 conversion; the corrected leaf has not been replayed.

The workspace tests in `crates/jocforge-domain/tests/meter_installed48_renderer.rs` cover the cold zero-input frame shape and, separately, the cross-consumer F03R/F04A boundary. The cross-consumer fixture keeps host10's authored Control52 word10 at 1, binds the same host/callback to typed F03R `(component_enum=0,diffuse=0)`, runs the F03R value-only transition on an impulse, and confirms F04A requests its missing diffusion provider for the active event-diffuse path. It does not validate the corrected nonzero real-source F04A branch.

The isolated probe did not persist PCM or use an output hash as a later input. The selected-profile F03R typed adapter is now washed and materialized; it is a local typed translation, not a claim about a native hidden-field alias. The example now stages g0-g2 and runs the value-only F03R transition while keeping its raw-history proposal in memory, but the authorized WAV is absent from this checkout and `JOCFORGE_AUTHORIZED_WAV` is unset, so that updated probe has only been compiled. No leaf values have been joined to CurrentFeatures, selector/U04, component, direct LFE, SCP16/control16, an F06 owner, or a measurement receipt.

## F03R/F04A field separation

Selected F01E metadata has authored audio diffuse in Control52 word10 for hosts 10–17 and 20–21; word11 component is zero. Wash V2 binds the selected F03R feature inputs independently from the admitted profile as `(component_enum=0,diffuse=0)`, while preserving the original event value for F04A. This is a local typed translation; no native hidden-field mapping is claimed. The temporary pre-F04 `Installed48PostRouteRow` candidate remains nonnormative. The full authorized-WAV g0-g2 F03R value probe and corrected F04A replay remain unrun because the input is absent from this checkout.
