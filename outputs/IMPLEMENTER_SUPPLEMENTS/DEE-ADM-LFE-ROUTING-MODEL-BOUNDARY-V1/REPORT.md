# ADM LFE identity and bounded routing model

## Wash disposition

The hash-checked Wash-only receipt binds one ADM channel identity to physical BWF row 3 by UID/label evidence, not by the number 3. It also records the project's approved model contract for a separate direct-LFE route. The receipt does not bind the native DEE per-frame PCM row to its core LFE lane.

## Identity and route contract

- ADM UID `ATU_00000004` is associated through CHNA track index 4 with physical row 3, channel format `AC_00011004`, and explicit speaker label `RC_LFE` with suffix `LFE`.
- Compact bed ID 3 and pack-channel ordinal 3 are separate domains; they are not the evidence for the LFE identity.
- Static converter and DDP Atmos metadata paths recognize `RC_LFE`/`RC_LFE2` and place the match in descriptive label metadata. This setup branch alone is not an audio-buffer route.
- The approved model contract supports a bounded route: UID-resolved physical row → separate direct-LFE lane → independent three-slot delay → `allocated16` row 0; component rows occupy 1–15.

## Boundary

The `allocated16` and native per-frame buffer owner/generation/audio values were not captured. The input-row-to-native-core-LFE-lane join remains `UNBOUND`; the route above is `MODEL_CONTRACT_ONLY`, not native runtime parity. No DEE run or PCM, protected media, key, or license access occurred.
