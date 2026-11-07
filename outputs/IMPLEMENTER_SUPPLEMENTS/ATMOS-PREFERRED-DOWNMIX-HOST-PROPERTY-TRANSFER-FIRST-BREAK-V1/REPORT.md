# Preferred downmix: host property serialization boundary

## Static application mapping

The hash-bound DME app-source receipt maps `PreferredStereoDownmix` UI values `Stereo`, `Surround`, and `Not Indicated` to `loro`, `ltrt`, and `not_indicated` under `preferred_downmix_mode`. A separate source excerpt shows a generic serializer that emits `PropertyKeyValue("--" + key, value[1])` for an active `valuesMap` entry. The reviewed excerpts do not bind this specific downmix map entry to that serializer invocation, and no runtime UI event or child-process argument capture exists.

The prepared `atmos_mezz` candidate pair has only one XML difference, `preferred_downmix_mode=loro/ltrt`; both schema validations are recorded as passed, but both runs are `NOT_RUN`. These candidate jobs are not a same-run DME property/argument receipt.

## Native boundary

The native receipts show a distinct key `dmixmod`, parsed by `FUN_1404ecea0`; its first proven text consumer `FUN_1404dddd0` performs presence and string-allowlist validation. No typed alias or owner connects `preferred_downmix_mode` to `dmixmod` or typed `ddp_dmixmod`. The separate numeric mix-level path does not consume the preferred enum. The static Factory resolver does not join the property value to its receiver.

The first unresolved runtime edge is the active host property-list entry or candidate XML value to the selected `encode_to_atmos_ddp` configuration setter. The further edge to typed `ddp_dmixmod` also remains `UNBOUND`. No evidence establishes that the option is applied, ignored, UI-only, or affects audio/BSI; spatial rendering is `NOT_COVERED`.

## Provenance and scope

Upstream package manifest SHA-256: `12d878c0a32c593269f07c1d6fe79d925c7f8be0c97fa4527f51f30e7c4c1f99`; its normal and `python -O` verifier passed with 5 payloads, 6 checksum entries, and 7 parent receipts. The upstream directory has no separate package seal. This package excludes its verifier code; it carries only the sanitized manifest/checksum list, source-manifest references, input-only vectors, and this bounded Wash summary. No DEE/A-B/WinDbg run, media/PCM/payload/key/license read, or implementation contact occurred.
