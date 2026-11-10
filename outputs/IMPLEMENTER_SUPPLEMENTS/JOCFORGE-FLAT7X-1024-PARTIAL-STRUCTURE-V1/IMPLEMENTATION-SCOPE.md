---
status: partial_implementation_authorized
topic: JOCForge flat7x reconstruction-layout profile
spec: ETSI TS 103 420 V1.2.1
confidence: normative header field; demo/native route remains unverified
tags: [JOCForge, flat7x, E-AC-3 JOC, 1024 kbps, partial]
audience: implementation
---

# Flat7x header-layout profile: partial implementation scope

## Current target

Keep the existing JOCForge 1024 kbps, 5.1 E-AC-3 JOC audio path intact and add a selectable flat 7.X reconstruction layout. The layout value is `joc_dmx_config_idx=1`, defined by ETSI as 7.X with `Lb/Rb`. This is an additive profile variation over the current 5.1 carrier path.

The earlier 1664 kbps I0/D0 split remains historical context. This current slice uses the existing 1024 kbps profile and topology as its baseline. ADM input acceptance, audio coding, rate handling, substream assembly, and publication inherit current selected-profile behavior.

## Normative header contract

ETSI TS 103 420 V1.2.1 §6.2.2 defines the `joc_header()` field sequence:

1. `joc_dmx_config_idx`: 3 bits
2. `joc_num_objects_bits`: 6 bits
3. `joc_ext_config_idx`: 3 bits

Table 47 maps index 1 to 7.X `Lb/Rb`, index 2 to 5.X+2 `Tfl/Tfr`, and indices 5–7 to reserved values. Table 48 derives seven JOC downmix channels for index 1.

## Independent profile dimensions

- Keep the existing `JocDownmixProfile` control value (`mode0`, `phase1`, internal `index3`) as its current independent dimension.
- Add a separate typed `JocReconstructionLayout::Flat7x`, whose normative header value is `joc_dmx_config_idx=1`.
- Preserve independent types for ADM row count, E-AC-3 channel-map locations, JOC reconstruction channel count, `joc_num_objects_bits`, and `complexity_index_type_a`.
- Add the Flat7x row while retaining the current 5.1 row and its selection behavior.

## GTC and ownership projection

The layout selector is immutable attempt configuration produced by the existing profile-resolution boundary. Pass it as part of the resolved profile value to the JOC-header writer. It introduces no mutable state and no new leaf commit owner; existing attempt identity, generation receipts, codec owner, and publication owner remain the controlling boundaries.

## Partial code scope

This package authorizes the profile-model/resolver addition and serialization of the 3-bit `joc_dmx_config_idx` through the existing bit-writer boundary. Keep the current 5.1 profile's audio and bytes stable. The current change does not establish that the resulting complete file matches the official demo; full stream structure, payload validity, 7.1 rendering, and listening acceptance remain follow-up gates with status `NOT_RUN`.

Implementer verification for this slice covers: Flat7x resolves to numeric index 1; the header writer emits a 3-bit value in the syntax position; the current 5.1 row is unchanged; internal downmix controls remain independent; reserved values are rejected; object-count and complexity fields remain separate. The included vectors contain scalar/input-only cases and no encoded media.
