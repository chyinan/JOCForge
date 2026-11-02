# F13C preferred downmix / dmixmod first-usepoint boundary

## Bound static fact

The native `dmixmod` XML string reaches `FUN_1404dddd0`, which checks field presence and allowlist membership. Its configured tokens include `not_indicated`, `pro_logic`, `stereo`, and `pro_logic_ii`. The bounded trace did not identify a typed enum conversion or a selected-value store. The error branch constructs a generic name/value/error tree; the successful parse path has no bound read of the downmix result in an audio transform, BSI writer, or JOC frame path.

JOCForge's `preferred_downmix_mode` and native `dmixmod` are distinct names; the alias is unbound. The native image had zero code xrefs for the `preferred_downmix_mode` literal. Do not infer a native mapping from matching names or values.

## Audio-effect boundary

This does not prove the setting is ignored, metadata-only, or acoustically effective. Native enum conversion, post-parser consumer, audio/BSI/JOC sink, and LoRo/LtRt A/B are all `UNBOUND` or `NOT_RUN`. The one-field test plan changes only `preferred_downmix_mode` between `loro` and `ltrt` while fixing mix levels and other job controls; it has not been executed.

## Scope

Wash independently verified the source package manifest and key payload hashes; its checker passed normally and with `python -O`. Static Ghidra records only; no encoder, media, PCM, payload, or key access. The project-recorded image hash was not rehashed from the installed binary.
