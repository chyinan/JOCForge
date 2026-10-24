# Independent meter installed48 leaf

## Scope

`MeterInstalled48State` is a fresh value state for the selected F04A path. It reads the same admitted ADM resource identity as the source cursor, consumes the selected source map, F03 primary activity and membership proposals, and produces one 48×3072 installed/post-window frame per normal callback. It owns previous PCM, previous installed rows, and per-host diffusion after-state. Each step returns a proposal; this leaf does not install a callback root or create an OAR packet.

The implementation uses the selected prefix membership producer for current gains, the validated selected diffusion parameters for active tails, and the shared F04A triangle window and R32 numeric helpers. It does not read expected vectors to seed gains or audio.

## In-memory validation

The authorized internal validation WAV was read in memory under its whole-resource identity: 743,696,526 bytes, SHA-256 `7826dcbef37ef69642a6164c97a37bcfb3f917cea74aa0f0a30d76d0a825f1d1`. A locally constructed accepted F01E caller document referenced `input-adm.wav`; this was not the source's original job document.

An earlier isolated run produced three 48×3072 values from source intervals `[0,1536)`, `[1536,3072)`, and `[3072,4608)`. A subsequent numeric audit found that the filtered-prefix weights had been promoted from F32 instead of using the specified binary64 constants. Those pre-correction output hashes are invalidated and are not accepted as verification. The implementation now uses `sqrt64(R64(0.85/0.15))` before the F32 conversion; the corrected leaf has not been replayed.

The isolated probe did not persist PCM or use an output hash as a later input. No leaf values have been joined to SourceFlags, CurrentFeatures, selector/U04, component, direct LFE, SCP16/control16, an F06 owner, or a measurement receipt.

## First full-chain stop

The selected F01E metadata includes nonzero ADM event diffuse in `control52` word 10 for hosts 10–17 and 20–21; word 11 component is zero for all objects. F03R's typed feature-control diffuse field has not been mapped to that ADM event field. Wash has explicitly confirmed they are separate interfaces, so the probe stops before F03R output and discards the g0 activity proposal. No g0 meter callback is committed, and g1/g2 full frontend callbacks remain unrun.
